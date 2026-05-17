use std::collections::HashSet;

use iced::{
    Element,
    widget::{column, row, table, text},
};

use crate::{
    logic::{
        category::{Category, CategoryBody, CategoryID},
        config::Config,
    },
    presentation::{
        Updateable, Viewable, application,
        input_handling::{InputCollection, InputMessage, category_input::CategoryInput},
        widget::{self, input::Error, text_style},
    },
};

#[derive(Debug)]
pub struct Categories {
    input: CategoryInput,

    edit_state: EditState,
    errors: HashSet<Error>,

    contents: Vec<Category>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CategoryListUpdate(Vec<Category>),
    Input(InputMessage),
    BeginEdit(Category),
    Save,
}
#[derive(Debug)]
enum EditState {
    Editing(CategoryID),
    None,
}
impl Categories {
    pub fn new(config: &Config) -> Self {
        Self {
            input: CategoryInput::new(config, input_msg),
            edit_state: EditState::None,
            errors: HashSet::new(),
            contents: Vec::new(),
        }
    }
    fn save(&mut self, name: String) -> application::Command {
        match self.edit_state {
            EditState::Editing(id) => application::Command::UpdateCategory(Category {
                id,
                body: CategoryBody { name },
            }),
            EditState::None => application::Command::AddCategory(name),
        }
    }
    fn build_category_entry(&self) -> Element<'_, application::Message> {
        let entry_header = match self.edit_state {
            EditState::None => iced::widget::text("New Category:"),
            EditState::Editing(_) => iced::widget::text("Edit Category:"),
        };
        column![entry_header, self.input.view()].into()
    }
    fn build_category_display(&self) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(200), |category: &Category| {
            text(&category.body.name)
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

impl Viewable<application::Message> for Categories {
    fn view(&self) -> Element<'_, application::Message> {
        let header = widget::header::header(text_style::title("Categories"));
        let category_entry = self.build_category_entry();
        let categories = self.build_category_display();
        let body = column![category_entry, categories];
        column![header, body].into()
    }
}
impl Updateable<Message> for Categories {
    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::CategoryListUpdate(list) => None,
            Message::Input(msg) => None,
            Message::BeginEdit(category) => {
                self.edit_state = EditState::Editing(category.id);
                None
            }
            Message::Save => None,
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Categories(Message::Input(msg))
}
