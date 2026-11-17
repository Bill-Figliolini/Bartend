use std::path::PathBuf;

use iced::{
    Alignment::Center,
    Border, Element,
    Length::{Fill, FillPortion},
    Task, Theme,
    widget::{column, container, pick_list, row, space, text, themer},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService, StockLevel},
    models::{
        Config, ConfigError, CustomPalette, EditableConfig, PaletteColors, PaletteFile,
        ThemeChoice, UnitSystem, palette_name,
    },
    persistence::Database,
    presentation::{
        application::{self, Context},
        color::MIN_CONTRAST,
        constants,
        input_handling::{PaletteInput, PaletteMessage, low_contrast},
        screen::{
            Panels,
            shared::{form_page, form_page_with_cards},
        },
        styling::{self, Styling, StylingOption, Stylings},
        theme::{Themes, colors_of, custom_theme},
        widget::{
            controls,
            detail::{
                Tone, caption, card, chip, gauge, hint, labeled, stat_tile, toned_chip, toned_text,
            },
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
    /// The palettes file as read when Settings opened, or `None` if it
    /// couldn't be read; then palettes can't be made or changed, so the
    /// file isn't overwritten and lost.
    palettes: Option<PaletteFile>,
    /// The palette being made or edited, shown in place of the sections.
    editor: Option<PaletteEditor>,
    /// Whether Delete is waiting to be confirmed.
    confirming_delete: bool,
}

/// A palette being made or edited.
#[derive(Debug)]
struct PaletteEditor {
    input: PaletteInput,
    /// The name of the palette being edited, or `None` for a new one.
    editing: Option<String>,
    /// Whether a save has been tried, so errors are shown.
    show_errors: bool,
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
    /// Opens the editor on a copy of the palette picked.
    NewPalette,
    /// Opens the editor on the custom palette picked.
    EditPalette,
    /// Asks to confirm deleting the custom palette picked.
    DeletePalette,
    ConfirmDeletePalette,
    /// Backs out of deleting a palette.
    KeepPalette,
    Palette(PaletteMessage),
    /// Saves the palette being edited and starts using it.
    SavePalette,
    /// Closes the editor, dropping its changes.
    CancelPalette,
    /// Sent by the app once a palette has been saved or deleted, with the
    /// palettes file as written and the palette now in use; see
    /// [`application::Message::PalettesChanged`].
    PalettesChanged(PaletteFile, ThemeChoice),
}

pub enum Command {
    UpdateConfig(Config),
    /// Writes `file`, then saves `palette` as the palette in use.
    SavePalette {
        file: PaletteFile,
        palette: CustomPalette,
    },
    /// Writes `file`, which no longer has the palette called `name`; if
    /// that was the palette in use, the default is used instead.
    DeletePalette {
        file: PaletteFile,
        name: String,
    },
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
            Command::SavePalette { file, palette } => {
                if let Err(e) = file.save() {
                    return config_error(e);
                }
                use_palette(ctx, file, ThemeChoice::Custom(palette))
            }
            Command::DeletePalette { file, name } => {
                if let Err(e) = file.save() {
                    return config_error(e);
                }
                let deleted_in_use = matches!(
                    ctx.config.theme(),
                    ThemeChoice::Custom(palette) if palette.name == name
                );
                if deleted_in_use {
                    use_palette(ctx, file, ThemeChoice::default())
                } else {
                    let theme = ctx.config.theme().clone();
                    Task::done(application::Message::PalettesChanged(file, theme))
                }
            }
        }
    }
}

fn config_error(error: ConfigError) -> Task<application::Message> {
    Task::done(application::Message::Error(
        crate::models::BartendError::Config(error),
    ))
}

/// Saves `theme` as the palette in use, in the config as last saved, so
/// any other unsaved edits in Settings stay pending. Then tells the app
/// the palettes in `file` changed, even if the config couldn't be saved,
/// since `file` was.
fn use_palette(
    ctx: &mut Context<'_>,
    file: PaletteFile,
    theme: ThemeChoice,
) -> Task<application::Message> {
    let mut config = ctx.config.editable();
    config.theme = theme;
    let config = config.commit();
    let saved = config.save();
    if saved.is_ok() {
        *ctx.config = config;
    }
    let changed = Task::done(application::Message::PalettesChanged(
        file,
        ctx.config.theme().clone(),
    ));
    match saved {
        Ok(()) => changed,
        Err(e) => Task::batch([changed, config_error(e)]),
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
        let (themes, palettes, notice) = match PaletteFile::load(current_config.palettes_path()) {
            Ok(file) => {
                let (themes, clashing) = Themes::with_custom(file.palettes());
                let skipped: Vec<String> = file.skipped().iter().cloned().chain(clashing).collect();
                let notice = (!skipped.is_empty()).then_some(ConfigError::SkippedPalettes(skipped));
                (themes, Some(file), notice)
            }
            Err(error) => (Themes::new(), None, Some(error)),
        };
        let settings = Self {
            config: current_config.editable(),
            section: Section::Database,
            themes,
            stylings: stylings.clone(),
            palettes,
            editor: None,
            confirming_delete: false,
        };
        (settings, notice)
    }

    /// What Enter does: saves the palette being edited, or else Settings.
    pub fn submit_message(&self) -> Message {
        if self.editor.is_some() {
            Message::SavePalette
        } else {
            Message::Save
        }
    }

    /// What Escape does: closes the palette editor, or backs out of a
    /// delete, if either is open.
    pub fn cancel_message(&self) -> Option<Message> {
        if self.editor.is_some() {
            Some(Message::CancelPalette)
        } else if self.confirming_delete {
            Some(Message::KeepPalette)
        } else {
            None
        }
    }

    /// The custom palette picked, as saved in the palettes file, if the
    /// palette picked is one of the user's and is still there.
    fn picked_custom(&self) -> Option<&CustomPalette> {
        let ThemeChoice::Custom(picked) = &self.config.theme else {
            return None;
        };
        self.palettes
            .as_ref()?
            .palettes()
            .iter()
            .find(|palette| palette.name == picked.name)
    }

    /// Every palette's name but `except`: what a palette mustn't be called.
    fn taken_names(&self, except: Option<&str>) -> Vec<String> {
        self.themes
            .options()
            .iter()
            .map(ToString::to_string)
            .filter(|name| Some(name.as_str()) != except)
            .collect()
    }

    /// A free name for a copy of `base`: "`base` copy", or if that's
    /// taken, "`base` copy 2", "`base` copy 3" and so on.
    fn copy_name(&self, base: &str) -> String {
        let taken = self.taken_names(None);
        // Of these, at most `taken.len()` can be taken.
        (1..=taken.len() + 1)
            .map(|count| match count {
                1 => format!("{base} copy"),
                _ => format!("{base} copy {count}"),
            })
            .find(|name| palette_name(name, taken.iter().map(String::as_str)).is_ok())
            .expect("more names to try than are taken")
    }

    fn open_editor(&mut self, input: PaletteInput, editing: Option<String>) {
        self.confirming_delete = false;
        self.editor = Some(PaletteEditor {
            input,
            editing,
            show_errors: false,
        });
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
        if let Some(editor) = &self.editor {
            let detail = palette_editor_page(styling, editor);
            return Panels { master, detail };
        }

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
                    self.palette_actions(styling),
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

    /// New, Edit and Delete for palettes, under the palette picker; or
    /// the question while a delete waits to be confirmed; or why palettes
    /// can't be changed.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn palette_actions(&self, styling: &Styling) -> Element<'_, application::Message> {
        if self.palettes.is_none() {
            return hint(
                "The palettes file can't be read, so palettes can't be made or changed \
                 until it's fixed or removed.",
            )
            .into();
        }
        let picked = self.picked_custom();
        if self.confirming_delete
            && let Some(palette) = picked
        {
            let delete = styled::danger_button(styling, "Delete")
                .on_press(settings_msg(Message::ConfirmDeletePalette));
            let keep = styled::secondary_button(styling, "Keep")
                .on_press(settings_msg(Message::KeepPalette));
            return row![
                text(format!("Delete '{}'?", palette.name)),
                space().width(Fill),
                delete,
                keep
            ]
            .spacing(constants::SPACING)
            .align_y(Center)
            .into();
        }
        let custom = picked.is_some();
        let new =
            styled::secondary_button(styling, "New").on_press(settings_msg(Message::NewPalette));
        let edit = styled::secondary_button(styling, "Edit")
            .on_press_maybe(custom.then(|| settings_msg(Message::EditPalette)));
        let delete = styled::danger_button(styling, "Delete")
            .on_press_maybe(custom.then(|| settings_msg(Message::DeletePalette)));
        row![new, edit, delete].spacing(constants::SPACING).into()
    }

    pub fn update(&mut self, message: Message) -> Option<Command> {
        match message {
            Message::SelectSection(section) => {
                // Leaving the editor drops its changes, as leaving Settings
                // does.
                self.editor = None;
                self.confirming_delete = false;
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
                self.confirming_delete = false;
                self.config.theme = self.themes.choice_for(&theme);
                None
            }
            Message::UpdateStyling(option) => {
                self.config.styling = styling::choice_for(&option);
                None
            }
            Message::ResetConfig(config) => {
                self.config = config.editable();
                self.confirming_delete = false;
                None
            }
            Message::NewPalette => {
                self.palettes.as_ref()?;
                let theme = self.themes.resolve(&self.config.theme);
                let name = self.copy_name(&theme.to_string());
                let input = PaletteInput::new(&name, &colors_of(&theme), self.taken_names(None));
                self.open_editor(input, None);
                None
            }
            Message::EditPalette => {
                let palette = self.picked_custom()?.clone();
                let taken = self.taken_names(Some(&palette.name));
                let input = PaletteInput::new(&palette.name, &palette.colors, taken);
                self.open_editor(input, Some(palette.name));
                None
            }
            Message::DeletePalette => {
                self.confirming_delete = self.picked_custom().is_some();
                None
            }
            Message::KeepPalette => {
                self.confirming_delete = false;
                None
            }
            Message::ConfirmDeletePalette => {
                self.confirming_delete = false;
                let name = self.picked_custom()?.name.clone();
                let mut file = self.palettes.clone()?;
                file.remove(&name);
                Some(Command::DeletePalette { file, name })
            }
            Message::Palette(message) => {
                if let Some(editor) = &mut self.editor {
                    editor.input.update(message);
                }
                None
            }
            Message::SavePalette => {
                let editor = self.editor.as_mut()?;
                let Ok(palette) = editor.input.validate() else {
                    editor.show_errors = true;
                    return None;
                };
                let mut file = self.palettes.clone()?;
                file.put(editor.editing.as_deref(), palette.clone());
                Some(Command::SavePalette { file, palette })
            }
            Message::CancelPalette => {
                self.editor = None;
                None
            }
            Message::PalettesChanged(file, theme) => {
                self.themes = Themes::with_custom(file.palettes()).0;
                self.palettes = Some(file);
                self.editor = None;
                self.confirming_delete = false;
                self.config.theme = theme;
                None
            }
        }
    }
}

/// The palette editor, in place of a section: the name and colors on the
/// left, and on the right a preview drawn in the colors as they stand,
/// with any contrast warnings under it.
#[cfg_attr(coverage_nightly, coverage(off))]
fn palette_editor_page<'a>(
    styling: &Styling,
    editor: &'a PaletteEditor,
) -> Element<'a, application::Message> {
    let title = match editor.editing {
        Some(_) => "Edit palette",
        None => "New palette",
    };
    let save = styled::primary_button(styling, "Save").on_press(settings_msg(Message::SavePalette));
    let cancel =
        styled::secondary_button(styling, "Cancel").on_press(settings_msg(Message::CancelPalette));
    let buttons = row![save, cancel].spacing(constants::SPACING);

    let form = card(
        styling,
        None,
        editor
            .input
            .view(styling, editor.show_errors)
            .map(|message| settings_msg(Message::Palette(message))),
    );
    let colors = editor.input.draft();
    let side = column![
        caption(styling, "Preview"),
        palette_preview(styling, &colors),
        contrast_notes(&colors),
    ]
    .spacing(constants::SPACING);
    let content = row![
        container(form).width(FillPortion(3)),
        container(side).width(FillPortion(2)),
    ]
    .spacing(16);
    form_page_with_cards(
        styling,
        title.to_string(),
        "The preview follows your changes. Save to start using the palette.",
        buttons.into(),
        content.into(),
    )
}

/// A sample of the app drawn in `colors`: a title, chips in each tone,
/// stock tiles, a card and the three kinds of button.
#[cfg_attr(coverage_nightly, coverage(off))]
fn palette_preview<'a>(
    styling: &Styling,
    colors: &PaletteColors,
) -> Element<'a, application::Message> {
    let theme = custom_theme(&CustomPalette {
        name: "Preview".to_string(),
        colors: *colors,
    });
    let noop = || application::Message::NoOp;
    let chips = row![
        toned_chip(styling, "Ready", Tone::Success),
        toned_chip(styling, "Low", Tone::Warning),
        toned_chip(styling, "Missing", Tone::Danger),
        chip(styling, "Gin"),
    ]
    .spacing(6);
    let tiles = row![
        stat_tile(
            styling,
            "Tanqueray",
            "540 ml",
            Some(gauge(styling, 0.72, StockLevel::Healthy)),
        ),
        stat_tile(
            styling,
            "Campari",
            "80 ml",
            Some(gauge(styling, 0.08, StockLevel::Critical)),
        ),
    ]
    .spacing(constants::SPACING);
    let note = card(
        styling,
        Some("Notes"),
        column![
            text("Text on a card."),
            hint("A hint, quieter than the text.")
        ]
        .spacing(4),
    );
    let buttons = row![
        styled::primary_button(styling, "Serve").on_press(noop()),
        styled::secondary_button(styling, "Edit").on_press(noop()),
        styled::danger_button(styling, "Delete").on_press(noop()),
    ]
    .spacing(constants::SPACING);
    let sample = column![
        styled::title(styling, "Negroni"),
        chips,
        tiles,
        note,
        buttons
    ]
    .spacing(constants::SPACING);
    let drawn = themer(
        Some(theme),
        container(sample).padding(constants::PADDING).width(Fill),
    )
    .background(|theme: &Theme| theme.palette().background.into())
    .text_color(|theme: &Theme| theme.palette().text);
    container(drawn)
        .padding(1)
        .style(|theme: &Theme| container::Style {
            border: Border {
                width: 1.0,
                color: theme.extended_palette().background.weak.color,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// A warning for each pair in `colors` that may be hard to read, or a
/// note that all of them read clearly.
#[cfg_attr(coverage_nightly, coverage(off))]
fn contrast_notes<'a>(colors: &PaletteColors) -> Element<'a, application::Message> {
    let low = low_contrast(colors);
    if low.is_empty() {
        return hint(format!(
            "Text reads clearly: every pair stands at least {MIN_CONTRAST}:1 apart."
        ))
        .into();
    }
    let warnings = low.into_iter().map(|(pair, ratio)| {
        text(format!(
            "{pair}: {ratio:.1}:1, under the {MIN_CONTRAST}:1 that reads easily"
        ))
        .size(constants::SMALL_TEXT)
        .style(|theme: &Theme| toned_text(theme, Tone::Warning))
        .into()
    });
    column(warnings).spacing(4).into()
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
            _ => panic!("save should produce a config to save"),
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

    mod palettes {
        use super::*;
        use crate::{
            models::{DEFAULT_THEME_NAME, Rgb},
            presentation::{
                input_handling::{PaletteMessage, Role},
                theme::navy_and_champagne,
                widget::input::ColorMessage,
            },
        };

        /// Settings over a world whose palettes file holds `names`.
        fn open_with(world: &World, names: &[&str]) -> Settings {
            let palettes: Vec<serde_json::Value> = names.iter().map(|n| palette_json(n)).collect();
            write_palettes(&world.config, &serde_json::Value::Array(palettes));
            open(&world.config)
        }

        fn pick(settings: &mut Settings, name: &str) {
            settings.update(Message::UpdateTheme(custom_theme(&sample(name))));
        }

        fn names_on_disk(world: &World) -> Vec<String> {
            PaletteFile::load(world.config.palettes_path())
                .unwrap()
                .palettes()
                .iter()
                .map(|palette| palette.name.clone())
                .collect()
        }

        /// Saves the palette being edited, applies the command, and feeds
        /// the change back to Settings as the app would.
        fn save_palette(settings: &mut Settings, world: &mut World) -> Vec<application::Message> {
            let command = settings
                .update(Message::SavePalette)
                .expect("the palette should be saved");
            settings_back(settings, drain(command.apply(&mut world.ctx())))
        }

        /// Hands any `PalettesChanged` in `messages` to Settings.
        fn settings_back(
            settings: &mut Settings,
            messages: Vec<application::Message>,
        ) -> Vec<application::Message> {
            for message in &messages {
                if let application::Message::PalettesChanged(file, theme) = message {
                    settings.update(Message::PalettesChanged(file.clone(), theme.clone()));
                }
            }
            messages
        }

        fn editor(settings: &Settings) -> &PaletteEditor {
            settings.editor.as_ref().expect("the editor should be open")
        }

        #[test]
        fn new_copies_the_picked_palette_under_a_free_name() {
            let world = World::new();
            let mut settings = open_with(&world, &["Navy & Champagne copy"]);

            settings.update(Message::NewPalette);

            let palette = editor(&settings).input.validate().unwrap();
            assert_eq!(palette.name, "Navy & Champagne copy 2");
            assert_eq!(palette.colors, colors_of(&navy_and_champagne()));
            assert_eq!(editor(&settings).editing, None);
        }

        #[test]
        fn new_copies_a_custom_palette_exactly() {
            let world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);
            pick(&mut settings, "Harbor");

            settings.update(Message::NewPalette);

            let palette = editor(&settings).input.validate().unwrap();
            assert_eq!(palette.name, "Harbor copy");
            assert_eq!(palette.colors, sample("Harbor").colors);
        }

        #[test]
        fn nothing_can_be_made_or_changed_when_the_palettes_file_is_unreadable() {
            let world = World::new();
            std::fs::write(world.config.palettes_path(), "{ broken").unwrap();
            let mut settings = open(&world.config);

            settings.update(Message::NewPalette);
            assert!(settings.editor.is_none());
        }

        #[test]
        fn only_a_custom_palette_can_be_edited_or_deleted() {
            let world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);

            settings.update(Message::EditPalette);
            settings.update(Message::DeletePalette);
            assert!(settings.editor.is_none());
            assert!(!settings.confirming_delete);

            pick(&mut settings, "Harbor");
            settings.update(Message::EditPalette);
            assert_eq!(editor(&settings).editing.as_deref(), Some("Harbor"));
        }

        #[test]
        fn saving_a_new_palette_writes_it_and_starts_using_it() {
            let mut world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);
            settings.update(Message::NewPalette);

            let messages = save_palette(&mut settings, &mut world);

            let copy = CustomPalette {
                name: "Navy & Champagne copy".to_string(),
                colors: colors_of(&navy_and_champagne()),
            };
            assert!(
                matches!(
                    messages.as_slice(),
                    [application::Message::PalettesChanged(_, ThemeChoice::Custom(used))] if *used == copy
                ),
                "{messages:?}"
            );
            assert_eq!(names_on_disk(&world), ["Harbor", "Navy & Champagne copy"]);
            assert_eq!(world.config.theme(), &ThemeChoice::Custom(copy.clone()));
            let on_disk = Config::load(Some(world.config.path().clone()), None).unwrap();
            assert_eq!(on_disk.theme(), &ThemeChoice::Custom(copy.clone()));

            assert!(settings.editor.is_none());
            assert!(settings.themes.options().contains(&custom_theme(&copy)));
            assert_eq!(settings.config.theme, ThemeChoice::Custom(copy));
        }

        #[test]
        fn saving_a_palette_leaves_other_edits_pending() {
            let mut world = World::new();
            let mut settings = open_with(&world, &[]);
            settings.update(Message::UpdateUnitSystem(UnitSystem::Imperial));
            settings.update(Message::NewPalette);

            save_palette(&mut settings, &mut world);

            assert_eq!(world.config.default_units(), UnitSystem::Metric);
            assert_eq!(save(&mut settings).default_units(), UnitSystem::Imperial);
        }

        #[test]
        fn editing_renames_the_palette_in_its_place() {
            let mut world = World::new();
            let mut settings = open_with(&world, &["Harbor", "Dusk"]);
            pick(&mut settings, "Harbor");
            settings.update(Message::EditPalette);
            settings.update(Message::Palette(PaletteMessage::Name("Port".to_string())));

            save_palette(&mut settings, &mut world);

            assert_eq!(names_on_disk(&world), ["Port", "Dusk"]);
            let ThemeChoice::Custom(used) = world.config.theme() else {
                panic!("a custom palette should be in use");
            };
            assert_eq!(used.name, "Port");
        }

        #[test]
        fn a_palette_may_keep_its_own_name_when_edited() {
            let mut world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);
            pick(&mut settings, "Harbor");
            settings.update(Message::EditPalette);
            settings.update(Message::Palette(PaletteMessage::Color(
                Role::Accent,
                ColorMessage::Hex("#FF0000".to_string()),
            )));

            save_palette(&mut settings, &mut world);

            assert_eq!(names_on_disk(&world), ["Harbor"]);
            let ThemeChoice::Custom(used) = world.config.theme() else {
                panic!("a custom palette should be in use");
            };
            assert_eq!(used.colors.accent, Rgb::new(255, 0, 0));
        }

        #[test]
        fn a_palette_that_fails_its_checks_shows_errors_and_saves_nothing() {
            let world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);
            settings.update(Message::NewPalette);
            settings.update(Message::Palette(PaletteMessage::Name("harbor".to_string())));

            assert!(settings.update(Message::SavePalette).is_none());

            assert!(editor(&settings).show_errors);
            assert_eq!(names_on_disk(&world), ["Harbor"]);
        }

        #[test]
        fn canceling_or_leaving_the_section_closes_the_editor() {
            let world = World::new();
            let mut settings = open_with(&world, &[]);
            settings.update(Message::NewPalette);
            settings.update(Message::CancelPalette);
            assert!(settings.editor.is_none());

            settings.update(Message::NewPalette);
            settings.update(Message::SelectSection(Section::Units));
            assert!(settings.editor.is_none());
        }

        #[test]
        fn deleting_asks_first_and_can_be_backed_out_of() {
            let world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);
            pick(&mut settings, "Harbor");

            settings.update(Message::DeletePalette);
            assert!(settings.confirming_delete);
            settings.update(Message::KeepPalette);
            assert!(!settings.confirming_delete);

            settings.update(Message::DeletePalette);
            settings.update(Message::UpdateTheme(Theme::Nord));
            assert!(
                !settings.confirming_delete,
                "picking another palette backs out"
            );
            assert_eq!(names_on_disk(&world), ["Harbor"]);
        }

        #[test]
        fn deleting_the_palette_in_use_switches_to_the_default() {
            let mut world = World::new();
            let mut settings = open_with(&world, &["Harbor", "Dusk"]);
            pick(&mut settings, "Harbor");
            settings.update(Message::EditPalette);
            save_palette(&mut settings, &mut world);
            assert!(matches!(world.config.theme(), ThemeChoice::Custom(_)));

            settings.update(Message::DeletePalette);
            let command = settings.update(Message::ConfirmDeletePalette).unwrap();
            settings_back(&mut settings, drain(command.apply(&mut world.ctx())));

            assert_eq!(names_on_disk(&world), ["Dusk"]);
            assert_eq!(world.config.theme(), &ThemeChoice::default());
            let on_disk = Config::load(Some(world.config.path().clone()), None).unwrap();
            assert_eq!(on_disk.theme(), &ThemeChoice::default());
            assert_eq!(settings.config.theme, ThemeChoice::default());
            assert_eq!(
                settings.themes.resolve(&settings.config.theme).to_string(),
                DEFAULT_THEME_NAME
            );
            assert!(!settings.confirming_delete);
        }

        #[test]
        fn deleting_a_palette_not_in_use_keeps_the_saved_one() {
            let mut world = World::new();
            let mut settings = open_with(&world, &["Harbor", "Dusk"]);
            pick(&mut settings, "Dusk");

            settings.update(Message::DeletePalette);
            let command = settings.update(Message::ConfirmDeletePalette).unwrap();
            let messages = settings_back(&mut settings, drain(command.apply(&mut world.ctx())));

            assert!(
                matches!(
                    messages.as_slice(),
                    [application::Message::PalettesChanged(_, ThemeChoice::BuiltIn(name))]
                        if name == DEFAULT_THEME_NAME
                ),
                "{messages:?}"
            );
            assert_eq!(names_on_disk(&world), ["Harbor"]);
            assert_eq!(settings.config.theme, ThemeChoice::default());
        }

        #[test]
        fn a_palettes_file_that_cannot_be_written_is_reported_and_nothing_changes() {
            let mut world = World::new();
            let mut settings = open_with(&world, &[]);
            std::fs::remove_file(world.config.palettes_path()).unwrap();
            std::fs::create_dir(world.config.palettes_path()).unwrap();
            settings.update(Message::NewPalette);

            let command = settings.update(Message::SavePalette).unwrap();
            let messages = drain(command.apply(&mut world.ctx()));

            assert!(
                matches!(
                    messages.as_slice(),
                    [application::Message::Error(BartendError::Config(
                        ConfigError::UnableToSavePalettes
                    ))]
                ),
                "{messages:?}"
            );
            assert_eq!(world.config.theme(), &ThemeChoice::default());
            assert!(settings.editor.is_some(), "the edit is kept to try again");
        }

        #[test]
        fn a_delete_that_cannot_be_written_is_reported_and_nothing_changes() {
            let mut world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);
            pick(&mut settings, "Harbor");
            std::fs::remove_file(world.config.palettes_path()).unwrap();
            std::fs::create_dir(world.config.palettes_path()).unwrap();

            settings.update(Message::DeletePalette);
            let command = settings.update(Message::ConfirmDeletePalette).unwrap();
            let messages = drain(command.apply(&mut world.ctx()));

            assert!(
                matches!(
                    messages.as_slice(),
                    [application::Message::Error(BartendError::Config(
                        ConfigError::UnableToSavePalettes
                    ))]
                ),
                "{messages:?}"
            );
            assert!(
                settings
                    .themes
                    .options()
                    .contains(&custom_theme(&sample("Harbor")))
            );
        }

        #[test]
        fn a_config_that_cannot_be_saved_still_shows_the_written_palettes() {
            let mut world = World::new();
            let mut settings = open_with(&world, &[]);
            settings.update(Message::NewPalette);
            let command = settings.update(Message::SavePalette).unwrap();
            // The config can no longer be written, but the palettes file can.
            std::fs::remove_file(world.config.path()).unwrap();
            std::fs::create_dir(world.config.path()).unwrap();

            let messages = settings_back(&mut settings, drain(command.apply(&mut world.ctx())));

            assert_eq!(messages.len(), 2, "{messages:?}");
            assert!(messages.iter().any(|message| matches!(
                message,
                application::Message::Error(BartendError::Config(ConfigError::WriteError))
            )));
            assert_eq!(names_on_disk(&world), ["Navy & Champagne copy"]);
            assert_eq!(world.config.theme(), &ThemeChoice::default());
            assert_eq!(settings.config.theme, ThemeChoice::default());
        }

        #[test]
        fn enter_and_escape_follow_what_is_open() {
            let world = World::new();
            let mut settings = open_with(&world, &["Harbor"]);
            assert!(matches!(settings.submit_message(), Message::Save));
            assert!(settings.cancel_message().is_none());

            pick(&mut settings, "Harbor");
            settings.update(Message::DeletePalette);
            assert!(matches!(
                settings.cancel_message(),
                Some(Message::KeepPalette)
            ));

            settings.update(Message::NewPalette);
            assert!(!settings.confirming_delete, "opening the editor backs out");
            assert!(matches!(settings.submit_message(), Message::SavePalette));
            assert!(matches!(
                settings.cancel_message(),
                Some(Message::CancelPalette)
            ));
        }

        #[test]
        fn edits_without_an_editor_are_ignored() {
            let world = World::new();
            let mut settings = open_with(&world, &[]);
            assert!(
                settings
                    .update(Message::Palette(PaletteMessage::Name("x".to_string())))
                    .is_none()
            );
            assert!(settings.update(Message::SavePalette).is_none());
            assert!(settings.update(Message::ConfirmDeletePalette).is_none());
        }
    }
}
