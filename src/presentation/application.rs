use std::path::PathBuf;

use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{container, row},
};
use rfd::AsyncFileDialog;

use crate::{
    logic::{
        BarCollection,
        category::{Category, CategoryID},
        config::Config,
        item::Item,
        quantity::Quantity,
    },
    presentation::{
        screen::{self, Screen, categories, inventory, settings},
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
}

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    OpenInventory,
    DeleteItem(Item),
    UpdateInventory,

    OpenSettings,
    ResetSettings,
    OpenDBPicker(PathBuf),

    OpenCategories,
    UpdateCategories,
    DeleteCategory(Category),

    Inventory(screen::inventory::Message),
    Settings(screen::settings::Message),
    Categories(screen::categories::Message),
}
//For instances where internals of a screen need to effect application state.
pub enum Command {
    AddItem(String, Quantity, Option<CategoryID>),
    UpdateItem(Item, Option<CategoryID>),

    UpdateConfig(Config),

    AddCategory(String),
    UpdateCategory(Category),
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
        let items = bar_collection.get_items();
        let mapping = bar_collection.get_item_mapping(&items);
        let mut screen = Screen::start(&config, items);
        let categories = bar_collection.get_categories();
        _ = screen.update(Message::Inventory(
            inventory::Message::CategoryListInitialization(categories),
        ));
        screen.update(Message::Inventory(
            inventory::Message::CategoryMappingUpdate(mapping),
        ));
        Self {
            screen,
            config,
            bar_collection,
        }
    }

    fn title(&self) -> String {
        format!("Bartend")
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::NoOp => Task::none(),

            Message::OpenInventory => {
                if let Screen::Inventory(_) = self.screen {
                    Task::none()
                } else {
                    self.screen = Screen::inventory(&self.config);
                    let categories = self.bar_collection.get_categories();
                    _ = self.screen.update(Message::Inventory(
                        inventory::Message::CategoryListInitialization(categories),
                    ));
                    Task::done(Message::UpdateInventory)
                }
            }
            Message::DeleteItem(item) => {
                self.bar_collection.delete_item(item);
                Task::done(Message::UpdateInventory)
            }
            Message::UpdateInventory => {
                let items = self.bar_collection.get_items();
                let mapping = self.bar_collection.get_item_mapping(&items);
                self.screen
                    .update(Message::Inventory(inventory::Message::InventoryUpdate(
                        items,
                    )));
                self.screen.update(Message::Inventory(
                    inventory::Message::CategoryMappingUpdate(mapping),
                ));
                Task::none()
            }

            Message::OpenSettings => {
                if let Screen::Settings(_) = self.screen {
                } else {
                    self.screen = Screen::settings(&self.config);
                }
                Task::none()
            }
            Message::ResetSettings => {
                self.screen
                    .update(Message::Settings(settings::Message::ResetConfig(
                        self.config.clone(),
                    )));
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
                    self.screen = Screen::categories(&self.config);
                }
                Task::done(Message::UpdateCategories)
            }
            Message::UpdateCategories => {
                let categories = self.bar_collection.get_categories();
                self.screen.update(Message::Categories(
                    categories::Message::CategoryListUpdate(categories),
                ));
                Task::none()
            }
            Message::DeleteCategory(category) => {
                self.bar_collection.delete_category(category);
                Task::done(Message::UpdateCategories)
            }

            Message::Inventory(_) => {
                if let Some(command) = self.screen.update(message) {
                    match command {
                        Command::AddItem(name, quantity, category_id) => {
                            let item_id = self.bar_collection.add_item(&name, quantity);
                            if let Some(category_id) = category_id {
                                self.bar_collection
                                    .add_item_category_mapping(item_id, category_id);
                            }
                            Task::done(Message::UpdateInventory)
                        }
                        Command::UpdateItem(item, category_id) => {
                            self.bar_collection
                                .update_item_category_mapping(item.id, category_id);
                            self.bar_collection.update_item(item);
                            Task::done(Message::UpdateInventory)
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::Settings(_) => {
                if let Some(command) = self.screen.update(message) {
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
                if let Some(command) = self.screen.update(message) {
                    match command {
                        Command::AddCategory(name) => {
                            self.bar_collection.add_category(name);
                            Task::done(Message::UpdateCategories)
                        }
                        Command::UpdateCategory(category) => {
                            self.bar_collection.update_category(category);
                            Task::done(Message::UpdateCategories)
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
            .button("Settings", || Message::OpenSettings)
            .into();

        let screen_contents = self.screen.view();
        let screen = container(screen_contents).width(Fill).height(Fill);

        container(row![sidebar, screen])
            .height(Fill)
            .width(Fill)
            .into()
    }
}
