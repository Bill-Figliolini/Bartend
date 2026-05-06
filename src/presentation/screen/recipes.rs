use std::collections::HashSet;

use iced::{
    Element,
    widget::{button, column, row, text, text_input},
};

use crate::{
    logic::{category::Category, config::Config, quantity::UnitSystem},
    presentation::{
        application,
        screen::Composition,
        widget::{
            header::header,
            input::{Error, name_unload},
            text_style::title,
        },
    },
};

#[derive(Debug)]
pub struct Recipes {
    input_name: String, //I can improve this by newtyping it. with a trait for display.
    unit_system: UnitSystem,

    errors: HashSet<Error>,

    categories: Vec<Category>,
}
impl Recipes {
    fn build_input(&self) -> Element<'_, application::Message> {
        let name_input = text_input("Name", &self.input_name)
            .id("name-input")
            .on_input(|str: String| application::Message::Recipes(Message::NameUpdate(str)));
        let save_button = button("Save").on_press(application::Message::Recipes(Message::Save));
        let input_row = row![name_input, save_button];

        let error_row = row(self
            .errors
            .iter()
            .map(|error| text!("{} ", error.to_string()).into()));

        column![input_row, error_row].into()
    }
    fn save(&mut self, name: String) -> application::Command {
        self.errors.clear();

        todo!()
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    NameUpdate(String),
    Save,

    InitializeCategoryList(Vec<Category>),
}
impl Composition<Message> for Recipes {
    fn new(config: &Config) -> Self {
        let unit_system = config.default_units();
        Self {
            input_name: String::new(),
            unit_system,
            errors: HashSet::new(),
            categories: Vec::new(),
        }
    }

    fn view(&self) -> Element<'_, application::Message> {
        let header = header(title("Recipes"));
        let input_row = self.build_input();
        let body = column![input_row];
        column![header, body].into()
    }

    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::NameUpdate(new_name) => {
                self.input_name = new_name;
                None
            }
            Message::Save => match name_unload(&self.input_name) {
                Ok(name) => {
                    self.input_name.clear();
                    Some(self.save(name))
                }
                Err(e) => {
                    self.errors.insert(e);
                    None
                }
            },
            Message::InitializeCategoryList(categories) => {
                self.categories = categories;
                None
            }
        }
    }
}
