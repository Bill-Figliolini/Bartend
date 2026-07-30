mod category_input;
mod item_input;
mod recipe_input;
mod serving_input;

use iced::widget::Id;

use crate::models::{Category, Item, Recipe, Unit};

#[derive(Clone, Debug)]
pub enum InputMessage {
    String(Id, String),
    Unit(Id, Unit),
    Category(Id, Category),
    Recipe(Id, Recipe),
    Item(Id, Item),
}

impl InputMessage {
    fn id(&self) -> Id {
        match self {
            InputMessage::String(id, _) => id.clone(),
            InputMessage::Unit(id, _) => id.clone(),
            InputMessage::Category(id, _) => id.clone(),
            InputMessage::Recipe(id, _) => id.clone(),
            InputMessage::Item(id, _) => id.clone(),
        }
    }
}

pub use {
    category_input::{CategoryInput, CategoryRelationInput},
    item_input::ItemInput,
    recipe_input::RecipeInput,
    serving_input::ServingInput,
};
