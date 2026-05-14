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
    MustChooseValue,
}

pub trait Input<InternalType, OutputType, Message>: Viewable<Message>
where
    InternalType: Clone,
    Message: Clone,
{
    fn update(&mut self, input: InternalType);
    fn clear(&mut self) {}
    fn id(&self) -> &Id;
}
pub trait InputContents<T> {
    fn get_output(&self) -> Result<T, Error>;
}

pub trait InputOptionalContents<T> {
    fn get_output(&self) -> Result<Option<T>, Error>;
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::StringEmpty => "String Must Not Be Empty".to_string(),
            Self::QuantityInvalid => "Must be a non-zero positive number".to_string(),
            Self::MustChooseValue => "A value must be selected".to_string(),
        };
        write!(f, "{text}")
    }
}
