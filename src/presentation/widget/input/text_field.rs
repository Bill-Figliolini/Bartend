use std::marker::PhantomData;

use iced::{Element, widget::text_input};

use crate::presentation::widget::input::{Error, with_error};

/// How a [`TextField`]'s raw text is turned into a value.
pub trait Rule {
    type Output;
    fn apply(raw: &str) -> Result<Self::Output, Error>;
}

/// Text with surrounding whitespace removed; whitespace-only text counts as
/// empty and is rejected.
#[derive(Debug)]
pub struct NonEmpty;

impl Rule for NonEmpty {
    type Output = String;

    fn apply(raw: &str) -> Result<String, Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(Error::StringEmpty)
        } else {
            Ok(trimmed.to_string())
        }
    }
}

/// A finite number that is zero or above.
#[derive(Debug)]
pub struct NonNegative;

impl Rule for NonNegative {
    type Output = f32;

    /// `-0` passes the `>= 0.0` check, so `abs` normalises it to `0.0`.
    fn apply(raw: &str) -> Result<f32, Error> {
        raw.trim()
            .parse::<f32>()
            .ok()
            .filter(|quantity| quantity.is_finite() && *quantity >= 0.0)
            .map(f32::abs)
            .ok_or(Error::QuantityInvalid)
    }
}

/// A finite number above zero.
#[derive(Debug)]
pub struct Positive;

impl Rule for Positive {
    type Output = f32;

    fn apply(raw: &str) -> Result<f32, Error> {
        raw.trim()
            .parse::<f32>()
            .ok()
            .filter(|number| number.is_finite() && *number > 0.0)
            .ok_or(Error::NumberNotPositive)
    }
}

/// A text box whose contents are checked by the rule `R`.
///
/// The field holds only what the user typed. Validation is a pure read
/// ([`value`](Self::value)); whether its error is shown is decided by the form
/// that owns the field, when it calls [`view`](Self::view).
#[derive(Debug)]
pub struct TextField<R: Rule> {
    text: String,
    placeholder: String,
    rule: PhantomData<R>,
}

/// A text box that must not be blank.
pub type StringField = TextField<NonEmpty>;
/// A text box for a quantity.
pub type NumberField = TextField<NonNegative>;
/// A text box for a number that must be above zero, such as a count of
/// servings.
pub type PositiveField = TextField<Positive>;

impl<R: Rule> TextField<R> {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            placeholder: placeholder.into(),
            rule: PhantomData,
        }
    }

    /// Replaces the text, as typed or when filling the form for an edit.
    pub fn set(&mut self, text: String) {
        self.text = text;
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    /// The text as typed.
    #[cfg(test)]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text, checked by `R`. Reading it changes nothing.
    pub fn value(&self) -> Result<R::Output, Error> {
        R::apply(&self.text)
    }

    /// `on_input` builds the message for each edit. With `show_error`, the
    /// field's current error, if any, is shown under it.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view<'a, M: Clone + 'a>(
        &'a self,
        on_input: impl Fn(String) -> M + 'a,
        show_error: bool,
    ) -> Element<'a, M> {
        let input = text_input(&self.placeholder, &self.text).on_input(on_input);
        let error = if show_error { self.value().err() } else { None };
        with_error(input.into(), error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field<R: Rule>(text: &str) -> TextField<R> {
        let mut field = TextField::new("Name");
        field.set(text.to_string());
        field
    }

    /// Compares by bit pattern, which avoids `float_cmp` and tells `-0.0`
    /// apart from `0.0`.
    fn number_bits(text: &str) -> Result<u32, Error> {
        field::<NonNegative>(text).value().map(f32::to_bits)
    }

    #[test]
    fn empty_text_is_rejected() {
        assert_eq!(field::<NonEmpty>("").value(), Err(Error::StringEmpty));
    }

    #[test]
    fn whitespace_only_text_is_rejected() {
        assert_eq!(field::<NonEmpty>(" \t ").value(), Err(Error::StringEmpty));
    }

    #[test]
    fn text_is_trimmed() {
        assert_eq!(field::<NonEmpty>("  Gin ").value(), Ok("Gin".to_string()));
    }

    #[test]
    fn value_tracks_the_latest_text() {
        let mut field = field::<NonEmpty>("");
        assert_eq!(field.value(), Err(Error::StringEmpty));
        field.set("Gin".to_string());
        assert_eq!(field.value(), Ok("Gin".to_string()));
        // Reading the value does not change the field.
        assert_eq!(field.value(), Ok("Gin".to_string()));
    }

    #[test]
    fn clear_removes_text() {
        let mut field = field::<NonEmpty>("Gin");
        field.clear();
        assert_eq!(field.text, "");
        assert_eq!(field.value(), Err(Error::StringEmpty));
    }

    #[test]
    fn debug_shows_placeholder_and_text() {
        let shown = format!("{:?}", field::<NonEmpty>("Gin"));
        assert!(shown.contains("\"Name\""), "{shown}");
        assert!(shown.contains("\"Gin\""), "{shown}");
    }

    #[test]
    fn accepts_non_negative_numbers() {
        assert_eq!(number_bits("1.5"), Ok(1.5f32.to_bits()));
        assert_eq!(number_bits("0"), Ok(0f32.to_bits()));
        assert_eq!(number_bits("  2 "), Ok(2f32.to_bits()));
    }

    #[test]
    fn negative_zero_becomes_zero() {
        assert_eq!(number_bits("-0"), Ok(0f32.to_bits()));
    }

    #[test]
    fn positive_accepts_numbers_above_zero() {
        for (text, expected) in [("0.5", 0.5f32), ("1", 1.0), (" 1.5 ", 1.5)] {
            let value = field::<Positive>(text).value().map(f32::to_bits);
            assert_eq!(value, Ok(expected.to_bits()), "{text:?}");
        }
    }

    #[test]
    fn positive_rejects_zero_negatives_and_non_numbers() {
        for text in ["", "abc", "0", "-0", "-1", "NaN", "inf", "-inf"] {
            let value = field::<Positive>(text).value();
            assert_eq!(value, Err(Error::NumberNotPositive), "{text:?}");
        }
    }

    #[test]
    fn rejects_invalid_numbers() {
        for text in ["", "abc", "-1", "NaN", "inf", "-inf"] {
            assert_eq!(number_bits(text), Err(Error::QuantityInvalid), "{text:?}");
        }
    }
}
