use std::collections::HashSet;

use iced::{
    Element,
    widget::{column, row, table, text},
};

use crate::{
    logic::{
        category::{Category, CategoryID},
        config::Config,
    },
    presentation::{
        application,
        screen::Composition,
        widget::{
            self,
            input::{Error, Input, StringInputUpdate, name_input::NameInput},
            text_style,
        },
    },
};

#[derive(Debug)]
pub struct Categories {
    input_name: NameInput,

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
impl Categories {
    fn save(&mut self, name: String) -> application::Command {
        match self.edit_state {
            EditState::Editing(id) => application::Command::UpdateCategory(Category { id, name }),
            EditState::None => application::Command::AddCategory(name),
        }
    }
    fn build_category_entry(&self) -> Element<'_, application::Message> {
        let entry_header = match self.edit_state {
            EditState::None => iced::widget::text("New Category:"),
            EditState::Editing(_) => iced::widget::text("Edit Category:"),
        };
        let name_input = self.input_name.display();
        let confirm_button = iced::widget::Button::new("Save")
            .on_press(application::Message::Categories(Message::Save));
        let entry_row = row![name_input, confirm_button];
        let error_row = row(self
            .errors
            .iter()
            .map(|error| text!("{} ", error.to_string()).into()));
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
            input_name: NameInput::new("name-input", |str: String| {
                application::Message::Categories(Message::NameUpdate(str))
            }),
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
                self.input_name.string_update(name);
                None
            }
            Message::BeginEdit(category) => {
                self.input_name.string_update(category.name);
                self.edit_state = EditState::Editing(category.id);
                None
            }
            Message::Save => match self.input_name.get_output() {
                Ok(name) => {
                    self.input_name.clear();
                    Some(self.save(name))
                }
                Err(e) => {
                    self.errors.insert(e);
                    None
                }
            },
        }
    }
}
