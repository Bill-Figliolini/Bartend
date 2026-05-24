use std::{collections::HashSet, rc::Rc};

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
    errors: HashSet<Error>,
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
            errors: HashSet::new(),
        }
    }
}

impl<Message> InputContents<String> for StringInput<Message>
where
    Message: Clone,
{
    fn get_output(&mut self) -> Result<String, ()> {
        self.errors.clear();
        if self.inner.text.is_empty() {
            self.errors.insert(Error::StringEmpty);
            Err(())
        } else {
            Ok(self.inner.text.clone())
        }
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
                .errors
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
    }
    fn id(&self) -> &Id {
        self.inner.id()
    }
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}
