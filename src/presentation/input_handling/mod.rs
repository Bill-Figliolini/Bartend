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
            InputMessage::String(id, _)
            | InputMessage::Unit(id, _)
            | InputMessage::Category(id, _)
            | InputMessage::Recipe(id, _)
            | InputMessage::Item(id, _) => id.clone(),
        }
    }
}

pub use {
    category_input::CategoryInput, item_input::ItemInput, recipe_input::RecipeInput,
    serving_input::ServingInput,
};
