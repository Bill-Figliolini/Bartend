pub mod pick_input;
pub mod string_input;
use std::fmt::Display;

use iced::widget::Id;

use crate::{
    logic::quantity::{Quantity, Unit},
    presentation::Viewable,
};

pub fn quantity_unload(value: &String, unit: &Unit) -> Result<Quantity, Error> {
    let unvalidated_quantity = value.trim().parse::<f32>();
    let quantity = match unvalidated_quantity {
        Ok(quantity) if quantity > 0.0 => quantity,
        _ => return Err(Error::QuantityInvalid),
    };
    Ok(Quantity::new(quantity, *unit))
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Error {
    StringEmpty,
    QuantityInvalid,
}

pub trait Input<InputType, OutputType, Message>: Viewable<Message>
where
    InputType: Display + Clone,
    Message: Clone,
{
    fn new<F: Fn(Id, InputType) -> Message + 'static>(msg: F) -> Self;
    fn update(&mut self, input: InputType);
    fn get_output(&self) -> Result<OutputType, Error>;
    fn clear(&mut self) {}
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::StringEmpty => "String Must Not Be Empty".to_string(),
            Self::QuantityInvalid => "Quantity must be a non-zero positive number".to_string(),
        };
        write!(f, "{text}")
    }
}
