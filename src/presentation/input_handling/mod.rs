mod category_input;
mod item_input;
mod recipe_input;
mod service_input;

use iced::widget::Id;

use crate::{
    models::{Category, Unit, UnitSystem},
    presentation::{Viewable, application::Message},
};
#[derive(Clone, Debug)]
pub enum InputMessage {
    String(Id, String),
    Unit(Id, Unit),
    Category(Id, Category),
}

impl InputMessage {
    fn id(&self) -> Id {
        match self {
            InputMessage::String(id, _) => id.clone(),
            InputMessage::Unit(id, _) => id.clone(),
            InputMessage::Category(id, _) => id.clone(),
        }
    }
}

pub trait InputCollection<T>: Viewable<Message> {
    fn update(&mut self, msg: InputMessage);
    fn output(&mut self) -> Result<T, ()>;
    fn clear(&mut self);
}

pub trait EditableCollection<T>: InputCollection<T> {
    fn begin_edit(&mut self, edit: &T, unit_system: UnitSystem);
}

pub use {category_input::CategoryInput, item_input::ItemInput, recipe_input::RecipeInput};
