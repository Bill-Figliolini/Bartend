use std::path::PathBuf;

use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{column, row},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, EditableConfig, UnitSystem},
    persistence::Database,
    presentation::{
        application::{self, Context},
        constants,
        widget::{footer::footer, header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Settings {
    config: EditableConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    UpdateDBPath(PathBuf),
    UpdateUnitSystem(UnitSystem),
    ResetConfig(Config),
}

pub enum Command {
    UpdateConfig(Config),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::UpdateConfig(config) => {
                let db_changed = ctx.config.db_path() != config.db_path();
                if db_changed {
                    match Database::load(config.db_path()) {
                        Ok(db) => {
                            let item_service = match ItemService::new(&db.item_db()) {
                                Ok(service) => service,
                                Err(e) => {
                                    return Task::done(application::Message::Error(e));
                                }
                            };
                            let category_service = match CategoryService::new(&db.category_db()) {
                                Ok(service) => service,
                                Err(e) => {
                                    return Task::done(application::Message::Error(e));
                                }
                            };
                            let recipe_service = match RecipeService::new(&db.recipe_db()) {
                                Ok(service) => service,
                                Err(e) => {
                                    return Task::done(application::Message::Error(e));
                                }
                            };
                            *ctx.item_service = item_service;
                            *ctx.category_service = category_service;
                            *ctx.recipe_service = recipe_service;
                            *ctx.database = db
                        }
                        Err(e) => {
                            return Task::done(application::Message::Error(
                                crate::models::BartendError::Db(e),
                            ));
                        }
                    }
                }
                if let Err(e) = config.save() {
                    return Task::done(application::Message::Error(
                        crate::models::BartendError::Config(e),
                    ));
                }
                *ctx.config = config;
                Task::done(application::Message::ReloadScreen)
            }
        }
    }
}

impl Settings {
    pub fn new(current_config: &Config) -> Self {
        let config = current_config.editable();
        Self { config }
    }

    pub fn view(&self) -> Element<'_, application::Message> {
        let text_boundary = 500;

        let title_text = title("Settings");
        let header = header(title_text);

        let current_db_path = iced::widget::text!(
            "Current DB Path: {}",
            self.config.db_path.display().to_string()
        )
        .width(text_boundary);
        let db_button = iced::widget::Button::new("Choose DB File").on_press(
            application::Message::OpenDBPicker(self.config.db_path.clone()),
        );
        let db_row = row![current_db_path, db_button];

        let divider = iced::widget::rule::horizontal(constants::DIV_SIZE);

        let unit_text = iced::widget::text("Default units:").width(text_boundary);
        let unit_systems = vec![UnitSystem::Metric, UnitSystem::Imperial];
        let unit_picker = iced::widget::pick_list(
            unit_systems,
            Some(self.config.default_unit_system),
            |unit_system: UnitSystem| {
                application::Message::Settings(Message::UpdateUnitSystem(unit_system))
            },
        );
        let unit_system_row = row![unit_text, unit_picker];

        let body = column![db_row, divider, unit_system_row].height(Fill);

        let save_button =
            iced::widget::button("Save").on_press(application::Message::Settings(Message::Save));
        let cancel_button =
            iced::widget::button("Cancel").on_press(application::Message::ResetSettings);
        let footer_contents = row![save_button, cancel_button].spacing(20);
        let footer_container = iced::widget::Container::new(footer_contents).center_x(Fill);
        let footer = footer(footer_container);

        column![header, body, footer].into()
    }

    pub fn update(&mut self, message: Message) -> Option<Command> {
        match message {
            Message::Save => Some(Command::UpdateConfig(self.config.commit())),
            Message::UpdateDBPath(db_path) => {
                self.config.db_path = db_path;
                None
            }
            Message::UpdateUnitSystem(unit_system) => {
                self.config.default_unit_system = unit_system;
                None
            }
            Message::ResetConfig(config) => {
                self.config = config.editable();
                None
            }
        }
    }
}
