use iced::{Element, widget::column};

use crate::{
    logic::{config::Config, quantity::UnitSystem},
    presentation::{
        application,
        screen::Composition,
        widget::{header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Recipes {
    unit_system: UnitSystem,
}
#[derive(Debug, Clone)]
pub enum Message {}

impl Composition<Message> for Recipes {
    fn new(config: &Config) -> Self {
        let unit_system = config.default_units();
        Self { unit_system }
    }

    fn view(&self) -> Element<'_, application::Message> {
        let header = header(title("Recipes"));
        column![header].into()
    }

    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {}
    }
}
