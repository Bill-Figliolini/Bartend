use std::path::PathBuf;

use iced::{
    Element,
    Length::Fill,
    widget::{column, row},
};

use crate::{
    common::{config::Config, quantity::UnitSystem},
    presentation::{
        application::{self, Command},
        constants,
        widget::{footer::footer, header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Settings {
    config: Config,
    input_db_path: PathBuf,
    input_unit_system: UnitSystem,
}

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    UpdateDBPath(PathBuf),
    UpdateUnitSystem(UnitSystem),
}

impl Settings {
    pub(super) fn new(current_config: &Config) -> Self {
        let config = current_config.clone();
        let input_db_path = current_config.db_path().clone();
        let default_unit_system = current_config.default_units();
        Self {
            config,
            input_db_path,
            input_unit_system: default_unit_system,
        }
    }
    pub(super) fn view(&self) -> Element<'_, application::Message> {
        let text_boundary = 500;

        let title_text = title("Settings");
        let header = header(title_text);

        let current_db_path = iced::widget::text!(
            "Current DB Path: {}",
            self.input_db_path.display().to_string()
        )
        .width(text_boundary);
        let db_button = iced::widget::Button::new("Choose DB File").on_press(
            application::Message::OpenDBPicker(self.input_db_path.clone()),
        );
        let db_row = row![current_db_path, db_button];

        let divider = iced::widget::rule::horizontal(constants::DIV_SIZE);

        let unit_text = iced::widget::text("Default units:").width(text_boundary);
        let unit_systems = vec![UnitSystem::Metric, UnitSystem::Imperial];
        let unit_picker = iced::widget::pick_list(
            unit_systems,
            Some(self.input_unit_system),
            |unit_system: UnitSystem| {
                application::Message::Settings(Message::UpdateUnitSystem(unit_system))
            },
        );
        let unit_system_row = row![unit_text, unit_picker];

        let body = column![db_row, divider, unit_system_row].height(Fill);

        let save_button =
            iced::widget::button("Save").on_press(application::Message::Settings(Message::Save));
        let footer_contents = row![save_button];
        let footer = footer(footer_contents);

        column![header, body, footer].into()
    }
    pub(super) fn update(&mut self, message: Message) -> Option<Command> {
        match message {
            Message::Save => {
                self.config.update_db_path(self.input_db_path.clone());
                self.config.update_default_units(self.input_unit_system);
                Some(Command::UpdateConfig(self.config.clone()))
            }
            Message::UpdateDBPath(db_path) => {
                self.input_db_path = db_path;
                None
            }
            Message::UpdateUnitSystem(unit_system) => {
                self.input_unit_system = unit_system;
                None
            }
        }
    }
}
