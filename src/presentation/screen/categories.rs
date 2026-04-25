use std::{collections::HashSet, fmt::Display, mem::take};

use iced::{
    Element,
    widget::{column, row, table, text, text_input},
};

use crate::{
    logic::{
        category::{Category, CategoryID},
        config::Config,
    },
    presentation::{
        application,
        screen::Composition,
        widget::{self, text_style},
    },
};

#[derive(Debug)]
pub struct Categories {
    input_name: String,

    edit_state: EditState,
    errors: HashSet<Error>,

    contents: Vec<Category>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CategoryListUpdate(Vec<Category>),
    NameUpdate(String),
    BeginEdit(Category),
    Save,
}
#[derive(Debug)]
enum EditState {
    Editing(CategoryID),
    None,
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Error {
    NameEmpty,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::NameEmpty => "Name Must Not Be Empty".to_string(),
        };
        write!(f, "{text}")
    }
}

impl Categories {
    fn save(&mut self) -> Option<application::Command> {
        if self.input_name.is_empty() {
            self.errors.insert(Error::NameEmpty);
            return None;
        }
        let name = take(&mut self.input_name);
        match self.edit_state {
            EditState::Editing(id) => {
                Some(application::Command::UpdateCategory(Category { id, name }))
            }
            EditState::None => Some(application::Command::AddCategory(name)),
        }
    }
    fn build_category_entry(&self) -> Element<'_, application::Message> {
        let entry_header = iced::widget::text("New Category:");
        let name_input = text_input("Name", &self.input_name)
            .id("name-input")
            .on_input(|str: String| application::Message::Categories(Message::NameUpdate(str)));
        let confirm_button = iced::widget::Button::new("Save")
            .on_press(application::Message::Categories(Message::Save));
        let entry_row = row![name_input, confirm_button];
        let error_row = row(self
            .errors
            .iter()
            .map(|error| text(error.to_string()).into()));
        column![entry_header, entry_row, error_row].into()
    }
    fn build_category_display(&self) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(200), |category: &Category| {
            text(&category.name)
        });
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |category: &Category| match self.edit_state {
                EditState::None => iced::widget::Button::new(text("Edit").center())
                    .on_press(application::Message::Categories(Message::BeginEdit(
                        category.clone(),
                    )))
                    .width(edit_column_width),
                EditState::Editing(category_id) if category.id == category_id => {
                    iced::widget::Button::new(text("Cancel").center())
                        .on_press(application::Message::UpdateCategories)
                        .width(edit_column_width)
                }
                EditState::Editing(_) => {
                    iced::widget::Button::new(text("Edit").center()).width(edit_column_width)
                }
            },
        );
        let delete_column_width = 50;
        let delete_column = table::column(
            text("Delete").width(delete_column_width).center(),
            |category: &Category| {
                iced::widget::Button::new(text("X").width(delete_column_width).center())
                    .on_press(application::Message::DeleteCategory(category.clone()))
            },
        );
        let columns = vec![name_column, edit_column, delete_column];
        table(columns, &self.contents).into()
    }
}

impl Composition<Message> for Categories {
    fn new(_config: &Config) -> Self {
        Self {
            input_name: String::new(),
            edit_state: EditState::None,
            errors: HashSet::new(),
            contents: Vec::new(),
        }
    }

    fn view(&self) -> Element<'_, application::Message> {
        let header = widget::header::header(text_style::title("Categories"));
        let category_entry = self.build_category_entry();
        let categories = self.build_category_display();
        let body = column![category_entry, categories];
        column![header, body].into()
    }

    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::CategoryListUpdate(list) => {
                self.contents = list;
                self.edit_state = EditState::None;
                self.input_name.clear();
                None
            }
            Message::NameUpdate(name) => {
                self.input_name = name;
                None
            }
            Message::BeginEdit(category) => {
                self.input_name = category.name;
                self.edit_state = EditState::Editing(category.id);
                None
            }
            Message::Save => {
                self.errors.clear();
                self.save()
            }
        }
    }
}
