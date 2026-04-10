use iced::{Element, widget::column};

use crate::{
    common::config::Config,
    presentation::{
        application,
        screen::Viewable,
        widget::{self, text_style},
    },
};

#[derive(Debug)]
pub struct Categories {
    input_name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateName(String),
    SaveCategory,
}

impl Viewable<Message> for Categories {
    fn new(_config: &Config) -> Self {
        Self {
            input_name: String::new(),
        }
    }

    fn view(&self) -> Element<'_, application::Message> {
        let header = widget::header::header(text_style::title("Categories"));

        column![header].into()
    }

    fn update(&mut self, message: Message) -> Option<crate::presentation::application::Command> {
        todo!()
    }
}
