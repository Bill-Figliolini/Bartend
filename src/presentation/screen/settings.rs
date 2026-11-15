use std::path::PathBuf;

use iced::{
    Length::Fill,
    Task,
    widget::{button, column, container, pick_list, row, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, EditableConfig, UnitSystem},
    persistence::Database,
    presentation::{
        application::{self, Context},
        constants,
        screen::Panels,
        widget::{
            footer::footer,
            master_list::{Entry, master_list},
        },
    },
};

#[derive(Debug)]
pub struct Settings {
    config: EditableConfig,
    /// The group of settings shown in the detail panel.
    section: Section,
}

/// A group of settings, listed in the master panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Database,
    Units,
}
impl Section {
    const ALL: [Section; 2] = [Section::Database, Section::Units];

    fn label(self) -> &'static str {
        match self {
            Section::Database => "Database",
            Section::Units => "Units",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectSection(Section),
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
        Self {
            config,
            section: Section::Database,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> Panels<'_> {
        let entries = Section::ALL.into_iter().map(|section| Entry {
            label: section.label().to_string(),
            selected: self.section == section,
            on_press: settings_msg(Message::SelectSection(section)),
            dimmed: false,
        });
        let master = master_list(None, entries);

        let heading = text(self.section.label()).size(24);
        let contents = match self.section {
            Section::Database => {
                let current_db_path = text!("Current DB Path: {}", self.config.db_path.display());
                let db_button = button("Choose DB File").on_press(
                    application::Message::OpenDBPicker(self.config.db_path.clone()),
                );
                column![current_db_path, db_button].spacing(constants::SPACING)
            }
            Section::Units => {
                let unit_text = text("Default units:");
                let unit_systems = vec![UnitSystem::Metric, UnitSystem::Imperial];
                let unit_picker = pick_list(
                    unit_systems,
                    Some(self.config.default_unit_system),
                    |unit_system: UnitSystem| settings_msg(Message::UpdateUnitSystem(unit_system)),
                );
                column![
                    row![unit_text, unit_picker]
                        .spacing(constants::SPACING)
                        .align_y(iced::Alignment::Center)
                ]
            }
        };
        let body = column![heading, contents].spacing(16).height(Fill);

        let save_button = button("Save").on_press(settings_msg(Message::Save));
        let cancel_button = button("Cancel")
            .style(button::secondary)
            .on_press(application::Message::ResetSettings);
        let footer_contents = row![save_button, cancel_button].spacing(constants::SPACING);
        let footer = footer(container(footer_contents).center_y(Fill));

        let detail = column![body, footer].into();
        Panels { master, detail }
    }

    pub fn update(&mut self, message: Message) -> Option<Command> {
        match message {
            Message::SelectSection(section) => {
                self.section = section;
                None
            }
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

fn settings_msg(msg: Message) -> application::Message {
    application::Message::Settings(msg)
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
    fn opens_on_database_and_switches_sections_keeping_edits() {
        let world = World::new();
        let mut settings = Settings::new(&world.config);
        assert_eq!(settings.section, Section::Database);
        settings.update(Message::UpdateUnitSystem(UnitSystem::Imperial));

        assert!(
            settings
                .update(Message::SelectSection(Section::Units))
                .is_none()
        );
        assert_eq!(settings.section, Section::Units);

        settings.update(Message::SelectSection(Section::Database));
        let config = save(&mut settings);
        assert_eq!(config.default_units(), UnitSystem::Imperial);
    }

    #[test]
    fn sections_have_names() {
        let labels: Vec<&str> = Section::ALL.iter().map(|section| section.label()).collect();
        assert_eq!(labels, ["Database", "Units"]);
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
