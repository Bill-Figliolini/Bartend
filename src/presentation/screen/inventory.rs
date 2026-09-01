use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{column, container, row, table, text},
};

use crate::{
    logic::{CategoryService, ItemService},
    models::{Category, CategoryFilter, CategoryID, Config, Item, ItemBody, ItemID, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{InputMessage, ItemInput},
        widget::{footer::footer, header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Inventory {
    input: ItemInput,

    current_page: usize,
    unit_system: UnitSystem,

    edit_state: EditState,
}
#[derive(Debug)]
enum EditState {
    None,
    Editing(ItemID),
}

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    SwapUnits,
    BeginEdit(ItemID, Option<Category>),
    CancelEdit,
    Input(InputMessage),
    DeleteItem(ItemID),
    SelectPage(usize),

    //Variants for Application's use
    Reload,
}

pub enum Command {
    AddItem(ItemBody, Option<CategoryID>),
    UpdateItem(Item, Option<CategoryID>),
    DeleteItem(ItemID),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::AddItem(item_body, category_id) => {
                let mut tasks = Vec::new();
                let item_id = match ctx.item_service.add(&ctx.database.item_db(), &item_body) {
                    Ok(id) => {
                        tasks.push(Task::done(application::Message::ReloadScreen));
                        id
                    }
                    Err(e) => return Task::done(application::Message::Error(e)),
                };
                if let Some(category_id) = category_id
                    && let Err(e) = ctx.category_service.add_item_mapping(
                        &ctx.database.category_db(),
                        &item_id,
                        &category_id,
                    )
                {
                    tasks.push(Task::done(application::Message::Error(e)));
                }
                Task::batch(tasks)
            }
            Command::UpdateItem(item, category_id) => {
                let mut tasks = Vec::new();
                let item_id = item.id;
                if let Err(e) = ctx.item_service.update(&ctx.database.item_db(), item) {
                    tasks.push(Task::done(application::Message::Error(e)));
                } else {
                    tasks.push(Task::done(application::Message::ReloadScreen));
                }
                if let Err(e) = ctx.category_service.update_item_mapping(
                    &ctx.database.category_db(),
                    &item_id,
                    &category_id,
                ) {
                    tasks.push(Task::done(application::Message::Error(e)));
                }
                Task::batch(tasks)
            }
            Command::DeleteItem(id) => match ctx.item_service.delete(&ctx.database.item_db(), id) {
                Ok(()) => Task::done(application::Message::ReloadScreen),
                Err(e) => Task::done(application::Message::Error(e)),
            },
        }
    }
}

impl Inventory {
    pub fn new(config: &Config, category_service: &CategoryService) -> Self {
        let unit_system = config.default_units();

        Self {
            // Input Handlers
            input: ItemInput::new(
                config,
                input_msg,
                category_service.get_all(CategoryFilter {}),
            ),

            // Display Managers
            unit_system,
            current_page: 0,

            // Input State
            edit_state: EditState::None,
        }
    }

    fn build_item_entry_section(&self) -> Element<'_, application::Message> {
        let entry_header = match self.edit_state {
            EditState::None => text("New Item:"),
            EditState::Editing(_) => text("Edit Item:"),
        };
        let save_button = iced::widget::Button::new(text("save"))
            .on_press(application::Message::Inventory(Message::Save));
        column![entry_header, row![self.input.view(), save_button]].into()
    }

    fn build_inventory_display(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(Fill), |item: ItemID| {
            text(item_service.get(item).cloned().unwrap_or_default().name)
        });
        let quantity_column = table::column(text("Quantity"), |item: ItemID| {
            let item = item_service.get(item).cloned().unwrap_or_default();
            text!(
                "{:.0} {}",
                item.quantity.value(self.unit_system),
                item.quantity.unit(self.unit_system).to_string()
            )
        });
        let category_column =
            table::column(
                text("Category").width(200),
                |item: ItemID| match category_service.item_category(&item) {
                    Some(cat_id) => text(
                        category_service
                            .get(&cat_id)
                            .cloned()
                            .unwrap_or_default()
                            .name,
                    ),
                    None => text!("None"),
                },
            );
        //Something is wrong in the design here. Might be a misunderstanding of how to handle the edit state
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |item: ItemID| match self.edit_state {
                EditState::None => {
                    let category = category_service
                        .item_category(&item)
                        .map(|id_ref| Category {
                            id: id_ref,
                            body: category_service.get(&id_ref).cloned().unwrap_or_default(),
                        });
                    iced::widget::Button::new(text("Edit").center()).on_press(
                        application::Message::Inventory(Message::BeginEdit(item, category)),
                    )
                }
                .width(edit_column_width),
                EditState::Editing(item_id) if item == item_id => {
                    iced::widget::Button::new(text("Cancel").center())
                        .on_press(application::Message::Inventory(Message::CancelEdit))
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
            |item: ItemID| {
                iced::widget::Button::new(text("X").width(delete_column_width).center())
                    .on_press(application::Message::Inventory(Message::DeleteItem(item)))
            },
        );
        let columns = vec![
            name_column,
            quantity_column,
            category_column,
            edit_column,
            delete_column,
        ];
        let contents = item_service.get_page(self.current_page, 15);
        table(columns, contents).into()
    }

    pub fn view(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let title_text = title("Inventory");
        let header = header(title_text);

        let entry_section = self.build_item_entry_section();

        let inventory = self.build_inventory_display(item_service, category_service);

        let body_contents = column![entry_section, inventory];
        let body = container(body_contents).align_top(Fill);

        let mut page_controller_contents = Vec::new();
        if self.current_page != 0 {
            let previous_page = self.current_page.wrapping_sub(1);
            let previous_page_button = iced::widget::button("Previous")
                .on_press(application::Message::Inventory(Message::SelectPage(
                    previous_page,
                )))
                .into();
            page_controller_contents.push(previous_page_button);
        }
        let space = iced::widget::space().width(Fill).into();
        page_controller_contents.push(space);
        if (self.current_page + 1) * 15 < item_service.item_count() {
            let next_page = self.current_page.wrapping_add(1);
            let next_page_button = iced::widget::button("Next")
                .on_press(application::Message::Inventory(Message::SelectPage(
                    next_page,
                )))
                .into();
            page_controller_contents.push(next_page_button);
        }
        let page_controller = row(page_controller_contents);
        let unit_swap_button = iced::widget::Button::new(text(self.unit_system.to_string()))
            .on_press(application::Message::Inventory(Message::SwapUnits));
        let footer_contents = row![unit_swap_button];
        let footer_container = iced::widget::Container::new(footer_contents).align_left(Fill);
        let footer = footer(footer_container);

        column![header, body, page_controller, footer].into()
    }

    pub fn update(&mut self, item_service: &ItemService, message: Message) -> Option<Command> {
        match message {
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::CancelEdit => {
                self.edit_state = EditState::None;
                None
            }
            Message::Input(msg) => {
                self.input.update(msg);
                None
            }
            Message::BeginEdit(item, category) => match self.edit_state {
                EditState::None => {
                    self.edit_state = EditState::Editing(item);
                    self.input.begin_edit(
                        &(
                            item_service.get(item).cloned().unwrap_or_default(),
                            category,
                        ),
                        self.unit_system,
                    );
                    None
                }
                EditState::Editing(item_id) if item == item_id => {
                    self.input.clear();
                    self.edit_state = EditState::None;
                    None
                }
                EditState::Editing(_) => None,
            },
            Message::Save => {
                if let Ok(result) = self.input.output() {
                    match self.edit_state {
                        EditState::None => {
                            Some(Command::AddItem(result.0, result.1.map(|cat| cat.id)))
                        }
                        EditState::Editing(item_id) => Some(Command::UpdateItem(
                            Item {
                                id: item_id,
                                body: result.0,
                            },
                            result.1.map(|cat| cat.id),
                        )),
                    }
                } else {
                    None
                }
            }
            Message::Reload => {
                self.input.clear();
                self.edit_state = EditState::None;
                self.current_page = self
                    .current_page
                    .min((item_service.item_count() / 15).saturating_sub(1));
                None
            }
            Message::DeleteItem(item) => Some(Command::DeleteItem(item)),
            Message::SelectPage(page) => {
                self.current_page = page;
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Inventory(Message::Input(msg))
}
