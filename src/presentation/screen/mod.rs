pub(super) mod categories;
pub(super) mod inventory;
pub(super) mod recipes;
pub(super) mod serving;
pub(super) mod settings;

use iced::{Element, Task};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::Config,
    presentation::{
        application::{self, Context, Message},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    Inventory,
    Settings,
    Categories,
    Recipes,
    Serving,
}

//Unifies each screen's own Command type so `Screen::update` has a single return type.
//The logic for each variant lives with the screen that produces it; this only delegates.
pub enum ScreenCommand {
    Inventory(inventory::Command),
    Settings(settings::Command),
    Categories(categories::Command),
    Recipes(recipes::Command),
    Serving(serving::Command),
}
impl ScreenCommand {
    pub fn apply(self, ctx: &mut Context) -> Task<application::Message> {
        match self {
            ScreenCommand::Inventory(cmd) => cmd.apply(ctx),
            ScreenCommand::Settings(cmd) => cmd.apply(ctx),
            ScreenCommand::Categories(cmd) => cmd.apply(ctx),
            ScreenCommand::Recipes(cmd) => cmd.apply(ctx),
            ScreenCommand::Serving(cmd) => cmd.apply(ctx),
        }
    }
}

impl Screen {
    pub fn start(config: &Config, categories: &CategoryService) -> Self {
        let inventory = inventory::Inventory::new(config, categories);
        Self::Inventory(Box::new(inventory))
    }

    pub fn kind(&self) -> ScreenKind {
        match self {
            Self::Inventory(_) => ScreenKind::Inventory,
            Self::Settings(_) => ScreenKind::Settings,
            Self::Categories(_) => ScreenKind::Categories,
            Self::Recipes(_) => ScreenKind::Recipes,
            Self::Serving(_) => ScreenKind::Serving,
        }
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
            Self::Serving(serving) => serving.view(category_service),
        }
    }

    pub fn update(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        _recipe_service: &RecipeService,
        message: Message,
    ) -> Option<ScreenCommand> {
        match (self, message) {
            (Self::Inventory(inventory), Message::Inventory(message)) => inventory
                .update(item_service, message)
                .map(ScreenCommand::Inventory),
            (Self::Settings(settings), Message::Settings(message)) => {
                settings.update(message).map(ScreenCommand::Settings)
            }
            (Self::Categories(categories), Message::Categories(message)) => {
                categories.update(message).map(ScreenCommand::Categories)
            }
            (Self::Recipes(recipes), Message::Recipes(message)) => {
                recipes.update(message).map(ScreenCommand::Recipes)
            }
            (Self::Serving(service), Message::Serving(message)) => service
                .update(item_service, category_service, message)
                .map(ScreenCommand::Serving),
            //A message meant for a screen that is no longer active (e.g. an async
            //task, like the DB file picker, resolving after the user navigated away).
            _ => None,
        }
    }

    //The message that refreshes the active screen after a Command has been applied.
    pub fn reload_message(&self, config: &Config) -> Message {
        match self {
            Screen::Inventory(_) => Message::Inventory(inventory::Message::Reload),
            Screen::Settings(_) => {
                Message::Settings(settings::Message::ResetConfig(config.clone()))
            }
            Screen::Categories(_) => Message::Categories(categories::Message::Reload),
            Screen::Recipes(_) => Message::Recipes(recipes::Message::Reload),
            Screen::Serving(_) => Message::Serving(serving::Message::Reload),
        }
    }

    pub fn inventory(config: &Config, category_service: &CategoryService) -> Self {
        let inventory = inventory::Inventory::new(config, category_service);
        Self::Inventory(Box::new(inventory))
    }

    pub fn settings(current_config: &Config) -> Self {
        Self::Settings(settings::Settings::new(current_config))
    }
    pub fn categories(config: &Config) -> Self {
        Self::Categories(categories::Categories::new(config))
    }
    pub fn recipes(config: &Config, category_service: &CategoryService) -> Self {
        Screen::Recipes(Recipes::new(config, category_service))
    }
    pub fn serving(config: &Config, recipe_service: &RecipeService) -> Self {
        Screen::Serving(Serving::new(config, recipe_service))
    }
}
