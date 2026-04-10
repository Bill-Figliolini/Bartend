pub mod categories;
pub mod inventory;
pub mod settings;

use iced::Element;

use crate::{
    common::{config::Config, item::Item},
    presentation::application::{Command, Message},
};

#[derive(Debug)]
pub enum Screen {
    Inventory(inventory::Inventory),
    Settings(settings::Settings),
    Categories(categories::Categories),
}

impl Screen {
    pub fn start(config: &Config, items: Vec<Item>) -> Self {
        let mut inventory = inventory::Inventory::new(config);
        inventory.update(inventory::Message::InventoryUpdate(items));
        Self::Inventory(inventory)
    }
    pub fn view(&self) -> Element<'_, Message> {
        match self {
            Self::Inventory(inventory) => inventory.view(),
            Self::Settings(settings) => settings.view(),
            Self::Categories(categories) => categories.view(),
        }
    }
    pub fn update(&mut self, message: Message) -> Option<Command> {
        match (self, message) {
            (Self::Inventory(inventory), Message::Inventory(message)) => inventory.update(message),
            (Self::Settings(settings), Message::Settings(message)) => settings.update(message),
            (Self::Categories(categories), Message::Categories(message)) => {
                categories.update(message)
            }
            _ => unreachable!(),
        }
    }
    pub fn inventory(config: &Config, items: Vec<Item>) -> Self {
        let mut inventory = inventory::Inventory::new(config);
        Self::Inventory(inventory)
    }

    //These can be implemented as module::Messages that Application Passes to the underlying
    // Screen. Reducing the interface size and making it fully Indepenent
    pub fn reset_config(&mut self, config: &Config) {
        match self {
            Self::Settings(settings) => settings.reset(config),
            _ => unreachable!(),
        }
    }

    pub fn settings(current_config: &Config) -> Self {
        Self::Settings(settings::Settings::new(current_config))
    }
}

trait Viewable<T: Clone> {
    fn new(config: &Config) -> Self;
    fn view(&self) -> Element<'_, Message>;
    fn update(&mut self, message: T) -> Option<Command>;
}
