use std::{fmt::Display, mem::take};

use iced::Element;

use crate::logic::quantity::{Quantity, Unit};

pub fn _name_entry<'a, Message: Clone>(
    _value: &str,
    _message: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    todo!()
}

pub fn name_unload(name: &mut String) -> Result<String, Error> {
    if name.is_empty() {
        Err(Error::StringEmpty)
    } else {
        Ok(take(name))
    }
}

pub fn quantity_unload(value: &mut String, unit: &Unit) -> Result<Quantity, Error> {
    let unvalidated_quantity = value.trim().parse::<f32>();
    let quantity = match unvalidated_quantity {
        Ok(quantity) if quantity > 0.0 => quantity,
        _ => return Err(Error::QuantityInvalid),
    };
    Ok(Quantity::new(quantity, *unit))
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Error {
    StringEmpty,
    QuantityInvalid,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::StringEmpty => "String Must Not Be Empty".to_string(),
            Self::QuantityInvalid => todo!(),
        };
        write!(f, "{text}")
    }
}
