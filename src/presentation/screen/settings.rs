use std::path::PathBuf;

use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{column, row},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, EditableConfig, UnitSystem},
    persistence::Database,
    presentation::{
        application::{self, Context},
        constants,
        widget::{footer::footer, header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Settings {
    config: EditableConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    UpdateDBPath(PathBuf),
    UpdateUnitSystem(UnitSystem),
    ResetConfig(Config),
}

pub enum Command {
    UpdateConfig(Config),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::UpdateConfig(config) => {
                let db_changed = ctx.config.db_path() != config.db_path();
                let staged = if db_changed {
                    match Database::load(config.db_path()) {
                        Ok(db) => {
                            let items = match ItemService::new(&db.item_db()) {
                                Ok(service) => service,
                                Err(e) => {
                                    return Task::done(application::Message::Error(e));
                                }
                            };
                            let category = match CategoryService::new(&db.category_db()) {
                                Ok(service) => service,
                                Err(e) => {
                                    return Task::done(application::Message::Error(e));
                                }
                            };
                            let recipes = match RecipeService::new(&db.recipe_db()) {
                                Ok(service) => service,
                                Err(e) => {
                                    return Task::done(application::Message::Error(e));
                                }
                            };
                            Some((db, items, category, recipes))
                        }
                        Err(e) => {
                            return Task::done(application::Message::Error(
                                crate::models::BartendError::Db(e),
                            ));
                        }
                    }
                } else {
                    None
                };
                if let Err(e) = config.save() {
                    return Task::done(application::Message::Error(
                        crate::models::BartendError::Config(e),
                    ));
                }
                if let Some((db, items, category, recipes)) = staged {
                    *ctx.database = db;
                    *ctx.item_service = items;
                    *ctx.category_service = category;
                    *ctx.recipe_service = recipes;
                }
                *ctx.config = config;
                Task::done(application::Message::ReloadScreen)
            }
        }
    }
}

impl Settings {
    pub fn new(current_config: &Config) -> Self {
        let config = current_config.editable();
        Self { config }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> Element<'_, application::Message> {
        let text_boundary = 500;

        let title_text = title("Settings");
        let header = header(title_text);

        let current_db_path = iced::widget::text!(
            "Current DB Path: {}",
            self.config.db_path.display().to_string()
        )
        .width(text_boundary);
        let db_button = iced::widget::Button::new("Choose DB File").on_press(
            application::Message::OpenDBPicker(self.config.db_path.clone()),
        );
        let db_row = row![current_db_path, db_button];

        let divider = iced::widget::rule::horizontal(constants::DIV_SIZE);

        let unit_text = iced::widget::text("Default units:").width(text_boundary);
        let unit_systems = vec![UnitSystem::Metric, UnitSystem::Imperial];
        let unit_picker = iced::widget::pick_list(
            unit_systems,
            Some(self.config.default_unit_system),
            |unit_system: UnitSystem| {
                application::Message::Settings(Message::UpdateUnitSystem(unit_system))
            },
        );
        let unit_system_row = row![unit_text, unit_picker];

        let body = column![db_row, divider, unit_system_row].height(Fill);

        let save_button =
            iced::widget::button("Save").on_press(application::Message::Settings(Message::Save));
        let cancel_button =
            iced::widget::button("Cancel").on_press(application::Message::ResetSettings);
        let footer_contents = row![save_button, cancel_button].spacing(20);
        let footer_container = iced::widget::Container::new(footer_contents).center_x(Fill);
        let footer = footer(footer_container);

        column![header, body, footer].into()
    }

    pub fn update(&mut self, message: Message) -> Option<Command> {
        match message {
            Message::Save => Some(Command::UpdateConfig(self.config.commit())),
            Message::UpdateDBPath(db_path) => {
                self.config.db_path = db_path;
                None
            }
            Message::UpdateUnitSystem(unit_system) => {
                self.config.default_unit_system = unit_system;
                None
            }
            Message::ResetConfig(config) => {
                self.config = config.editable();
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{BartendError, ConfigError},
        persistence::DBError,
        presentation::screen::test_support::{World, drain},
    };

    fn save(settings: &mut Settings) -> Config {
        match settings.update(Message::Save) {
            Some(Command::UpdateConfig(config)) => config,
            None => panic!("save should produce a command"),
        }
    }

    fn apply(world: &mut World, config: Config) -> Vec<application::Message> {
        drain(Command::UpdateConfig(config).apply(&mut world.ctx()))
    }

    fn assert_reloads(messages: &[application::Message]) {
        assert!(
            matches!(messages, [application::Message::ReloadScreen]),
            "{messages:?}"
        );
    }

    #[test]
    fn saving_updates_and_persists_the_config() {
        let mut world = World::new();
        let mut settings = Settings::new(&world.config);
        assert!(
            settings
                .update(Message::UpdateUnitSystem(UnitSystem::Imperial))
                .is_none()
        );

        let config = save(&mut settings);
        assert_reloads(&apply(&mut world, config));

        assert_eq!(world.config.default_units(), UnitSystem::Imperial);
        let on_disk = Config::load(Some(world.config.path().clone()), None).unwrap();
        assert_eq!(on_disk.default_units(), UnitSystem::Imperial);
    }

    #[test]
    fn reset_discards_unsaved_edits() {
        let world = World::new();
        let mut settings = Settings::new(&world.config);
        settings.update(Message::UpdateUnitSystem(UnitSystem::Imperial));
        settings.update(Message::UpdateDBPath(PathBuf::from("elsewhere.db")));

        settings.update(Message::ResetConfig(world.config.clone()));

        let config = save(&mut settings);
        assert_eq!(config.default_units(), UnitSystem::Metric);
        assert_eq!(config.db_path(), world.config.db_path());
    }

    #[test]
    fn changing_the_db_path_switches_to_that_database() {
        let mut world = World::new();
        world.add_item("Tanqueray", 750.0);
        let new_db = world.dir.path().join("other.db");
        let mut settings = Settings::new(&world.config);
        settings.update(Message::UpdateDBPath(new_db.clone()));

        let config = save(&mut settings);
        assert_reloads(&apply(&mut world, config));

        assert_eq!(world.config.db_path(), &new_db);
        assert!(new_db.exists());
        assert_eq!(world.items.item_count(), 0, "services should be reloaded");
        assert_eq!(world.categories.category_count(), 0);
        assert_eq!(world.recipes.recipe_count(), 0);
    }

    #[test]
    fn an_unreadable_database_is_reported_and_nothing_changes() {
        let mut world = World::new();
        world.add_item("Tanqueray", 750.0);
        let bad_db = world.dir.path().join("not_a_db.db");
        std::fs::write(&bad_db, "this is not a sqlite database at all").unwrap();
        let mut settings = Settings::new(&world.config);
        settings.update(Message::UpdateDBPath(bad_db));

        let config = save(&mut settings);
        let messages = apply(&mut world, config);

        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Db(
                    DBError::External(_)
                ))]
            ),
            "{messages:?}"
        );
        assert_eq!(world.config.db_path(), &world.dir.path().join("bartend.db"));
        assert_eq!(world.items.item_count(), 1);
    }

    #[test]
    fn a_failed_config_save_is_reported_and_nothing_changes() {
        let mut world = World::new();
        // A config whose directory no longer exists cannot be saved.
        let gone = tempfile::tempdir().unwrap();
        let unsaveable = Config::load(
            Some(gone.path().join("config.json")),
            Some(world.config.db_path().clone()),
        )
        .unwrap();
        drop(gone);
        let mut settings = Settings::new(&unsaveable);
        settings.update(Message::UpdateUnitSystem(UnitSystem::Imperial));

        let config = save(&mut settings);
        let messages = apply(&mut world, config);

        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Config(
                    ConfigError::WriteError
                ))]
            ),
            "{messages:?}"
        );
        assert_eq!(world.config.default_units(), UnitSystem::Metric);
    }
}
