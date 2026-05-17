use std::rc::Rc;

use iced::widget::Id;

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
        }
    }
}

impl<Message> InputContents<String> for StringInput<Message>
where
    Message: Clone,
{
    fn get_output(&self) -> Result<String, Error> {
        if self.inner.text.is_empty() {
            Err(Error::StringEmpty)
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
        self.inner.view()
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
}
