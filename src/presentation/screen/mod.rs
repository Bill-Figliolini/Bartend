pub(super) mod categories;
pub(super) mod inventory;
pub(super) mod recipes;
pub(super) mod serving;
pub(super) mod settings;

use iced::Element;

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::Config,
    presentation::{
        Updateable, Viewable,
        application::{Command, Message},
        screen::{recipes::Recipes, serving::Serving},
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
    pub fn view(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, Message> {
        match self {
            Self::Inventory(inventory) => inventory.view(item_service, category_service),
            Self::Settings(settings) => settings.view(),
            Self::Categories(categories) => categories.view(category_service),
            Self::Recipes(recipes) => recipes.view(category_service, recipe_service),
            Self::Serving(serving) => serving.view(item_service, category_service, recipe_service),
        }
    }
    pub fn update(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
        message: Message,
    ) -> Option<Command> {
        match (self, message) {
            (Self::Inventory(inventory), Message::Inventory(message)) => {
                inventory.update(item_service, message)
            }
            (Self::Settings(settings), Message::Settings(message)) => settings.update(message),
            (Self::Categories(categories), Message::Categories(message)) => {
                categories.update(category_service, message)
            }
            (Self::Recipes(recipes), Message::Recipes(message)) => {
                recipes.update(recipe_service, message)
            }
            (Self::Serving(service), Message::Serving(message)) => {
                service.update(item_service, category_service, message)
            }
            _ => unreachable!(),
        }
    }

    pub fn reload(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) {
        match self {
            Screen::Inventory(inventory) => {
                inventory.update(item_service, inventory::Message::Reload)
            }
            Screen::Settings(_) => unreachable!(),
            Screen::Categories(categories) => {
                categories.update(category_service, categories::Message::Reload)
            }
            Screen::Recipes(recipes) => recipes.update(recipe_service, recipes::Message::Reload),
            Screen::Serving(serving) => {
                serving.update(item_service, category_service, serving::Message::Reload)
            }
        };
    }

    pub fn inventory(
        config: &Config,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Self {
        let inventory = inventory::Inventory::new(config, item_service, category_service);
        Self::Inventory(Box::new(inventory))
    }

    pub fn settings(current_config: &Config) -> Self {
        Self::Settings(settings::Settings::new(current_config))
    }
    pub fn categories(config: &Config, category_service: &CategoryService) -> Self {
        Self::Categories(categories::Categories::new(config, category_service))
    }
    pub fn recipes(
        config: &Config,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Self {
        Screen::Recipes(Recipes::new(config, category_service, recipe_service))
    }
    pub fn serving(config: &Config, recipe_service: &RecipeService) -> Self {
        Screen::Serving(Serving::new(config, recipe_service))
    }
}
