use std::{path::PathBuf, process::exit};

use iced::{
    Element, Event,
    Length::Fill,
    Subscription, Task, Theme, event,
    keyboard::{self, Key, key::Named},
    widget::{button, column, container, operation, row, rule, text},
    window,
};
use rfd::AsyncFileDialog;

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{BartendError, CategoryID, Config, Recipe, RecipeID},
    persistence::Database,
    presentation::{
        constants,
        input_handling::ServingMessage,
        screen::{self, Panels, Screen, ScreenKind},
        theme::Themes,
        widget::tab_bar::tab_bar,
    },
};

pub fn run() -> iced::Result {
    iced::application(Bartend::start, Bartend::update, Bartend::view)
        .title("Bartend")
        .theme(Bartend::theme)
        .subscription(Bartend::subscription)
        .window_size((1100.0, 720.0))
        .run()
}

#[derive(Debug)]
struct Bartend {
    screen: Screen,
    config: Config,
    database: Database,
    category_service: CategoryService,
    item_service: ItemService,
    recipe_service: RecipeService,
    error: Option<BartendError>,
    themes: Themes,
    /// A theme picked in Settings but not yet saved. Shown in place of the
    /// saved theme until the edit is saved or discarded.
    preview_theme: Option<Theme>,
}

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    NoOp,
    ReloadScreen,
    ClearError,

    OpenScreen(ScreenKind),
    /// Opens the Recipes screen on the given recipe.
    ShowRecipe(RecipeID),
    /// Opens the Categories screen on the given category.
    ShowCategory(CategoryID),
    /// Opens the Serving screen with the given recipe chosen, ready to
    /// serve.
    ShowServing(RecipeID),
    Error(BartendError),

    ResetSettings,
    OpenDBPicker(PathBuf),
    /// A theme was picked in Settings: show it now, and stage it as an edit.
    PreviewTheme(Theme),

    /// Tab: focus the next field.
    FocusNext,
    /// Shift+Tab: focus the previous field.
    FocusPrevious,
    /// Enter: save the active screen's form.
    Submit,
    /// Escape: cancel the active screen's edit.
    Cancel,

    Inventory(screen::inventory::Message),
    Settings(screen::settings::Message),
    Categories(screen::categories::Message),
    Recipes(screen::recipes::Message),
    Serving(screen::serving::Message),
}

//The mutable app state every screen's Command needs in order to apply itself.
pub(in crate::presentation) struct Context<'a> {
    pub(in crate::presentation) database: &'a mut Database,
    pub(in crate::presentation) item_service: &'a mut ItemService,
    pub(in crate::presentation) category_service: &'a mut CategoryService,
    pub(in crate::presentation) recipe_service: &'a mut RecipeService,
    pub(in crate::presentation) config: &'a mut Config,
}

impl Bartend {
    fn start() -> Self {
        let config = match Config::load(None, None) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Unable to load Config: {e}");
                exit(1);
            }
        };

        let database = match Database::load(config.db_path()) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Program unable to start: {e}");
                exit(1);
            }
        };
        let item_service = match ItemService::new(&database.item_db()) {
            Ok(items) => items,
            Err(e) => {
                eprintln!("Program unable to start: {e}");
                exit(1);
            }
        };
        let category_service = match CategoryService::new(&database.category_db()) {
            Ok(categories) => categories,
            Err(e) => {
                eprintln!("Program unable to start: {e}");
                exit(1);
            }
        };
        let recipe_service = match RecipeService::new(&database.recipe_db()) {
            Ok(recipes) => recipes,
            Err(e) => {
                eprintln!("Program unable to start: {e}");
                exit(1);
            }
        };
        let screen = Screen::start(&config, &category_service);
        let themes = Themes::new();
        let error = theme_notice(&config, &themes);

        Self {
            screen,
            config,
            database,
            category_service,
            item_service,
            recipe_service,
            error,
            themes,
            preview_theme: None,
        }
    }
    fn theme(&self) -> Theme {
        match &self.preview_theme {
            Some(theme) => theme.clone(),
            None => self.themes.resolve(self.config.theme()),
        }
    }

    /// Keyboard navigation; see [`shortcut`].
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[expect(
        clippy::unused_self,
        reason = "iced passes the state; the keys mean the same on every screen"
    )]
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(shortcut)
    }

    fn context(&mut self) -> Context<'_> {
        Context {
            database: &mut self.database,
            item_service: &mut self.item_service,
            category_service: &mut self.category_service,
            recipe_service: &mut self.recipe_service,
            config: &mut self.config,
        }
    }

    fn open_screen(&mut self, kind: ScreenKind) {
        if self.screen.kind() == kind {
            return;
        }
        // Leaving Settings discards its unsaved edits, including a previewed theme.
        self.preview_theme = None;
        self.screen = match kind {
            ScreenKind::Inventory => Screen::inventory(&self.config, &self.category_service),
            ScreenKind::Settings => Screen::settings(&self.config, &self.themes),
            ScreenKind::Categories => Screen::categories(),
            ScreenKind::Recipes => Screen::recipes(&self.config, &self.category_service),
            ScreenKind::Serving => Screen::serving(&self.config),
        };
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::ReloadScreen => {
                // On Settings this follows a successful save, so the saved
                // theme already matches the preview.
                self.preview_theme = None;
                let reload_message = self.screen.reload_message(&self.config);
                self.update(reload_message)
            }
            Message::ClearError => {
                self.error = None;
                Task::none()
            }
            Message::OpenScreen(kind) => {
                self.open_screen(kind);
                Task::none()
            }
            Message::ShowRecipe(recipe) => {
                self.open_screen(ScreenKind::Recipes);
                self.update(Message::Recipes(screen::recipes::Message::Select(Some(
                    recipe,
                ))))
            }
            Message::ShowCategory(category) => {
                self.open_screen(ScreenKind::Categories);
                self.update(Message::Categories(screen::categories::Message::Select(
                    Some(category),
                )))
            }
            Message::ShowServing(id) => match self.recipe_service.get(id) {
                Ok(body) => {
                    self.open_screen(ScreenKind::Serving);
                    let choose = ServingMessage::Recipe(Recipe { id, body });
                    self.update(Message::Serving(screen::serving::Message::Input(choose)))
                }
                Err(e) => self.update(Message::Error(e)),
            },
            Message::Error(e) => {
                self.error = Some(e);
                Task::none()
            }

            Message::ResetSettings => {
                self.preview_theme = None;
                let reset_message =
                    Message::Settings(screen::settings::Message::ResetConfig(self.config.clone()));
                self.update(reset_message)
            }
            Message::OpenDBPicker(path) => Task::future(async {
                let file = AsyncFileDialog::new()
                    .add_filter("Database", &["db"])
                    .set_directory(path)
                    .save_file()
                    .await;
                file.map_or(Message::NoOp, |inner_file| {
                    let file_buf = inner_file.path().to_path_buf();
                    Message::Settings(screen::settings::Message::UpdateDBPath(file_buf))
                })
            }),
            Message::PreviewTheme(theme) => {
                if self.screen.kind() != ScreenKind::Settings {
                    return Task::none();
                }
                self.preview_theme = Some(theme.clone());
                self.update(Message::Settings(screen::settings::Message::UpdateTheme(
                    theme,
                )))
            }

            Message::FocusNext => operation::focus_next(),
            Message::FocusPrevious => operation::focus_previous(),
            Message::Submit => {
                let submit = self.screen.submit_message();
                self.update(submit)
            }
            Message::Cancel => match self.screen.cancel_message() {
                Some(cancel) => self.update(cancel),
                None => Task::none(),
            },

            Message::Inventory(_)
            | Message::Settings(_)
            | Message::Categories(_)
            | Message::Recipes(_)
            | Message::Serving(_) => {
                let command = self.screen.update(
                    &self.item_service,
                    &self.category_service,
                    &self.recipe_service,
                    message,
                );
                match command {
                    Some(command) => {
                        self.error = None;
                        command.apply(&mut self.context())
                    }
                    None => Task::none(),
                }
            }
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view(&self) -> Element<'_, Message> {
        let Panels { master, detail } = self.screen.view(
            &self.item_service,
            &self.category_service,
            &self.recipe_service,
        );

        let mut detail_contents = column![].spacing(constants::SPACING);
        if let Some(error) = &self.error {
            let error_text = text(error.to_string()).width(Fill);
            let error_clear_button = button(text("X")).on_press(Message::ClearError);
            let error_row = row![error_text, error_clear_button].spacing(constants::SPACING);
            detail_contents = detail_contents.push(error_row);
        }
        detail_contents = detail_contents.push(detail);
        let detail = container(detail_contents)
            .width(Fill)
            .height(Fill)
            .padding(constants::PADDING);

        let body = row![master, rule::vertical(constants::DIV_SIZE), detail];
        column![
            tab_bar(self.screen.kind()),
            rule::horizontal(constants::DIV_SIZE),
            body
        ]
        .height(Fill)
        .width(Fill)
        .into()
    }
}

/// Maps a key press to app navigation: Tab and Shift+Tab move focus, Enter
/// saves the active screen's form, and Escape cancels its edit.
///
/// Only keys no widget has used are mapped. So Enter picks an option in an
/// open drop-down rather than saving, and Escape in a text box only leaves
/// the box; a second Escape cancels. Held Enter and Escape do not repeat,
/// so holding Enter saves once instead of then failing on the empty form.
#[expect(
    clippy::needless_pass_by_value,
    reason = "event::listen_with needs this exact fn signature"
)]
fn shortcut(event: Event, status: event::Status, _window: window::Id) -> Option<Message> {
    if status == event::Status::Captured {
        return None;
    }
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Named(key),
        modifiers,
        repeat,
        ..
    }) = event
    else {
        return None;
    };
    match key {
        Named::Tab if modifiers.shift() => Some(Message::FocusPrevious),
        Named::Tab => Some(Message::FocusNext),
        Named::Enter if !repeat => Some(Message::Submit),
        Named::Escape if !repeat => Some(Message::Cancel),
        _ => None,
    }
}

/// The error to show at startup if the saved theme can't be found. The app
/// falls back to the default theme either way.
fn theme_notice(config: &Config, themes: &Themes) -> Option<BartendError> {
    themes
        .lookup(config.theme())
        .err()
        .map(BartendError::Config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::LogicError,
        models::{CategoryID, ConfigError, ItemID, ThemeChoice, UnitSystem},
        presentation::{
            input_handling::{CategoryMessage, EditorMessage, ItemMessage},
            screen::{
                categories, inventory, settings,
                test_support::{World, drain},
            },
        },
    };

    /// The app over a test world. The returned directory holds the config
    /// file and must outlive the app.
    fn app() -> (Bartend, tempfile::TempDir) {
        let World {
            dir,
            config,
            database,
            items,
            categories,
            recipes,
        } = World::new();
        let app = Bartend {
            screen: Screen::start(&config, &categories),
            config,
            database,
            category_service: categories,
            item_service: items,
            recipe_service: recipes,
            error: None,
            themes: Themes::new(),
            preview_theme: None,
        };
        (app, dir)
    }

    /// Sends `message`, then feeds every message the resulting tasks produce
    /// back in, as the iced runtime would.
    fn run(app: &mut Bartend, message: Message) {
        let mut queue = vec![message];
        let mut steps = 0;
        while let Some(message) = queue.pop() {
            steps += 1;
            assert!(steps < 100, "messages did not settle");
            queue.extend(drain(app.update(message)));
        }
    }

    fn logic_error() -> BartendError {
        BartendError::Logic(LogicError::InvalidItem(ItemID(1)))
    }

    #[test]
    fn opening_a_screen_switches_to_it() {
        let (mut app, _dir) = app();
        for kind in [
            ScreenKind::Categories,
            ScreenKind::Recipes,
            ScreenKind::Serving,
            ScreenKind::Settings,
            ScreenKind::Inventory,
        ] {
            run(&mut app, Message::OpenScreen(kind));
            assert_eq!(app.screen.kind(), kind);
        }
    }

    #[test]
    fn show_recipe_opens_the_recipes_screen_on_it() {
        let (mut app, _dir) = app();
        let recipe = app
            .recipe_service
            .add(
                &app.database.recipe_db(),
                crate::models::RecipeBody {
                    name: "Negroni".to_string(),
                    ingredients: Vec::new(),
                },
            )
            .unwrap();

        run(&mut app, Message::ShowRecipe(recipe));

        let Screen::Recipes(recipes) = &app.screen else {
            panic!("expected the Recipes screen, got {:?}", app.screen.kind());
        };
        assert_eq!(recipes.selected(), Some(recipe));
    }

    #[test]
    fn show_category_opens_the_categories_screen_on_it() {
        let (mut app, _dir) = app();
        let gin = app
            .category_service
            .insert(
                &app.database.category_db(),
                &crate::models::CategoryBody {
                    name: "Gin".to_string(),
                },
            )
            .unwrap();

        run(&mut app, Message::ShowCategory(gin));

        let Screen::Categories(categories) = &app.screen else {
            panic!(
                "expected the Categories screen, got {:?}",
                app.screen.kind()
            );
        };
        assert_eq!(categories.selected(), Some(gin));
    }

    #[test]
    fn show_serving_opens_the_serving_screen_with_the_recipe_chosen() {
        let (mut app, _dir) = app();
        let recipe = app
            .recipe_service
            .add(
                &app.database.recipe_db(),
                crate::models::RecipeBody {
                    name: "Negroni".to_string(),
                    ingredients: Vec::new(),
                },
            )
            .unwrap();

        run(&mut app, Message::ShowServing(recipe));

        let Screen::Serving(serving) = &app.screen else {
            panic!("expected the Serving screen, got {:?}", app.screen.kind());
        };
        assert_eq!(serving.chosen_recipe(), Some(recipe));
    }

    #[test]
    fn show_serving_a_missing_recipe_reports_it_and_stays_put() {
        let (mut app, _dir) = app();

        run(&mut app, Message::ShowServing(RecipeID(99)));

        assert!(matches!(
            app.error,
            Some(BartendError::Logic(LogicError::InvalidRecipe(RecipeID(99))))
        ));
        assert_eq!(app.screen.kind(), ScreenKind::Inventory);
    }

    #[test]
    fn reopening_the_current_screen_keeps_its_state() {
        let (mut app, _dir) = app();
        run(&mut app, Message::OpenScreen(ScreenKind::Settings));
        run(
            &mut app,
            Message::Settings(settings::Message::UpdateUnitSystem(UnitSystem::Imperial)),
        );

        run(&mut app, Message::OpenScreen(ScreenKind::Settings));
        run(&mut app, Message::Settings(settings::Message::Save));

        assert_eq!(app.config.default_units(), UnitSystem::Imperial);
    }

    #[test]
    fn errors_are_shown_until_cleared() {
        let (mut app, _dir) = app();
        run(&mut app, Message::Error(logic_error()));
        assert!(app.error.is_some());

        run(&mut app, Message::ClearError);
        assert!(app.error.is_none());
    }

    #[test]
    fn a_failing_command_shows_its_error() {
        let (mut app, _dir) = app();
        run(&mut app, Message::OpenScreen(ScreenKind::Categories));

        let (parent, child) = (CategoryID(1), CategoryID(2));
        run(
            &mut app,
            Message::Categories(categories::Message::AddRelation(parent, child)),
        );

        assert!(matches!(
            app.error,
            Some(BartendError::Logic(LogicError::CategoryNotInGraph(_)))
        ));
    }

    #[test]
    fn a_new_command_clears_the_previous_error() {
        let (mut app, _dir) = app();
        run(&mut app, Message::Error(logic_error()));

        run(
            &mut app,
            Message::Inventory(inventory::Message::DeleteItem(ItemID(1))),
        );

        assert!(app.error.is_none());
    }

    #[test]
    fn messages_without_a_command_change_nothing() {
        let (mut app, _dir) = app();
        run(&mut app, Message::Error(logic_error()));

        run(&mut app, Message::Inventory(inventory::Message::SwapUnits));
        run(&mut app, Message::NoOp);
        run(&mut app, Message::ReloadScreen);

        assert!(app.error.is_some());
        assert_eq!(app.screen.kind(), ScreenKind::Inventory);
    }

    #[test]
    fn reset_settings_discards_unsaved_edits() {
        let (mut app, _dir) = app();
        run(&mut app, Message::OpenScreen(ScreenKind::Settings));
        run(
            &mut app,
            Message::Settings(settings::Message::UpdateUnitSystem(UnitSystem::Imperial)),
        );

        run(&mut app, Message::ResetSettings);
        run(&mut app, Message::Settings(settings::Message::Save));

        assert_eq!(app.config.default_units(), UnitSystem::Metric);
    }

    #[test]
    fn opening_the_db_picker_returns_a_pending_task() {
        let (mut app, dir) = app();
        // Running the task would open a native file dialog, so only check
        // that one was created.
        let task = app.update(Message::OpenDBPicker(dir.path().to_path_buf()));
        assert_eq!(task.units(), 1);
    }

    fn press(key: Key, modifiers: keyboard::Modifiers, repeat: bool) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat,
        })
    }

    fn shortcut_for(event: Event, status: event::Status) -> Option<Message> {
        shortcut(event, status, window::Id::unique())
    }

    fn named(key: Named) -> Event {
        press(Key::Named(key), keyboard::Modifiers::default(), false)
    }

    #[test]
    fn navigation_keys_map_to_their_messages() {
        let ignored = event::Status::Ignored;
        let shift_tab = press(Key::Named(Named::Tab), keyboard::Modifiers::SHIFT, false);
        assert!(matches!(
            shortcut_for(named(Named::Tab), ignored),
            Some(Message::FocusNext)
        ));
        assert!(matches!(
            shortcut_for(shift_tab, ignored),
            Some(Message::FocusPrevious)
        ));
        assert!(matches!(
            shortcut_for(named(Named::Enter), ignored),
            Some(Message::Submit)
        ));
        assert!(matches!(
            shortcut_for(named(Named::Escape), ignored),
            Some(Message::Cancel)
        ));
    }

    #[test]
    fn keys_a_widget_used_are_left_alone() {
        let captured = event::Status::Captured;
        for key in [Named::Tab, Named::Enter, Named::Escape] {
            assert!(shortcut_for(named(key), captured).is_none(), "{key:?}");
        }
    }

    #[test]
    fn held_enter_and_escape_do_not_repeat_but_tab_does() {
        let ignored = event::Status::Ignored;
        let held = |key| press(Key::Named(key), keyboard::Modifiers::default(), true);
        assert!(shortcut_for(held(Named::Enter), ignored).is_none());
        assert!(shortcut_for(held(Named::Escape), ignored).is_none());
        assert!(matches!(
            shortcut_for(held(Named::Tab), ignored),
            Some(Message::FocusNext)
        ));
    }

    #[test]
    fn other_events_are_not_shortcuts() {
        let ignored = event::Status::Ignored;
        let letter = press(
            Key::Character("a".into()),
            keyboard::Modifiers::default(),
            false,
        );
        let release = Event::Keyboard(keyboard::Event::KeyReleased {
            key: Key::Named(Named::Enter),
            modified_key: Key::Named(Named::Enter),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
        });
        assert!(shortcut_for(letter, ignored).is_none());
        assert!(shortcut_for(named(Named::Space), ignored).is_none());
        assert!(shortcut_for(release, ignored).is_none());
    }

    #[test]
    fn focus_messages_return_a_focus_task() {
        let (mut app, _dir) = app();
        assert_eq!(app.update(Message::FocusNext).units(), 1);
        assert_eq!(app.update(Message::FocusPrevious).units(), 1);
    }

    fn type_item(app: &mut Bartend, message: ItemMessage) {
        let input = inventory::Message::Editor(EditorMessage::Input(message));
        run(app, Message::Inventory(input));
    }

    #[test]
    fn submit_saves_the_active_form() {
        let (mut app, _dir) = app();
        type_item(&mut app, ItemMessage::Name("Tanqueray".to_string()));
        type_item(&mut app, ItemMessage::Quantity("750".to_string()));

        run(&mut app, Message::Submit);

        assert_eq!(app.item_service.item_count(), 1);
        assert!(app.error.is_none());
    }

    #[test]
    fn submit_with_an_invalid_form_saves_nothing() {
        let (mut app, _dir) = app();
        run(&mut app, Message::Submit);
        assert_eq!(app.item_service.item_count(), 0);
    }

    #[test]
    fn cancel_discards_the_entry() {
        let (mut app, _dir) = app();
        run(&mut app, Message::OpenScreen(ScreenKind::Categories));
        let name = CategoryMessage::Name("Gin".to_string());
        run(
            &mut app,
            Message::Categories(categories::Message::Editor(EditorMessage::Input(name))),
        );

        run(&mut app, Message::Cancel);
        run(&mut app, Message::Submit);

        assert_eq!(app.category_service.category_count(), 0);
    }

    #[test]
    fn cancel_without_an_editor_does_nothing() {
        let (mut app, _dir) = app();
        run(&mut app, Message::OpenScreen(ScreenKind::Settings));
        run(
            &mut app,
            Message::Settings(settings::Message::UpdateUnitSystem(UnitSystem::Imperial)),
        );

        run(&mut app, Message::Cancel);
        run(&mut app, Message::Submit);

        assert_eq!(app.config.default_units(), UnitSystem::Imperial);
    }

    #[test]
    fn the_default_theme_is_kanagawa_wave() {
        let (app, _dir) = app();
        assert_eq!(app.theme(), Theme::KanagawaWave);
    }

    /// Opens Settings and picks `theme`, without saving.
    fn preview(app: &mut Bartend, theme: Theme) {
        run(app, Message::OpenScreen(ScreenKind::Settings));
        run(app, Message::PreviewTheme(theme));
    }

    #[test]
    fn picking_a_theme_shows_it_before_saving() {
        let (mut app, _dir) = app();
        preview(&mut app, Theme::Nord);

        assert_eq!(app.theme(), Theme::Nord);
        assert_eq!(app.config.theme(), &ThemeChoice::default());
    }

    #[test]
    fn cancelling_settings_restores_the_saved_theme() {
        let (mut app, _dir) = app();
        preview(&mut app, Theme::Nord);

        run(&mut app, Message::ResetSettings);
        assert_eq!(app.theme(), Theme::KanagawaWave);

        // The staged edit is discarded too.
        run(&mut app, Message::Settings(settings::Message::Save));
        assert_eq!(app.config.theme(), &ThemeChoice::default());
    }

    #[test]
    fn leaving_settings_restores_the_saved_theme() {
        let (mut app, _dir) = app();
        preview(&mut app, Theme::Nord);

        run(&mut app, Message::OpenScreen(ScreenKind::Inventory));

        assert_eq!(app.theme(), Theme::KanagawaWave);
    }

    #[test]
    fn saving_keeps_the_previewed_theme() {
        let (mut app, _dir) = app();
        preview(&mut app, Theme::Nord);

        run(&mut app, Message::Settings(settings::Message::Save));

        assert!(app.preview_theme.is_none());
        assert_eq!(app.theme(), Theme::Nord);
        let on_disk = Config::load(Some(app.config.path().clone()), None).unwrap();
        assert_eq!(on_disk.theme(), &ThemeChoice::BuiltIn("Nord".to_string()));
    }

    #[test]
    fn a_theme_picked_off_settings_is_ignored() {
        let (mut app, _dir) = app();
        run(&mut app, Message::PreviewTheme(Theme::Nord));

        assert_eq!(app.theme(), Theme::KanagawaWave);
    }

    #[test]
    fn an_unknown_saved_theme_is_reported_at_startup() {
        let (app, _dir) = app();
        let themes = Themes::new();
        assert!(theme_notice(&app.config, &themes).is_none());

        let mut config = app.config.editable();
        config.theme = ThemeChoice::BuiltIn("Not A Theme".to_string());
        let config = config.commit();

        assert!(matches!(
            theme_notice(&config, &themes),
            Some(BartendError::Config(ConfigError::UnknownTheme(_)))
        ));
        assert_eq!(themes.resolve(config.theme()), Theme::KanagawaWave);
    }
}
