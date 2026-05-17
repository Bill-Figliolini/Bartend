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
    OptionalCategory(Id, Option<Category>),
}

pub trait InputCollection<T>: Viewable<Message> {
    fn update(&mut self, msg: InputMessage);
    fn output(&mut self) -> Result<T, ()>;
    fn begin_edit(&mut self, edit: T, unit_system: UnitSystem);
}
