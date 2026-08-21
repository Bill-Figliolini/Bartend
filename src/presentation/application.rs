use std::path::PathBuf;

use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    widget::{container, row},
};
use rfd::AsyncFileDialog;

use crate::{
    logic::{BarCollection, CategoryService, ItemService, RecipeService},
    models::{CategoryID, Config, ItemID, RecipeID},
    presentation::{
        screen::{self, Screen, ScreenKind},
        widget::sidebar,
    },
};

pub fn run() -> iced::Result {
    iced::application(Bartend::start, Bartend::update, Bartend::view)
        .title(Bartend::title)
        .theme(Theme::Dracula)
        .window_size((500.0, 600.0))
        .run()
}

#[derive(Debug)]
struct Bartend {
    screen: Screen,
    config: Config,
    bar_collection: BarCollection,
    category_service: CategoryService,
    item_service: ItemService,
    recipe_service: RecipeService,
}

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    NoOp,
    ReloadScreen,

    OpenScreen(ScreenKind),
    DeleteItem(ItemID),

    ResetSettings,
    OpenDBPicker(PathBuf),

    DeleteCategory(CategoryID),

    DeleteRecipe(RecipeID),

    Inventory(screen::inventory::Message),
    Settings(screen::settings::Message),
    Categories(screen::categories::Message),
    Recipes(screen::recipes::Message),
    Serving(screen::serving::Message),
}

//The mutable app state every screen's Command needs in order to apply itself.
pub(in crate::presentation) struct Context<'a> {
    pub(in crate::presentation) bar_collection: &'a mut BarCollection,
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
                print!("{e:?}");
                panic!("Unable to load Config")
            }
        };

        let bar_collection = BarCollection::new(config.db_path());
        let item_service = ItemService::new(&bar_collection.db.item_db()).unwrap();
        let category_service = CategoryService::new(&bar_collection.db.category_db()).unwrap();
        let recipe_service = RecipeService::new(&bar_collection.db.recipe_db()).unwrap();
        let screen = Screen::start(&config, &category_service);

        Self {
            screen,
            config,
            bar_collection,
            category_service,
            item_service,
            recipe_service,
        }
    }

    fn title(&self) -> String {
        "Bartend".to_string()
    }

    fn context(&mut self) -> Context<'_> {
        Context {
            bar_collection: &mut self.bar_collection,
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

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::ReloadScreen => {
                let reload_message = self.screen.reload_message(&self.config);
                self.update(reload_message)
            }
            Message::OpenScreen(kind) => {
                self.open_screen(kind);
                Task::none()
            }
            Message::DeleteItem(id) => {
                self.item_service
                    .delete(&self.bar_collection.db.item_db(), id)
                    .unwrap();
                Task::done(Message::ReloadScreen)
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

            Message::DeleteCategory(category) => {
                self.category_service
                    .delete(&self.bar_collection.db.category_db(), category)
                    .unwrap();
                Task::done(Message::ReloadScreen)
            }

            Message::DeleteRecipe(recipe) => {
                self.recipe_service
                    .delete(&self.bar_collection.db.recipe_db(), recipe)
                    .unwrap();
                Task::done(Message::ReloadScreen)
            }

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
                    Some(command) => command.apply(&mut self.context()),
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

        let screen_contents = self.screen.view(
            &self.item_service,
            &self.category_service,
            &self.recipe_service,
        );
        let screen = container(screen_contents).width(Fill).height(Fill);

        container(row![sidebar, screen])
            .height(Fill)
            .width(Fill)
            .into()
    }
}
