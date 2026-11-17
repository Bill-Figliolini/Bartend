use std::path::PathBuf;

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Task, Theme,
    widget::{column, pick_list, row, space, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, ConfigError, EditableConfig, PaletteFile, UnitSystem},
    persistence::Database,
    presentation::{
        application::{self, Context},
        constants,
        screen::{Panels, shared::form_page},
        styling::{self, Styling, StylingOption, Stylings},
        theme::Themes,
        widget::{
            controls,
            detail::labeled,
            master_list::{Entry, master_list},
            styled,
        },
    },
};

#[derive(Debug)]
pub struct Settings {
    config: EditableConfig,
    /// The group of settings shown in the detail panel.
    section: Section,
    /// The built-in themes and the user's custom palettes, read from the
    /// palettes file when Settings opens.
    themes: Themes,
    stylings: Stylings,
}

/// A group of settings, listed in the master panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Database,
    Units,
    Appearance,
}
impl Section {
    const ALL: [Section; 3] = [Section::Database, Section::Units, Section::Appearance];

    fn label(self) -> &'static str {
        match self {
            Section::Database => "Database",
            Section::Units => "Units",
            Section::Appearance => "Appearance",
        }
    }

    /// The line under the section's name, saying what its settings do.
    fn note(self) -> &'static str {
        match self {
            Section::Database => "Where your inventory and recipes are kept.",
            Section::Units => {
                "What amounts are shown in when Bartend opens. Switch any time from the top bar."
            }
            Section::Appearance => {
                "The style and palette preview as you pick them. Save to keep them."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectSection(Section),
    Save,
    UpdateDBPath(PathBuf),
    UpdateUnitSystem(UnitSystem),
    /// Sent by the app after it starts previewing the theme; see
    /// [`application::Message::PreviewTheme`].
    UpdateTheme(Theme),
    /// Sent by the app after it starts previewing the styling; see
    /// [`application::Message::PreviewStyling`].
    UpdateStyling(StylingOption),
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
    /// Opens Settings on `current_config`, reading the user's palettes
    /// from the palettes file beside it.
    ///
    /// Also gives back what went wrong reading the palettes, if anything:
    /// [`ConfigError::UnreadablePalettes`] (only the built-in themes are
    /// offered), or [`ConfigError::SkippedPalettes`] naming palettes that
    /// were malformed or named like a built-in theme.
    pub fn new(current_config: &Config, stylings: &Stylings) -> (Self, Option<ConfigError>) {
        let (themes, notice) = match PaletteFile::load(current_config.palettes_path()) {
            Ok(file) => {
                let (themes, clashing) = Themes::with_custom(file.palettes());
                let skipped: Vec<String> = file.skipped().iter().cloned().chain(clashing).collect();
                let notice = (!skipped.is_empty()).then_some(ConfigError::SkippedPalettes(skipped));
                (themes, notice)
            }
            Err(error) => (Themes::new(), Some(error)),
        };
        let settings = Self {
            config: current_config.editable(),
            section: Section::Database,
            themes,
            stylings: stylings.clone(),
        };
        (settings, notice)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self, styling: &Styling) -> Panels<'_> {
        let entries = Section::ALL.into_iter().map(|section| Entry {
            label: section.label().to_string(),
            selected: self.section == section,
            on_press: settings_msg(Message::SelectSection(section)),
            dimmed: false,
        });
        let master = master_list(styling, None, entries, "");

        let fields: Element<'_, application::Message> = match self.section {
            Section::Database => {
                let path = text(self.config.db_path.display().to_string());
                let choose = styled::secondary_button(styling, "Choose file…").on_press(
                    application::Message::OpenDBPicker(self.config.db_path.clone()),
                );
                labeled(
                    styling,
                    "Database file",
                    row![path, space().width(Fill), choose]
                        .spacing(constants::SPACING)
                        .align_y(Center),
                )
            }
            Section::Units => {
                let unit_systems = vec![UnitSystem::Metric, UnitSystem::Imperial];
                let unit_picker = pick_list(
                    unit_systems,
                    Some(self.config.default_unit_system),
                    |unit_system: UnitSystem| settings_msg(Message::UpdateUnitSystem(unit_system)),
                )
                .width(Fill)
                .style(controls::picker(styling))
                .menu_style(controls::menu(styling));
                labeled(styling, "Default units", unit_picker)
            }
            Section::Appearance => {
                let styling_picker = pick_list(
                    self.stylings.options(),
                    Some(self.stylings.resolve(&self.config.styling)),
                    application::Message::PreviewStyling,
                )
                .width(Fill)
                .style(controls::picker(styling))
                .menu_style(controls::menu(styling));
                let theme_picker = pick_list(
                    self.themes.options(),
                    Some(self.themes.resolve(&self.config.theme)),
                    application::Message::PreviewTheme,
                )
                .width(Fill)
                .style(controls::picker(styling))
                .menu_style(controls::menu(styling));
                column![
                    labeled(styling, "Style", styling_picker),
                    labeled(styling, "Palette", theme_picker),
                ]
                .spacing(constants::SPACING)
                .into()
            }
        };

        let save_button =
            styled::primary_button(styling, "Save").on_press(settings_msg(Message::Save));
        let cancel_button = styled::secondary_button(styling, "Cancel")
            .on_press(application::Message::ResetSettings);
        let buttons = row![save_button, cancel_button].spacing(constants::SPACING);

        let detail = form_page(
            styling,
            self.section.label().to_string(),
            self.section.note(),
            buttons.into(),
            fields,
        );
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
            Message::UpdateTheme(theme) => {
                self.config.theme = self.themes.choice_for(&theme);
                None
            }
            Message::UpdateStyling(option) => {
                self.config.styling = styling::choice_for(&option);
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
        models::{BartendError, ConfigError, StylingChoice, ThemeChoice, palette_sample as sample},
        persistence::DBError,
        presentation::{
            screen::test_support::{World, drain},
            styling::PLAIN_OPTION,
            theme::custom_theme,
        },
    };

    /// Settings opened on `config`, with the test styling registered.
    fn open(config: &Config) -> Settings {
        Settings::new(config, &Stylings::with_plain()).0
    }

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
        let mut settings = open(&world.config);
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
    fn saving_persists_the_chosen_theme() {
        let mut world = World::new();
        let mut settings = open(&world.config);
        assert!(settings.update(Message::UpdateTheme(Theme::Nord)).is_none());

        let config = save(&mut settings);
        assert_reloads(&apply(&mut world, config));

        let nord = ThemeChoice::BuiltIn("Nord".to_string());
        assert_eq!(world.config.theme(), &nord);
        let on_disk = Config::load(Some(world.config.path().clone()), None).unwrap();
        assert_eq!(on_disk.theme(), &nord);
    }

    #[test]
    fn saving_persists_the_chosen_styling() {
        let mut world = World::new();
        let mut settings = open(&world.config);
        assert!(
            settings
                .update(Message::UpdateStyling(PLAIN_OPTION))
                .is_none()
        );

        let config = save(&mut settings);
        assert_reloads(&apply(&mut world, config));

        let plain = StylingChoice::BuiltIn(PLAIN_OPTION.name.to_string());
        assert_eq!(world.config.styling(), &plain);
        let on_disk = Config::load(Some(world.config.path().clone()), None).unwrap();
        assert_eq!(on_disk.styling(), &plain);
    }

    #[test]
    fn opens_on_database_and_switches_sections_keeping_edits() {
        let world = World::new();
        let mut settings = open(&world.config);
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
        assert_eq!(labels, ["Database", "Units", "Appearance"]);
    }

    #[test]
    fn reset_discards_unsaved_edits() {
        let world = World::new();
        let mut settings = open(&world.config);
        settings.update(Message::UpdateUnitSystem(UnitSystem::Imperial));
        settings.update(Message::UpdateDBPath(PathBuf::from("elsewhere.db")));
        settings.update(Message::UpdateTheme(Theme::Dracula));
        settings.update(Message::UpdateStyling(PLAIN_OPTION));

        settings.update(Message::ResetConfig(world.config.clone()));

        let config = save(&mut settings);
        assert_eq!(config.default_units(), UnitSystem::Metric);
        assert_eq!(config.theme(), world.config.theme());
        assert_eq!(config.styling(), world.config.styling());
        assert_eq!(config.db_path(), world.config.db_path());
    }

    #[test]
    fn changing_the_db_path_switches_to_that_database() {
        let mut world = World::new();
        world.add_item("Tanqueray", 750.0);
        let new_db = world.dir.path().join("other.db");
        let mut settings = open(&world.config);
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
        let mut settings = open(&world.config);
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
        let mut settings = open(&unsaveable);
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

    /// Writes `palettes` as the palettes file beside `config`.
    fn write_palettes(config: &Config, palettes: &serde_json::Value) {
        std::fs::write(config.palettes_path(), palettes.to_string()).unwrap();
    }

    fn palette_json(name: &str) -> serde_json::Value {
        serde_json::to_value(sample(name)).unwrap()
    }

    #[test]
    fn opening_offers_the_saved_custom_palettes() {
        let world = World::new();
        write_palettes(&world.config, &serde_json::json!([palette_json("Harbor")]));

        let (settings, notice) = Settings::new(&world.config, &Stylings::with_plain());

        assert_eq!(notice, None);
        assert!(
            settings
                .themes
                .options()
                .contains(&custom_theme(&sample("Harbor")))
        );
    }

    #[test]
    fn without_a_palettes_file_only_built_in_themes_are_offered() {
        let world = World::new();

        let (settings, notice) = Settings::new(&world.config, &Stylings::with_plain());

        assert_eq!(notice, None);
        assert_eq!(settings.themes.options(), Themes::new().options());
    }

    #[test]
    fn saving_a_custom_palette_keeps_a_copy_of_it_in_the_config() {
        let mut world = World::new();
        write_palettes(&world.config, &serde_json::json!([palette_json("Harbor")]));
        let mut settings = open(&world.config);
        settings.update(Message::UpdateTheme(custom_theme(&sample("Harbor"))));

        let config = save(&mut settings);
        assert_reloads(&apply(&mut world, config));

        let harbor = ThemeChoice::Custom(sample("Harbor"));
        assert_eq!(world.config.theme(), &harbor);
        let on_disk = Config::load(Some(world.config.path().clone()), None).unwrap();
        assert_eq!(on_disk.theme(), &harbor);
    }

    #[test]
    fn palettes_that_cannot_be_offered_are_reported_together() {
        let world = World::new();
        write_palettes(
            &world.config,
            &serde_json::json!([
                palette_json("Harbor"),
                {"name": "Broken"},
                palette_json("Nord"),
            ]),
        );

        let (settings, notice) = Settings::new(&world.config, &Stylings::with_plain());

        assert_eq!(
            notice,
            Some(ConfigError::SkippedPalettes(vec![
                "Broken".to_string(),
                "Nord".to_string()
            ]))
        );
        assert_eq!(
            settings.themes.options().len(),
            Themes::new().options().len() + 1
        );
    }

    #[test]
    fn an_unreadable_palettes_file_is_reported_and_only_built_ins_are_offered() {
        let world = World::new();
        std::fs::write(world.config.palettes_path(), "{ not a list").unwrap();

        let (settings, notice) = Settings::new(&world.config, &Stylings::with_plain());

        assert_eq!(notice, Some(ConfigError::UnreadablePalettes));
        assert_eq!(settings.themes.options(), Themes::new().options());
    }
}
