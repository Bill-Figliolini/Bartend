pub mod inventory;
pub mod settings;

use iced::Element;

use crate::{
    persistence::Item,
    presentation::application::{self, Message},
};

#[derive(Debug)]
pub enum Screen {
    Inventory(inventory::Inventory),
    Settings(settings::Settings),
}

impl Screen {
    pub fn start(item_list: Vec<Item>) -> Self {
        Self::Inventory(inventory::Inventory::new(item_list))
    }
    pub fn view(&self) -> Element<'_, Message> {
        match self {
            Self::Inventory(inventory) => inventory.view(),
            Self::Settings(settings) => settings.view(),
        }
    }
    pub fn update(&mut self, message: Message) -> Option<application::Command> {
        match (self, message) {
            (Self::Inventory(inventory), Message::Inventory(message)) => inventory.update(message),
            _ => unreachable!(),
        }
    }
    pub fn inventory(item_list: Vec<Item>) -> Self {
        Self::Inventory(inventory::Inventory::new(item_list))
    }
    pub fn settings() -> Self {
        Self::Settings(settings::Settings::new())
    }
}
