use std::rc::Rc;

use iced::widget::{Id, column, row, text};

use crate::presentation::{
    Viewable,
    widget::input::{Error, Input, InputContents, text_input::TextInput},
};

#[derive(Debug)]
pub struct StringInput<Message>
where
    Message: Clone,
{
    inner: TextInput<Message>,
    error: Option<Error>,
}
impl<Message> StringInput<Message>
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

impl<Message> InputContents<String> for StringInput<Message>
where
    Message: Clone,
{
    /// Returns the text with surrounding whitespace removed; whitespace-only
    /// input counts as empty.
    fn get_output(&mut self) -> Result<String, Error> {
        let trimmed = self.inner.text.trim();
        let result = if trimmed.is_empty() {
            Err(Error::StringEmpty)
        } else {
            Ok(trimmed.to_string())
        };
        self.error = result.as_ref().err().copied();
        result
    }
}
impl<Message> Viewable<Message> for StringInput<Message>
where
    Message: Clone,
{
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
impl<Message> Input<String, Message> for StringInput<Message>
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

    fn input(text: &str) -> StringInput<()> {
        StringInput::new(|_, _| (), "Name".to_string(), text.to_string())
    }

    #[test]
    fn empty_text_is_rejected_and_shown() {
        let mut input = input("");
        assert_eq!(input.get_output(), Err(Error::StringEmpty));
        assert_eq!(input.error, Some(Error::StringEmpty));
    }

    #[test]
    fn whitespace_only_text_is_rejected() {
        assert_eq!(input(" \t ").get_output(), Err(Error::StringEmpty));
    }

    #[test]
    fn output_is_trimmed() {
        assert_eq!(input("  Gin ").get_output(), Ok("Gin".to_string()));
    }

    #[test]
    fn valid_output_clears_previous_error() {
        let mut input = input("");
        _ = input.get_output();
        input.update("Gin".to_string());
        assert_eq!(input.get_output(), Ok("Gin".to_string()));
        assert_eq!(input.error, None);
    }

    #[test]
    fn clear_removes_text_and_error() {
        let mut input = input("");
        _ = input.get_output();
        input.update("Gin".to_string());
        input.clear();
        assert_eq!(input.error, None);
        assert_eq!(input.inner.text, "");
    }
}
