use iced::Element;

use crate::{
    logic::config::Config,
    presentation::{application, screen::Composition},
};

#[derive(Debug)]
pub struct Recipes {}
#[derive(Debug, Clone)]
pub enum Message {}

impl Composition<Message> for Recipes {
    fn new(config: &Config) -> Self {
        todo!()
    }

    fn view(&self) -> Element<'_, application::Message> {
        todo!()
    }

    fn update(&mut self, message: Message) -> Option<application::Command> {
        todo!()
    }
}
