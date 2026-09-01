use std::{collections::HashSet, rc::Rc};

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
    errors: HashSet<Error>,
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
            errors: HashSet::new(),
        }
    }
}
impl<Message> InputContents<f32> for NumberInput<Message>
where
    Message: Clone,
{
    fn get_output(&mut self) -> Result<f32, ()> {
        self.errors.clear();
        let unvalidated_quantity = self.inner.text.trim().parse::<f32>();
        if let Ok(quantity) = unvalidated_quantity {
            if quantity.is_finite() && quantity >= 0.0 {
                Ok(quantity)
            } else {
                self.errors.insert(Error::QuantityInvalid);
                Err(())
            }
        } else {
            self.errors.insert(Error::QuantityInvalid);
            Err(())
        }
    }
}
impl<Message> Viewable<Message> for NumberInput<Message>
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
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}
