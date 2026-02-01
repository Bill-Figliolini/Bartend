use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    widget::{column, container, row, text},
};

use crate::{
    logic::{self, BarCollection},
    presentation::{
        screen::{
            inventory::{self, Inventory},
            overview::{self, Overview},
            recipe::{self, Recipe},
        },
        widget::{sidebar, text_style::title},
    },
};

pub fn run() -> iced::Result {
    //Application requires the boot component to have default implemented,
    // Which is probably a good practice as it avoids inconsistency due to partial
    // state initialization
    iced::application(AppState::new, AppState::update, AppState::view).run()
}

#[derive(Debug)]
enum Screen {
    About,
    Inventory,
}

#[derive(Debug)]
struct AppState {
    screen: Screen,
    theme: Theme,
    bar_collection: logic::BarCollection,
}

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    OpenInventory,
}

#[derive(Debug, Clone)]
pub enum Event {
    None,
}

impl AppState {
    fn new() -> Self {
        AppState {
            screen: Screen::Inventory,
            theme: Theme::TokyoNight,
            bar_collection: BarCollection::new(),
        }
    }

    //Question I still need to figure out; What should a message be in the context of this application?
    //As a personal reminder while developing. Update is for transformations over AppState
    // Given my trying to keep this program modular, this will likely be a branching function between pages' update functions.
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::OpenInventory => {
                self.screen = Screen::Inventory;
                Task::none()
            }
            Message::NoOp => Task::none(),
        }
    }

    //Personal Reminder:
    // This is for displays and views of the current app state.
    fn view(&self) -> Element<'_, Message> {
        //TODO: Create a sidebar struct to handle store width and syncronize style
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
        //Do I really need this Container?
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
