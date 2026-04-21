use iced::{
    Element,
    widget::{column, row, text_input},
};

use crate::{
    common::{category::Category, config::Config},
    presentation::{
        application,
        screen::Composition,
        widget::{self, text_style},
    },
};

#[derive(Debug)]
pub struct Categories {
    input_name: String,

    categories: Vec<Category>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CategoryListUpdate(Vec<Category>),
    NameUpdate(String),
    Save,
}

impl Categories {
    fn build_category_entry(&self) -> Element<'_, application::Message> {
        let entry_header = iced::widget::text("New Category:");
        let name_input = text_input("Name", &self.input_name)
            .id("name-input")
            .on_input(|str: String| application::Message::Categories(Message::NameUpdate(str)));
        let confirm_button = iced::widget::Button::new("Save")
            .on_press(application::Message::Categories(Message::Save));
        let entry_row = row![name_input, confirm_button];
        column![entry_header, entry_row].into()
    }
}

impl Composition<Message> for Categories {
    fn new(_config: &Config) -> Self {
        Self {
            input_name: String::new(),
            categories: Vec::new(),
        }
    }

    fn view(&self) -> Element<'_, application::Message> {
        let header = widget::header::header(text_style::title("Categories"));
        let category_entry = self.build_category_entry();
        let categories = column![];
        let body = column![category_entry, categories];
        column![header, body].into()
    }

    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::CategoryListUpdate(list) => {
                self.categories = list;
                None
            }
            Message::NameUpdate(name) => {
                self.input_name = name;
                None
            }
            Message::Save => Some(application::Command::AddCategory(self.input_name.clone())),
        }
    }
}
