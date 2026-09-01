mod pick_input;
mod text_input;
use std::fmt::Display;

use iced::widget::Id;

pub use {
    pick_input::optional::OptionalPickInput, pick_input::required::RequiredPickInput,
    text_input::number_input::NumberInput, text_input::string_input::StringInput,
};

use crate::presentation::Viewable;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Error {
    StringEmpty,
    QuantityInvalid,
    MustChooseValue,
}

pub trait Input<InternalType, Message>: Viewable<Message>
where
    InternalType: Clone,
    Message: Clone,
{
    fn update(&mut self, input: InternalType);
    fn clear(&mut self) {}
    fn id(&self) -> &Id;
    fn has_error(&self) -> bool {
        false
    }
}
pub trait InputContents<T> {
    fn get_output(&mut self) -> Result<T, ()>;
}

pub trait InputOptionalContents<T> {
    fn get_output(&mut self) -> Result<Option<T>, ()>;
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::StringEmpty => "String Must Not Be Empty".to_string(),
            Self::QuantityInvalid => "Must be a number that is zero or above".to_string(),
            Self::MustChooseValue => "A value must be selected".to_string(),
        };
        write!(f, "{text}")
    }
}
