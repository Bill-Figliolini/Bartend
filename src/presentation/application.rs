use std::path::PathBuf;

use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{container, row},
};
use rfd::AsyncFileDialog;

use crate::{
    common::{
        config::Config,
        item::{Item, ItemID},
        quantity::Quantity,
    },
    logic::BarCollection,
    presentation::{
        screen::{self, Screen},
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
    OpenSettings,
    OpenDBPicker(PathBuf),
    DeleteItem(ItemID),
    RefreshItems,
    Inventory(screen::inventory::Message),
    Settings(screen::settings::Message),
}
//For instances where internals of a screen need to effect application state.
pub enum Command {
    AddItem(String, Quantity),
    UpdateItem(Item),
    UpdateConfig(Config),
}

impl Bartend {
    fn start() -> Self {
        let config = match Config::load(None, None) {
            Ok(config) => config,
            Err(e) => {
                print!("{:?}", e);
                panic!("Unable to load Config")
            }
        };

        let bar_collection = BarCollection::new(config.db_path());
        let items = bar_collection.get_items();
        let screen = Screen::start(&config, items);
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
                } else {
                    let items = self.bar_collection.get_items();
                    self.screen = Screen::inventory(&self.config, items);
                }
                Task::none()
            }
            Message::OpenSettings => {
                if let Screen::Settings(_) = self.screen {
                } else {
                    self.screen = Screen::settings(&self.config);
                }
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
            Message::DeleteItem(item) => {
                self.bar_collection.delete_item(item);
                let items = self.bar_collection.get_items();
                self.screen = Screen::inventory(&self.config, items);
                Task::none()
            }
            Message::RefreshItems => {
                let items = self.bar_collection.get_items();
                self.screen = Screen::inventory(&self.config, items);
                Task::none()
            }
            Message::Inventory(_) => {
                if let Some(command) = self.screen.update(message) {
                    match command {
                        Command::AddItem(name, quantity) => {
                            self.bar_collection.add_item(&name, quantity);
                            //TODO: Can this be improved, and should it?
                            let items = self.bar_collection.get_items();
                            self.screen = Screen::inventory(&self.config, items);
                            Task::none()
                        }
                        Command::UpdateItem(item) => {
                            self.bar_collection.update_item(item);
                            let items = self.bar_collection.get_items();
                            self.screen = Screen::inventory(&self.config, items);
                            Task::none()
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
                                Ok(_) => {}
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = sidebar::Sidebar::new()
            .button("Inventory", || Message::OpenInventory)
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
