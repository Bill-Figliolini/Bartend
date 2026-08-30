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
        .title(Bartend::title)
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
    fn title(&self) -> String {
        "Bartend".to_string()
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
