use std::path::PathBuf;

use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{container, row},
};
use rfd::AsyncFileDialog;

use crate::{
    logic::{BarCollection, CategoryService, ItemService},
    models::{
        Category, CategoryBody, CategoryFilter, CategoryID, Config, Item, ItemBody, ItemID, Recipe,
        RecipeBody,
    },
    presentation::{
        screen::{self, Screen, recipes, settings},
        widget::sidebar,
    },
};

pub fn run() -> iced::Result {
    iced::application(Bartend::start, Bartend::update, Bartend::view)
        .title(Bartend::title)
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
}

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    NoOp,
    ReloadScreen,

    OpenInventory,

    OpenSettings,
    ResetSettings,
    OpenDBPicker(PathBuf),

    OpenCategories,
    DeleteCategory(CategoryID),

    OpenRecipes,
    UpdateRecipes,
    DeleteRecipe(Recipe),

    Inventory(screen::inventory::Message),
    Settings(screen::settings::Message),
    Categories(screen::categories::Message),
    Recipes(screen::recipes::Message),
    Serving(screen::serving::Message),
}
//For instances where internals of a screen need to effect application state.
pub enum Command {
    AddItem(ItemBody, Option<CategoryID>),
    UpdateItem(Item, Option<CategoryID>),
    DeleteItem(ItemID),

    UpdateConfig(Config),

    AddCategory(CategoryBody),
    UpdateCategory(Category),

    AddRecipe(RecipeBody),
    UpdateRecipe(Recipe),
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
        let item_service = ItemService::new(&bar_collection.db.item_db());
        let category_service = CategoryService::new(&bar_collection.db);
        let screen = Screen::start(&config, &item_service, &category_service);

        Self {
            screen,
            config,
            bar_collection,
            category_service,
            item_service,
        }
    }

    fn title(&self) -> String {
        format!("Bartend")
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::ReloadScreen => {
                self.screen
                    .reload(&self.item_service, &self.category_service);
                Task::none()
            }
            Message::OpenInventory => {
                if let Screen::Inventory(_) = self.screen {
                    Task::none()
                } else {
                    self.screen =
                        Screen::inventory(&self.config, &self.item_service, &self.category_service);
                    Task::none()
                }
            }

            Message::OpenSettings => {
                if let Screen::Settings(_) = self.screen {
                } else {
                    self.screen = Screen::settings(&self.config);
                }
                Task::none()
            }
            Message::ResetSettings => {
                self.screen.update(
                    &self.item_service,
                    &self.category_service,
                    Message::Settings(settings::Message::ResetConfig(self.config.clone())),
                );
                Task::none()
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

            Message::OpenCategories => {
                if let Screen::Categories(_) = self.screen {
                } else {
                    let categories = self.category_service.get_all(CategoryFilter {});
                    self.screen = Screen::categories(&self.config, &self.category_service);
                }
                Task::none()
            }
            Message::DeleteCategory(category) => {
                self.category_service
                    .delete(&self.bar_collection.db.category_db(), category);
                Task::done(Message::ReloadScreen)
            }

            Message::OpenRecipes => {
                if let Screen::Recipes(_) = self.screen {
                } else {
                    let categories = self.category_service.get_all(CategoryFilter {});
                    let recipes = self.bar_collection.get_recipes();
                    self.screen = Screen::recipes(&self.config, categories, recipes);
                }
                Task::none()
            }
            Message::UpdateRecipes => {
                let recipes = self.bar_collection.get_recipes();
                self.screen_update(Message::Recipes(recipes::Message::Update(recipes)));
                Task::none()
            }
            Message::DeleteRecipe(recipe) => {
                self.bar_collection.delete_recipe(recipe);
                Task::done(Message::UpdateRecipes)
            }

            Message::Inventory(_) => {
                if let Some(command) = self.screen_update(message) {
                    match command {
                        Command::AddItem(item_body, category_id) => {
                            let item_id = self
                                .item_service
                                .add(&self.bar_collection.db.item_db(), &item_body);
                            if let Some(category_id) = category_id {
                                self.category_service.add_item_mapping(
                                    &self.bar_collection.db.mapping_db(),
                                    &item_id,
                                    &category_id,
                                );
                            }
                            Task::done(Message::ReloadScreen)
                        }
                        Command::UpdateItem(item, category_id) => {
                            let item_id = item.id;
                            self.item_service
                                .update(&self.bar_collection.db.item_db(), item);
                            self.category_service.update_item_mapping(
                                &self.bar_collection.db.mapping_db(),
                                &item_id,
                                &category_id,
                            );
                            Task::none()
                        }
                        Command::DeleteItem(id) => {
                            self.item_service
                                .delete(&self.bar_collection.db.item_db(), id);
                            Task::none()
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::Settings(_) => {
                if let Some(command) = self.screen_update(message) {
                    match command {
                        Command::UpdateConfig(config) => {
                            let db_changed = self.config.db_path() != config.db_path();
                            self.config = config;
                            match self.config.save() {
                                Ok(()) => {}
                                Err(e) => panic!("{e:?}"),
                            }
                            if db_changed {
                                self.bar_collection = BarCollection::new(self.config.db_path());
                            }
                            self.screen = Screen::settings(&self.config);
                            Task::none()
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::Categories(_) => {
                if let Some(command) = self.screen_update(message) {
                    match command {
                        Command::AddCategory(body) => {
                            self.category_service
                                .insert(&self.bar_collection.db.category_db(), &body);
                            Task::done(Message::ReloadScreen)
                        }
                        Command::UpdateCategory(category) => {
                            self.category_service
                                .update(&self.bar_collection.db.category_db(), &category);
                            Task::done(Message::ReloadScreen)
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::Recipes(_) => {
                if let Some(command) = self.screen_update(message) {
                    match command {
                        Command::AddRecipe(body) => {
                            self.bar_collection.add_recipe(&body);
                            Task::done(Message::UpdateRecipes)
                        }
                        Command::UpdateRecipe(recipe) => {
                            self.bar_collection.update_recipe(&recipe);
                            Task::done(Message::UpdateRecipes)
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = sidebar::Sidebar::new()
            .button("Inventory", || Message::OpenInventory)
            .button("Categories", || Message::OpenCategories)
            .button("Recipes", || Message::OpenRecipes)
            .button("Settings", || Message::OpenSettings)
            .into();

        let screen_contents = self.screen.view(&self.item_service, &self.category_service);
        let screen = container(screen_contents).width(Fill).height(Fill);

        container(row![sidebar, screen])
            .height(Fill)
            .width(Fill)
            .into()
    }
    fn screen_update(&mut self, msg: Message) -> Option<Command> {
        self.screen
            .update(&self.item_service, &self.category_service, msg)
    }
}
