use std::fmt::Debug;

use iced::widget::{Id, text_input};

use super::Error;
use crate::presentation::{
    Viewable,
    widget::input::{Input, InputContents},
};
struct TextInput<Message>
where
    Message: Clone,
{
    id: Id,
    text: String,
    placeholder: String,
    message: fn(Id, String) -> Message,
}
#[derive(Debug)]
pub struct StringInput<Message>
where
    Message: Clone,
{
    inner: TextInput<Message>,
}
#[derive(Debug)]
pub struct NumberInput<Message>
where
    Message: Clone,
{
    inner: TextInput<Message>,
}
impl<Message> TextInput<Message>
where
    Message: Clone,
{
    pub fn new(msg: fn(Id, String) -> Message, placeholder: String, initial_value: String) -> Self {
        Self {
            id: Id::unique(),
            text: initial_value,
            message: msg,
            placeholder,
        }
    }
}
impl<Message> Viewable<Message> for TextInput<Message>
where
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        let message = self.message.clone();
        text_input(&self.placeholder, &self.text)
            .id(self.id.clone())
            .on_input(move |str: String| message(self.id().clone(), str))
            .into()
    }
}

impl<Message> Input<String, Message> for TextInput<Message>
where
    Message: Clone,
{
    fn update(&mut self, input: String) {
        self.text = input;
    }

    fn clear(&mut self) {
        self.text.clear();
    }
    fn id(&self) -> &Id {
        &self.id
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

impl<Message> InputContents<f32> for NumberInput<Message>
where
    Message: Clone,
{
    fn get_output(&self) -> Result<f32, Error> {
        let unvalidated_quantity = self.inner.text.trim().parse::<f32>();
        match unvalidated_quantity {
            Ok(quantity) => Ok(quantity),
            _ => return Err(Error::QuantityInvalid),
        }
    }
}
impl<Message> Viewable<Message> for NumberInput<Message>
where
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        self.inner.view()
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
    }
    fn id(&self) -> &Id {
        self.inner.id()
    }
}

impl<Message> Debug for TextInput<Message>
where
    Message: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NameInput")
            .field("id", &self.id)
            .field("placeholder", &self.placeholder)
            .field("text", &self.text)
            .finish()
    }
}
