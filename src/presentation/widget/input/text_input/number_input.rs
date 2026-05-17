use std::rc::Rc;

use iced::widget::Id;

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
