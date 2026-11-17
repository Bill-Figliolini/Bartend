//! Form fields: [`StringField`] and [`NumberField`] for typed text, and
//! [`RequiredPick`] and [`OptionalPick`] for choosing from a list.
//!
//! Each field holds only what the user entered. Validation is a pure read
//! (`value`), and the form that owns the field decides when errors are shown
//! by passing `show_error` to `view`. The fields know nothing about the
//! app's messages: `view` takes the function that builds one. See
//! [`input_handling`](crate::presentation::input_handling) for the forms.

mod color_field;
mod keyboard_pick;
mod pick_field;
mod text_field;
use std::{convert::Infallible, fmt::Display};

use iced::{
    Element,
    widget::{Text, column, row, text},
};

use crate::presentation::constants;

pub use {
    pick_field::{OptionalPick, RequiredPick},
    text_field::{NumberField, OptionalPositiveField, PositiveField, StringField},
};

/// Why a single input field failed validation.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    StringEmpty,
    QuantityInvalid,
    NumberNotPositive,
    MustChooseValue,
    /// The amounts poured from several items do not add up to what the
    /// ingredient calls for.
    PoursDontAddUp,
    /// The chosen item has less in stock than is being taken from it.
    NotEnoughStock,
    /// A hex code that isn't six hex digits.
    ColorInvalid,
}

impl From<Infallible> for Error {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

/// A field with its error (if any) shown underneath.
///
/// The error sits in a row that is always present, even when empty, so the
/// widget tree keeps the same shape as errors come and go; otherwise iced
/// would rebuild the field's state and the text box would lose focus while
/// the user is typing.
#[cfg_attr(coverage_nightly, coverage(off))]
fn with_error<'a, M: 'a>(field: Element<'a, M>, error: Option<Error>) -> Element<'a, M> {
    column![field, row(error.map(|error| error_text(error).into()))]
        .spacing(2)
        .into()
}

/// `error` as shown under a field: small, in the theme's danger color.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn error_text<'a>(error: Error) -> Text<'a> {
    text(error.to_string())
        .size(constants::SMALL_TEXT)
        .style(text::danger)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::StringEmpty => "Can't be blank",
            Self::QuantityInvalid => "Enter a number, 0 or more",
            Self::NumberNotPositive => "Enter a number above 0",
            Self::MustChooseValue => "Choose an option",
            Self::PoursDontAddUp => "Amounts must add up to what the recipe calls for",
            Self::NotEnoughStock => "Not enough in stock",
            Self::ColorInvalid => "Enter six hex digits, like #D8C08A",
        };
        write!(f, "{text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_explain_what_is_expected() {
        assert_eq!(Error::StringEmpty.to_string(), "Can't be blank");
        assert_eq!(
            Error::QuantityInvalid.to_string(),
            "Enter a number, 0 or more"
        );
        assert_eq!(Error::MustChooseValue.to_string(), "Choose an option");
        assert_eq!(
            Error::NumberNotPositive.to_string(),
            "Enter a number above 0"
        );
        assert_eq!(
            Error::PoursDontAddUp.to_string(),
            "Amounts must add up to what the recipe calls for"
        );
        assert_eq!(Error::NotEnoughStock.to_string(), "Not enough in stock");
        assert_eq!(
            Error::ColorInvalid.to_string(),
            "Enter six hex digits, like #D8C08A"
        );
    }
}
