pub mod categories;
pub mod inventory;
pub mod recipes;
pub mod settings;

use iced::Element;

use crate::{
    logic::{
        category::Category,
        config::{self, Config},
        item::Item,
        quantity::Unit,
    },
    presentation::{
        Updateable, Viewable,
        application::{Command, Message},
        screen::recipes::Recipes,
        widget::input::{
            pick_input::{OptionalPickInput, RequiredPickInput},
            string_input::{StringInput, NumberInput},
        },
    },
};
#[derive(Debug)]
struct ItemInput {
    name_input: StringInput,
    quantity_input: NumberInput,
    unit_input: RequiredPickInput<Unit, Message>,
    category_input: OptionalPickInput<Category, Message>,
}
#[derive(Debug)]
struct IngredientInput {
    category_input: RequiredPickInput<Category, Message>,
    quantity_input: NumberInput,
    unit_input: RequiredPickInput<Unit, Message>,
}
#[derive(Debug)]
struct

#[derive(Debug)]
pub enum Screen {
    Inventory(inventory::Inventory),
    Settings(settings::Settings),
    Categories(categories::Categories),
    Recipes(recipes::Recipes),
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
    pub fn inventory(config: &Config) -> Self {
        let inventory = inventory::Inventory::new(config);
        Self::Inventory(inventory)
    }

    pub fn settings(current_config: &Config) -> Self {
        Self::Settings(settings::Settings::new(current_config))
    }
    pub fn categories(config: &Config) -> Self {
        Self::Categories(categories::Categories::new(config))
    }
    pub fn recipes(config: &Config) -> Self {
        Screen::Recipes(Recipes::new(config))
    }
}

impl ItemInput {
    pub fn new(config: &Config) -> Self {
        todo!()
    }
}
impl IngredientInput {
    pub fn new(config: &Config) -> Self {
        todo!()
    }
}
