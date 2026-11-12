use std::rc::Rc;

use iced::widget::{Id, column, row, text};

use crate::presentation::{
    Viewable,
    widget::input::{Error, Input, InputContents, text_input::TextInput},
};

#[derive(Debug)]
pub struct NumberInput<Message>
where
    Message: Clone,
{
    inner: TextInput<Message>,
    error: Option<Error>,
}
impl<Message> NumberInput<Message>
where
    Message: Clone,
{
    pub fn new<F: Fn(Id, String) -> Message + 'static>(
        msg: F,
        placeholder: String,
        initial_value: String,
    ) -> Self {
        Self {
            inner: TextInput {
                id: Id::unique(),
                text: initial_value,
                message: Rc::new(msg),
                placeholder,
            },
            error: None,
        }
    }
}
impl<Message> InputContents<f32> for NumberInput<Message>
where
    Message: Clone,
{
    /// Accepts finite numbers that are zero or above. `-0` passes the `>= 0.0`
    /// check, so `abs` normalises it to `0.0`.
    fn get_output(&mut self) -> Result<f32, Error> {
        let result = self
            .inner
            .text
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|quantity| quantity.is_finite() && *quantity >= 0.0)
            .map(f32::abs)
            .ok_or(Error::QuantityInvalid);
        self.error = result.err();
        result
    }
}
impl<Message> Viewable<Message> for NumberInput<Message>
where
    Message: Clone,
{
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view(&self) -> iced::Element<'_, Message> {
        column![
            self.inner.view(),
            row(self
                .error
                .iter()
                .map(|error| text(error.to_string()).into()))
        ]
        .into()
    }
}
impl<Message> Input<String, Message> for NumberInput<Message>
where
    Message: Clone,
{
    fn update(&mut self, input: String) {
        self.inner.update(input);
    }

    fn clear(&mut self) {
        self.inner.clear();
        self.error = None;
    }
    fn id(&self) -> &Id {
        self.inner.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(text: &str) -> NumberInput<()> {
        NumberInput::new(|_, _| (), "Quantity".to_string(), text.to_string())
    }

    /// Compares by bit pattern, which avoids `float_cmp` and tells `-0.0`
    /// apart from `0.0`.
    fn output_bits(text: &str) -> Result<u32, Error> {
        input(text).get_output().map(f32::to_bits)
    }

    #[test]
    fn accepts_non_negative_numbers() {
        assert_eq!(output_bits("1.5"), Ok(1.5f32.to_bits()));
        assert_eq!(output_bits("0"), Ok(0f32.to_bits()));
        assert_eq!(output_bits("  2 "), Ok(2f32.to_bits()));
    }

    #[test]
    fn negative_zero_becomes_zero() {
        assert_eq!(output_bits("-0"), Ok(0f32.to_bits()));
    }

    #[test]
    fn rejects_invalid_numbers() {
        for text in ["", "abc", "-1", "NaN", "inf", "-inf"] {
            assert_eq!(output_bits(text), Err(Error::QuantityInvalid), "{text:?}");
        }
    }

    #[test]
    fn records_and_clears_error() {
        let mut input = input("abc");
        _ = input.get_output();
        assert_eq!(input.error, Some(Error::QuantityInvalid));
        input.clear();
        assert_eq!(input.error, None);
    }
}
