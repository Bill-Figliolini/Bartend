pub mod category_input;
pub mod item_input;
pub mod recipe_input;

use iced::widget::Id;

use crate::{
    logic::{
        category::Category,
        quantity::{Unit, UnitSystem},
    },
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
    fn begin_edit(&mut self, edit: &T, unit_system: UnitSystem);
    fn clear(&mut self);
}
