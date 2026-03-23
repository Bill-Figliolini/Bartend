use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{column, container, row},
};

use crate::{
    common::{
        config::Config,
        item::{Item, ItemID},
        quantity::Quantity,
    },
    logic::{self, BarCollection},
    presentation::{
        screen::{self, Screen},
        widget::sidebar,
    },
};

pub fn run() -> iced::Result {
    iced::application(Bartend::new, Bartend::update, Bartend::view)
        .title(Bartend::title)
        .window_size((500.0, 600.0))
        .run()
}

#[derive(Debug)]
struct Bartend {
    screen: Screen,
    config: Config,
    bar_collection: logic::BarCollection,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenInventory,
    OpenSettings,
    DeleteItem(ItemID),
    RefreshItems,
    Inventory(screen::inventory::Message),
    Settings(screen::settings::Message),
}
//For instances where internals of a screen need to effect application state.
pub enum Command {
    AddItem(String, Quantity),
    UpdateItem(Item),
}

impl Bartend {
    fn new() -> Self {
        let config = match Config::load() {
            Ok(config) => config,
            Err(_) => panic!("Unable to load Config"),
        };

        let bar_collection = BarCollection::new(config.db_path());
        let items = bar_collection.get_items();
        let screen = Screen::start(items);
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
            Message::OpenInventory => {
                if let Screen::Inventory(_) = self.screen {
                } else {
                    let items = self.bar_collection.get_items();
                    self.screen = Screen::inventory(items);
                }
                Task::none()
            }
            Message::OpenSettings => {
                if let Screen::Settings(_) = self.screen {
                } else {
                    self.screen = Screen::settings(self.config.clone());
                }
                Task::none()
            }
            Message::DeleteItem(item) => {
                self.bar_collection.delete_item(item);
                let items = self.bar_collection.get_items();
                self.screen = Screen::inventory(items);
                Task::none()
            }
            Message::RefreshItems => {
                let items = self.bar_collection.get_items();
                self.screen = Screen::inventory(items);
                Task::none()
            }
            Message::Inventory(_) => {
                if let Some(command) = self.screen.update(message) {
                    match command {
                        Command::AddItem(name, quantity) => {
                            self.bar_collection.add_item(&name, quantity);
                            //TODO: Can this be improved, and should it?
                            let items = self.bar_collection.get_items();
                            self.screen = Screen::inventory(items);
                            Task::none()
                        }
                        Command::UpdateItem(item) => {
                            self.bar_collection.update_item(item);
                            let items = self.bar_collection.get_items();
                            self.screen = Screen::inventory(items);
                            Task::none()
                        }
                    }
                } else {
                    Task::none()
                }
            }
            Message::Settings(_) => {
                self.screen.update(message);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = column![
            sidebar::button("Inventory", || Message::OpenInventory),
            sidebar::button("Settings", || Message::OpenSettings),
        ]
        .width(300)
        .padding(10);

        let screen = self.screen.view();
        container(
            column![row![sidebar, container(screen).padding(10).width(Fill)].spacing(10),]
                .spacing(10),
        )
        .height(Fill)
        .width(Fill)
        .into()
    }
}
