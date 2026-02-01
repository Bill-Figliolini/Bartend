use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    widget::{column, container, row, text},
};

use crate::{
    logic::{self, BarCollection},
    presentation::widget::{sidebar, text_style::title},
};

pub fn run() -> iced::Result {
    iced::application(Bartend::new, Bartend::update, Bartend::view)
        .title(Bartend::title)
        .window_size((500.0, 600.0))
        .run()
}

#[derive(Debug)]
enum Screen {
    Inventory,
}

#[derive(Debug)]
struct Bartend {
    screen: Screen,
    bar_collection: logic::BarCollection,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenInventory,
}

impl Bartend {
    fn new() -> Self {
        Bartend {
            screen: Screen::Inventory,
            bar_collection: BarCollection::new(),
        }
    }

    fn title(&self) -> String {
        format!("Bartend")
    }
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::OpenInventory => {
                self.screen = Screen::Inventory;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = column![
            title("Sidebar"),
            sidebar::button("Inventory", || Message::OpenInventory),
        ]
        .width(300)
        .padding(10);

        let screen = match &self.screen {
            Screen::Inventory => {
                let title = title("Inventory");
                let body = text("Welcome to the Inventory!");
                column![title, body]
            }
            _ => todo!(),
        };
        container(
            column![row![sidebar, container(screen).padding(10).width(Fill)].spacing(10),]
                .spacing(10),
        )
        .height(Fill)
        .width(Fill)
        .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
