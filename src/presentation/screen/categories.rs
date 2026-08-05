use crate::{
    logic::CategoryService,
    models::{Category, CategoryBody, CategoryID, Config, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{CategoryInput, InputMessage},
        widget::{self, text_style},
    },
};
use iced::{
    Element, Task,
    widget::{column, pick_list, row, table, text},
};

#[derive(Debug)]
pub struct Categories {
    input: CategoryInput,

    edit_state: EditState,

    page_number: usize,
    stub: Option<Category>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Reload,
    Input(InputMessage),
    BeginEdit(Category),
    AddRelation(CategoryID, CategoryID),
    RemoveRelation(CategoryID, CategoryID),
    Save,
}
pub enum Command {
    AddCategory(CategoryBody),
    UpdateCategory(Category),
    AddRelation(CategoryID, CategoryID),
    RemoveRelation(CategoryID, CategoryID),
}
impl Command {
    pub fn apply(self, ctx: &mut Context) -> Task<application::Message> {
        match self {
            Command::AddCategory(body) => {
                ctx.category_service
                    .insert(&ctx.bar_collection.db.category_db(), &body);
                Task::done(application::Message::ReloadScreen)
            }
            Command::UpdateCategory(category) => {
                ctx.category_service
                    .update(&ctx.bar_collection.db.category_db(), &category);
                Task::done(application::Message::ReloadScreen)
            }
            Command::AddRelation(parent, child) => {
                let _result = ctx.category_service.add_category_relation(
                    &ctx.bar_collection.db.category_db(),
                    &parent,
                    &child,
                );
                Task::none()
            }
            Command::RemoveRelation(parent, child) => {
                let _result = ctx.category_service.remove_category_relation(
                    &ctx.bar_collection.db.category_db(),
                    &parent,
                    &child,
                );
                Task::none()
            }
        }
    }
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
            page_number: 0,
            stub: None,
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
        let name_column = table::column(text("Name").width(200), |category: CategoryID| {
            text(category_service.get(&category).name.clone())
        });
        let add_relation_column = table::column("Addable Relations", |category: CategoryID| {
            self.relation_add_view(category, category_service)
        });
        let list_relation_column = table::column("Current Relations", |category: CategoryID| {
            self.category_remove_view(category, category_service)
        });
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |category: CategoryID| match self.edit_state {
                EditState::None => iced::widget::Button::new(text("Edit").center())
                    .on_press(application::Message::Categories(Message::BeginEdit(
                        Category {
                            id: category,
                            body: category_service.get(&category).clone(),
                        },
                    )))
                    .width(edit_column_width),
                EditState::Editing(category_id) if category == category_id => {
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
            |category: CategoryID| {
                iced::widget::Button::new(text("X").width(delete_column_width).center())
                    .on_press(application::Message::DeleteCategory(category))
            },
        );
        let columns = vec![
            name_column,
            add_relation_column,
            list_relation_column,
            edit_column,
            delete_column,
        ];
        let contents = category_service.get_page(self.page_number, 15);
        table(columns, contents).into()
    }
    pub fn view(&self, category_service: &CategoryService) -> Element<'_, application::Message> {
        let header = widget::header::header(text_style::title("Categories"));
        let category_entry = self.build_category_entry();
        let categories = self.build_category_display(category_service);
        let body = column![category_entry, categories];
        column![header, body].into()
    }
    pub fn update(&mut self, message: Message) -> Option<Command> {
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
                    EditState::Editing(id) => Some(Command::UpdateCategory(Category { id, body })),
                    EditState::None => Some(Command::AddCategory(body)),
                },
                Err(()) => None,
            },
            Message::Reload => {
                self.input.clear();
                self.edit_state = EditState::None;
                None
            }
            Message::AddRelation(parent, child) => Some(Command::AddRelation(parent, child)),
            Message::RemoveRelation(parent, child) => Some(Command::RemoveRelation(parent, child)),
        }
    }
    fn relation_add_view(
        &self,
        id: CategoryID,
        service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let options = service
            .valid_relations(&id)
            .into_iter()
            .fold(Vec::new(), |mut acc, id| {
                acc.push(Category {
                    id,
                    body: service.get(&id).clone(),
                });
                acc
            });
        let id = id.clone();
        pick_list(options, self.stub.clone(), move |category| {
            category_add_msg(id, category.id)
        })
        .placeholder("Add a Subcategory")
        .into()
    }
    fn category_remove_view(
        &self,
        id: CategoryID,
        service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let options = service
            .child_categories(&id)
            .into_iter()
            .fold(Vec::new(), |mut acc, id| {
                acc.push(Category {
                    id,
                    body: service.get(&id).clone(),
                });
                acc
            });
        pick_list(options, self.stub.clone(), move |selected| {
            category_remove_msg(id, selected.id)
        })
        .placeholder("List subcategories (click to remove)")
        .into()
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Categories(Message::Input(msg))
}

fn category_add_msg(parent: CategoryID, child: CategoryID) -> application::Message {
    application::Message::Categories(Message::AddRelation(parent, child))
}

fn category_remove_msg(parent: CategoryID, child: CategoryID) -> application::Message {
    application::Message::Categories(Message::RemoveRelation(parent, child))
}
