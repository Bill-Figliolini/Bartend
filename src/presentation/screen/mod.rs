pub(super) mod categories;
pub(super) mod inventory;
pub(super) mod recipes;
pub(super) mod service;
pub(super) mod settings;

use std::collections::HashMap;

use iced::Element;

use crate::{
    models::{Category, CategoryID, Config, Item, ItemID, Recipe},
    presentation::{
        Updateable, Viewable,
        application::{Command, Message},
        screen::recipes::Recipes,
    },
};

#[derive(Debug)]
pub enum Screen {
    Inventory(Box<inventory::Inventory>),
    Settings(settings::Settings),
    Categories(categories::Categories),
    Recipes(recipes::Recipes),
}
impl Screen {
    pub fn start(
        config: &Config,
        items: Vec<Item>,
        categories: Vec<Category>,
        mapping: HashMap<ItemID, CategoryID>,
    ) -> Self {
        let inventory = inventory::Inventory::new(config, items, categories, mapping);
        Self::Inventory(Box::new(inventory))
    }
    pub fn view(&self) -> Element<'_, Message> {
        match self {
            Self::Inventory(inventory) => inventory.view(),
            Self::Settings(settings) => settings.view(),
            Self::Categories(categories) => categories.view(),
            Self::Recipes(recipes) => recipes.view(),
        }
    }
    pub fn update(&mut self, message: Message) -> Option<Command> {
        match (self, message) {
            (Self::Inventory(inventory), Message::Inventory(message)) => inventory.update(message),
            (Self::Settings(settings), Message::Settings(message)) => settings.update(message),
            (Self::Categories(categories), Message::Categories(message)) => {
                categories.update(message)
            }
            (Self::Recipes(recipes), Message::Recipes(message)) => recipes.update(message),
            _ => unreachable!(),
        }
    }
    pub fn inventory(
        config: &Config,
        items: Vec<Item>,
        categories: Vec<Category>,
        mapping: HashMap<ItemID, CategoryID>,
    ) -> Self {
        let inventory = inventory::Inventory::new(config, items, categories, mapping);
        Self::Inventory(Box::new(inventory))
    }

    pub fn settings(current_config: &Config) -> Self {
        Self::Settings(settings::Settings::new(current_config))
    }
    pub fn categories(config: &Config, categories: Vec<Category>) -> Self {
        Self::Categories(categories::Categories::new(config, categories))
    }
    pub fn recipes(config: &Config, categories: Vec<Category>, recipes: Vec<Recipe>) -> Self {
        Screen::Recipes(Recipes::new(config, categories, recipes))
    }
}
