use iced::{Element, widget::column};

use crate::{
    common::{config::Config, quantity::UnitSystem},
    presentation::{
        application::{self, Command},
        widget::text_style::title,
    },
};

#[derive(Debug)]
pub struct Settings {
    input_db_path: String,
    default_unit_system: UnitSystem,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateDBPath(String),
    UpdateUnitSystem(UnitSystem),
}

impl Settings {
    pub(super) fn new(current_config: Config) -> Self {
        let input_db_path = current_config.db_path().to_str().unwrap().to_string();
        let default_unit_system = current_config.default_units();
        Self {
            input_db_path,
            default_unit_system,
        }
    }
    pub(super) fn view(&self) -> Element<'_, application::Message> {
        let title = title("Settings");

        column![title].into()
    }
    pub(super) fn update(&mut self, _message: Message) -> Option<Command> {
        None
    }
}
