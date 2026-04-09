use iced::Element;

use crate::{
    common::config::Config,
    presentation::{application, screen::Viewable},
};

#[derive(Debug)]
pub struct Categories {}

#[derive(Debug, Clone)]
pub enum Message {}

impl Viewable<Message> for Categories {
    fn new(config: &Config) -> Self {
        todo!()
    }

    fn view(&self) -> Element<'_, application::Message> {
        todo!()
    }

    fn update(&mut self, message: Message) -> Option<crate::presentation::application::Command> {
        todo!()
    }
}
