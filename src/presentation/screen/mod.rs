pub mod inventory;
pub mod settings;

use iced::Element;

use crate::{
    common::{config::Config, item::Item},
    presentation::application::{self, Message},
};

#[derive(Debug)]
pub enum Screen {
    Inventory(inventory::Inventory),
    Settings(settings::Settings),
}

impl Screen {
    pub fn start(items: Vec<Item>) -> Self {
        Self::Inventory(inventory::Inventory::new(items))
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
            (Self::Settings(settings), Message::Settings(message)) => settings.update(message),
            _ => unreachable!(),
        }
    }
    pub fn inventory(items: Vec<Item>) -> Self {
        Self::Inventory(inventory::Inventory::new(items))
    }
    pub fn settings(current_config: &Config) -> Self {
        Self::Settings(settings::Settings::new(current_config))
    }
}
