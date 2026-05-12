use std::{fmt::Debug, rc::Rc};

use iced::widget::{Id, text_input};

use super::Error;
use crate::presentation::{Viewable, application::Message, widget::input::Input};
pub struct NameInput {
    id: Id,
    name: String,
    message: Rc<dyn Fn(Id, String) -> Message>,
}
pub struct NumberInput {
    id: Id,
    input_number: String,
    message: Rc<dyn Fn(Id, String) -> Message>,
}

impl Viewable<Message> for NameInput {
    fn view(&self) -> iced::Element<'_, Message> {
        let message = self.message.clone();
        text_input("Quantity", &self.name)
            .id(self.id.clone())
            .on_input(move |str: String| message(self.id.clone(), str))
            .into()
    }
}

impl Input<String, String, Message> for NameInput {
    fn new<F: Fn(Id, String) -> Message + 'static>(msg: F) -> Self {
        Self {
            id: Id::unique(),
            name: String::new(),
            message: Rc::new(msg),
        }
    }

    fn update(&mut self, input: String) {
        self.name = input;
    }
    fn get_output(&self) -> Result<String, Error> {
        if self.name.is_empty() {
            Err(Error::StringEmpty)
        } else {
            Ok(self.name.clone())
        }
    }
    fn clear(&mut self) {
        self.name.clear();
    }
}

impl Viewable<Message> for NumberInput {
    fn view(&self) -> iced::Element<'_, Message> {
        let message = self.message.clone();
        text_input("Quantity", &self.input_number)
            .id(self.id.clone())
            .on_input(move |str: String| message(self.id.clone(), str))
            .into()
    }
}
impl Input<String, f32, Message> for NumberInput {
    fn new<F: Fn(Id, String) -> Message + 'static>(msg: F) -> Self {
        Self {
            id: Id::unique(),
            input_number: String::new(),
            message: Rc::new(msg),
        }
    }
    fn update(&mut self, input: String) {
        self.input_number = input;
    }
    fn get_output(&self) -> Result<f32, Error> {
        let unvalidated_quantity = self.input_number.trim().parse::<f32>();
        match unvalidated_quantity {
            Ok(quantity) => Ok(quantity),
            _ => return Err(Error::QuantityInvalid),
        }
    }

    fn clear(&mut self) {
        self.input_number.clear();
    }
}

impl Debug for NameInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NameInput")
            .field("name", &self.name)
            .finish()
    }
}
