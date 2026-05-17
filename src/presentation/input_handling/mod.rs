pub mod category_input;
pub mod item_input;
pub mod recipe_input;

use iced::widget::Id;

use crate::{
    logic::{Editable, category::Category, config::Config, quantity::Unit},
    presentation::{Viewable, application::Message},
};
#[derive(Clone, Debug)]
pub enum InputMessage {
    String(Id, String),
    Unit(Id, Unit),
    Category(Id, Category),
    OptionalCategory(Id, Option<Category>),
}

pub trait InputCollection<T>: Viewable<Message>
where
    T: Editable,
{
    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self;
    fn update(&mut self, msg: InputMessage);
    fn output(&mut self) -> Result<T, ()>;
}
