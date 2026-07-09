use crate::{
    logic::CategoryService,
    models::{Category, CategoryID, Config, UnitSystem},
    presentation::{
        Viewable, application,
        input_handling::{CategoryInput, EditableCollection, InputCollection, InputMessage},
        widget::{self, text_style},
    },
};
use iced::{
    Element,
    widget::{column, row, table, text},
};

#[derive(Debug)]
pub struct Categories {
    input: CategoryInput,

    edit_state: EditState,

    contents: Vec<CategoryID>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Reload,
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
    pub fn new(config: &Config, category_service: &CategoryService) -> Self {
        Self {
            input: CategoryInput::new(config, input_msg),
            edit_state: EditState::None,
            contents: category_service.get_page(0),
        }
    }
    fn build_category_entry(&self) -> Element<'_, application::Message> {
        let header = match self.edit_state {
            EditState::None => iced::widget::text("New Category:"),
            EditState::Editing(_) => iced::widget::text("Edit Category:"),
        };
        let save_button = iced::widget::Button::new("Save")
            .on_press(application::Message::Categories(Message::Save));
        let body = row![self.input.view(), save_button];
        column![header, body].into()
    }
    fn build_category_display(
        &self,
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(200), |category: &CategoryID| {
            text(category_service.get(category).name.clone())
        });
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |category: &CategoryID| match self.edit_state {
                EditState::None => iced::widget::Button::new(text("Edit").center())
                    .on_press(application::Message::Categories(Message::BeginEdit(
                        Category {
                            id: category.clone(),
                            body: category_service.get(category).clone(),
                        },
                    )))
                    .width(edit_column_width),
                EditState::Editing(category_id) if *category == category_id => {
                    iced::widget::Button::new(text("Cancel").center())
                        .on_press(application::Message::ReloadScreen)
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
            |category: &CategoryID| {
                iced::widget::Button::new(text("X").width(delete_column_width).center())
                    .on_press(application::Message::DeleteCategory(category.clone()))
            },
        );
        let columns = vec![name_column, edit_column, delete_column];
        table(columns, &self.contents).into()
    }
    pub fn view(&self, category_service: &CategoryService) -> Element<'_, application::Message> {
        let header = widget::header::header(text_style::title("Categories"));
        let category_entry = self.build_category_entry();
        let categories = self.build_category_display(category_service);
        let body = column![category_entry, categories];
        column![header, body].into()
    }
    pub fn update(
        &mut self,
        category_service: &CategoryService,
        message: Message,
    ) -> Option<application::Command> {
        match message {
            Message::Input(msg) => {
                self.input.update(msg);
                None
            }
            Message::BeginEdit(category) => {
                match self.edit_state {
                    EditState::None => {
                        self.edit_state = EditState::Editing(category.id);
                        self.input.begin_edit(&category.body, UnitSystem::Metric);
                    }
                    EditState::Editing(category_id) if category_id == category.id => {
                        self.input.clear();
                        self.edit_state = EditState::None;
                    }
                    _ => unreachable!(
                        "Edit buttons to items not currently under edit should not be accesable"
                    ),
                }
                None
            }
            Message::Save => match self.input.output() {
                Ok(body) => match self.edit_state {
                    EditState::Editing(id) => {
                        Some(application::Command::UpdateCategory(Category { id, body }))
                    }
                    EditState::None => Some(application::Command::AddCategory(body)),
                },
                Err(()) => None,
            },
            Message::Reload => {
                self.contents = category_service.get_page(0);
                self.input.clear();
                self.edit_state = EditState::None;
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Categories(Message::Input(msg))
}
