use std::{fmt::Debug, rc::Rc};

use iced::widget::text_input;

use super::Error;
use crate::presentation::{
    application::Message,
    widget::input::{Input, InputString},
};
pub struct NameInput {
    id: String,
    name: String,
    on_input: Rc<dyn Fn(String) -> Message + 'static>,
}
impl<'a> Input<'a, String, String> for NameInput {
    fn new<F: Fn(String) -> Message + 'static>(id: &str, on_input: F) -> Self {
        Self {
            id: id.to_string(),
            name: String::new(),
            on_input: Rc::new(on_input),
        }
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

impl<'a> InputString<'a> for NameInput {
    fn display(&self) -> iced::Element<'a, Message> {
        let on_input = self.on_input.clone();
        text_input("Name", &self.name)
            .id(self.id.clone())
            .on_input(move |s| on_input(s))
            .into()
    }
    fn update(&mut self, input: String) {
        self.name = input
    }
}

impl Debug for NameInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NameInput")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}
