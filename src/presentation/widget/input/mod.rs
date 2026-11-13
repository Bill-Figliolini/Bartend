//! Form fields: [`StringField`] and [`NumberField`] for typed text, and
//! [`RequiredPick`] and [`OptionalPick`] for choosing from a list.
//!
//! Each field holds only what the user entered. Validation is a pure read
//! (`value`), and the form that owns the field decides when errors are shown
//! by passing `show_error` to `view`. The fields know nothing about the
//! app's messages: `view` takes the function that builds one. See
//! [`input_handling`](crate::presentation::input_handling) for the forms.

mod pick_field;
mod text_field;
use std::{convert::Infallible, fmt::Display};

use iced::{
    Element,
    widget::{column, row, text},
};

pub use {
    pick_field::{OptionalPick, RequiredPick},
    text_field::{NumberField, StringField},
};

/// Why a single input field failed validation.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    StringEmpty,
    QuantityInvalid,
    MustChooseValue,
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
    column![
        field,
        row(error.map(|error| text(error.to_string()).into()))
    ]
    .into()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_explain_what_is_expected() {
        assert_eq!(Error::StringEmpty.to_string(), "String Must Not Be Empty");
        assert_eq!(
            Error::QuantityInvalid.to_string(),
            "Must be a number that is zero or above"
        );
        assert_eq!(
            Error::MustChooseValue.to_string(),
            "A value must be selected"
        );
    }
}
