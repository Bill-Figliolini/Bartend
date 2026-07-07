pub(super) mod categories;
pub(super) mod inventory;
pub(super) mod recipes;
pub(super) mod serving;
pub(super) mod settings;

use iced::Element;

use crate::{
    logic::{CategoryService, ItemService},
    models::{Category, Config, Recipe},
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
    Serving(serving::Serving),
}
impl Screen {
    pub fn start(config: &Config, items: &ItemService, categories: &CategoryService) -> Self {
        let inventory = inventory::Inventory::new(config, items, categories);
        Self::Inventory(Box::new(inventory))
    }
    pub fn view(&self, items: &ItemService, categories: &CategoryService) -> Element<'_, Message> {
        match self {
            Self::Inventory(inventory) => inventory.view(items, categories),
            Self::Settings(settings) => settings.view(),
            Self::Categories(categories) => categories.view(),
            Self::Recipes(recipes) => recipes.view(),
            Self::Serving(service) => service.view(),
        }
    }
    pub fn update(&mut self, items: &ItemService, message: Message) -> Option<Command> {
        match (self, message) {
            (Self::Inventory(inventory), Message::Inventory(message)) => {
                inventory.update(items, message)
            }
            (Self::Settings(settings), Message::Settings(message)) => settings.update(message),
            (Self::Categories(categories), Message::Categories(message)) => {
                categories.update(message)
            }
            (Self::Recipes(recipes), Message::Recipes(message)) => recipes.update(message),
            (Self::Serving(service), Message::Serving(message)) => service.update(message),
            _ => unreachable!(),
        }
    }

    pub fn reload(&mut self, item_service: &ItemService, category_service: &CategoryService) {
        match self {
            Screen::Inventory(inventory) => {
                inventory.update(item_service, inventory::Message::Reload)
            }
            Screen::Settings(settings) => todo!(),
            Screen::Categories(categories) => todo!(),
            Screen::Recipes(recipes) => todo!(),
            Screen::Serving(serving) => todo!(),
        };
    }

    pub fn inventory(config: &Config, items: &ItemService, categories: &CategoryService) -> Self {
        let inventory = inventory::Inventory::new(config, items, categories);
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
