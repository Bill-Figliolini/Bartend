use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{column, container, row},
};

use crate::{
    logic::{self, BarCollection},
    persistence::Item,
    presentation::{
        screen::{self, Screen, inventory},
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
    bar_collection: logic::BarCollection,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenInventory,
    OpenSettings,
    Inventory(screen::inventory::Message),
    Settings(screen::settings::Message),
}

pub enum Command {
    AddItem(String, f32),
    UpdateItem(Item),
    DeleteItem(Item),
}

impl Bartend {
    fn new() -> Self {
        let path = "./bartend.db";
        let bar_collection = BarCollection::new(path);
        let items = bar_collection.get_items();
        let screen = Screen::start(items);
        Self {
            screen,
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
                    self.screen = Screen::settings();
                }
                Task::none()
            }
            _ => {
                if let Some(command) = self.screen.update(message) {
                    match command {
                        Command::AddItem(name, quantity) => {
                            self.bar_collection.add_item(&name, quantity);
                            //TODO: Can this be improved, and should it?
                            let items = self.bar_collection.get_items();
                            self.screen = Screen::inventory(items);
                            Task::none()
                        }
                        Command::UpdateItem(item) => todo!(),
                        Command::DeleteItem(item) => todo!(),
                    }
                } else {
                    Task::none()
                }
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
