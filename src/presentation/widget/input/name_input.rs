use iced::widget::text_input;

use super::Error;
use crate::presentation::{application::Message, widget::input::Input};

pub struct NameInput {
    id: String,
    name: String,
}
impl<'a> Input<'a, String> for NameInput {
    fn new(id: String) -> Self {
        Self {
            id,
            name: String::new(),
        }
    }

    fn display(&self, to_message: impl Fn(String) -> Message + 'a) -> iced::Element<'a, Message> {
        text_input("Name", &self.name)
            .id(self.id.clone())
            .on_input(to_message)
            .into()
    }
    fn update(&mut self, input: String) {
        self.name = input
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
