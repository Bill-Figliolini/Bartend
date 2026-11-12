mod pick_input;
mod text_input;
use std::fmt::Display;

use iced::widget::Id;

pub use {
    pick_input::optional::OptionalPickInput, pick_input::required::RequiredPickInput,
    text_input::number_input::NumberInput, text_input::string_input::StringInput,
};

use crate::presentation::Viewable;

/// Why a single input field failed validation.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
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
}

/// An input whose contents can fail validation.
///
/// `get_output` validates the current contents, records the resulting error
/// (if any) on the widget so it is shown inline, and returns it.
pub trait InputContents<T> {
    fn get_output(&mut self) -> Result<T, Error>;
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::StringEmpty => "String Must Not Be Empty",
            Self::QuantityInvalid => "Must be a number that is zero or above",
            Self::MustChooseValue => "A value must be selected",
        };
        write!(f, "{text}")
    }
}
