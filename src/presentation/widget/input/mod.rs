pub mod name_input;
pub mod quantity_input;
use std::fmt::Display;

use crate::{
    logic::quantity::{Quantity, Unit},
    presentation::application::Message,
};
use iced::Element;

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

pub trait Input<'a, InputType, OutputType>
where
    InputType: Display,
{
    fn new<F: Fn(InputType) -> Message + 'static>(id: &str, on_input: F) -> Self;
    fn get_output(&self) -> Result<OutputType, Error>;
    fn clear(&mut self);
}

pub trait InputString<'a> {
    fn display(&self) -> Element<'a, Message>;
    fn update(&mut self, input: String);
}

pub trait InputPick<'a, T>
where
    T: Display,
{
    fn display(&self) -> Element<'a, Message>;
    fn update(&mut self, input: Option<T>);
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

//Composition based approach. See notebook pages 61-62
// After more thought, probably a bad idea. Would not scale well with more functions, and would require more heap allocations.
struct CompositionInput<InputType> {
    input: InputType,
}
