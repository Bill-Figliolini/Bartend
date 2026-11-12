use std::{path::PathBuf, process::exit};

use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    widget::{button, column, container, row, text},
};
use rfd::AsyncFileDialog;

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{BartendError, Config},
    persistence::Database,
    presentation::{
        screen::{self, Screen, ScreenKind},
        widget::sidebar,
    },
};

pub fn run() -> iced::Result {
    iced::application(Bartend::start, Bartend::update, Bartend::view)
        .title("Bartend")
        .theme(Bartend::theme)
        .window_size((500.0, 600.0))
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
    theme_value: Theme,
}

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    NoOp,
    ReloadScreen,
    ClearError,

    OpenScreen(ScreenKind),
    Error(BartendError),

    ResetSettings,
    OpenDBPicker(PathBuf),

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

        Self {
            screen,
            config,
            database,
            category_service,
            item_service,
            recipe_service,
            error: None,
            theme_value: Theme::KanagawaWave,
        }
    }
    fn theme(&self) -> Theme {
        self.theme_value.clone()
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
        self.screen = match kind {
            ScreenKind::Inventory => Screen::inventory(&self.config, &self.category_service),
            ScreenKind::Settings => Screen::settings(&self.config),
            ScreenKind::Categories => Screen::categories(&self.config),
            ScreenKind::Recipes => Screen::recipes(&self.config, &self.category_service),
            ScreenKind::Serving => Screen::serving(&self.config, &self.recipe_service),
        };
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::ReloadScreen => {
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
            Message::Error(e) => {
                self.error = Some(e);
                Task::none()
            }

            Message::ResetSettings => {
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
        let sidebar = sidebar::Sidebar::new()
            .button("Inventory", || Message::OpenScreen(ScreenKind::Inventory))
            .button("Categories", || Message::OpenScreen(ScreenKind::Categories))
            .button("Recipes", || Message::OpenScreen(ScreenKind::Recipes))
            .button("Serving", || Message::OpenScreen(ScreenKind::Serving))
            .button("Settings", || Message::OpenScreen(ScreenKind::Settings))
            .into();

        let mut body = Vec::new();
        let screen_contents = self.screen.view(
            &self.item_service,
            &self.category_service,
            &self.recipe_service,
        );
        if let Some(error) = &self.error {
            let error_text = text(error.to_string());
            let error_clear_button = button(text("X")).on_press(Message::ClearError);
            let error_row = row![error_text, error_clear_button];
            body.push(error_row.into());
        }
        body.push(screen_contents);
        let screen = container(column(body)).width(Fill).height(Fill);

        container(row![sidebar, screen])
            .height(Fill)
            .width(Fill)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::LogicError,
        models::{CategoryID, ItemID, UnitSystem},
        presentation::screen::{
            categories, inventory, settings,
            test_support::{World, drain},
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
            theme_value: Theme::KanagawaWave,
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

    #[test]
    fn theme_is_kanagawa_wave() {
        let (app, _dir) = app();
        assert_eq!(app.theme(), Theme::KanagawaWave);
    }
}
