use std::{fmt::Display, rc::Rc};

use crate::presentation::{
    application::Message,
    widget::input::{Input, InputPick, InputString},
};

use super::Error;

pub struct NumberInput {
    id: String,
    input_number: String,
    on_input: Rc<dyn Fn(String) -> Message + 'static>,
}
pub struct PickInput<T> {
    id: String,
    input: Option<T>,
    on_input: Rc<dyn Fn(T) -> Message + 'static>,
}

impl<'a> Input<'a, String, f32> for NumberInput {
    fn new<F: Fn(String) -> Message + 'static>(id: &str, on_input: F) -> Self {
        let on_input = Rc::new(on_input);
        Self {
            id: id.to_string(),
            input_number: String::new(),
            on_input,
        }
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

impl<'a> InputString<'a> for NumberInput {
    fn display(&self) -> iced::Element<'a, Message> {
        todo!()
    }
    fn update(&mut self, input: String) {
        self.input_number = input;
    }
}

impl<'a, T> Input<'a, T, T> for PickInput<T>
where
    T: Display,
{
    fn new<F: Fn(T) -> Message + 'static>(id: &str, on_input: F) -> Self {
        let on_input = Rc::new(on_input);
        Self {
            id: id.to_string(),
            input: None,
            on_input,
        }
    }

    fn get_output(&self) -> Result<T, Error> {
        todo!()
    }

    fn clear(&mut self) {
        self.input = None;
    }
}

impl<'a, T> InputPick<'a, T> for PickInput<T>
where
    T: Display,
{
    fn display(&self) -> iced::Element<'a, Message> {
        todo!()
    }
    fn update(&mut self, input: Option<T>) {
        self.input = input;
    }
}
