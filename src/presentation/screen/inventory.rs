use std::collections::HashMap;

use iced::{
    Element,
    Length::Fill,
    widget::{column, container, row, table, text},
};

use crate::{
    logic::CategoryService,
    models::{Category, CategoryFilter, CategoryID, Config, Item, ItemID, UnitSystem},
    presentation::{
        Updateable, Viewable, application,
        input_handling::{EditableCollection, InputCollection, InputMessage, ItemInput},
        widget::{footer::footer, header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Inventory {
    input: ItemInput,

    contents: Vec<Item>,
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
    BeginEdit(Item, Option<Category>),
    Input(InputMessage),

    //Variants for Application's use
    InventoryUpdate(Vec<Item>),
}

impl Inventory {
    pub fn new(config: &Config, items: Vec<Item>, category_service: &CategoryService) -> Self {
        let unit_system = config.default_units();

        Self {
            // Input Handlers
            input: ItemInput::new(
                config,
                input_msg,
                category_service.get_all(CategoryFilter {}),
            ),

            // Display Managers
            contents: items,
            unit_system,

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
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(Fill), |item: &Item| {
            text(&item.body.name)
        });
        let quantity_column = table::column(text("Quantity"), |item: &Item| {
            text!(
                "{:.0} {}",
                item.body.quantity.value(self.unit_system),
                item.body.quantity.unit(self.unit_system).to_string()
            )
        });
        let category_column =
            table::column(
                text("Category").width(200),
                |item: &Item| match category_service.item_category(&item.id) {
                    Some(cat_id) => text(category_service.get(cat_id).name.clone()),
                    None => text!("None"),
                },
            );
        //Something is wrong in the design here. Might be a misunderstanding of how to handle the edit state
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |item: &Item| match self.edit_state {
                EditState::None => {
                    let category = match category_service.item_category(&item.id) {
                        Some(id_ref) => Some(Category {
                            id: *id_ref,
                            body: category_service.get(id_ref).clone(),
                        }),
                        None => None,
                    };
                    iced::widget::Button::new(text("Edit").center()).on_press(
                        application::Message::Inventory(Message::BeginEdit(item.clone(), category)),
                    )
                }
                .width(edit_column_width),
                EditState::Editing(item_id) if item.id == item_id => {
                    iced::widget::Button::new(text("Cancel").center())
                        .on_press(application::Message::UpdateInventory)
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
            |item: &Item| {
                iced::widget::Button::new(text("X").width(delete_column_width).center())
                    .on_press(application::Message::DeleteItem(item.clone()))
            },
        );
        let columns = vec![
            name_column,
            quantity_column,
            category_column,
            edit_column,
            delete_column,
        ];
        table(columns, &self.contents).into()
    }

    pub fn view(&self, category_service: &CategoryService) -> Element<'_, application::Message> {
        let title_text = title("Inventory");
        let header = header(title_text);

        let entry_section = self.build_item_entry_section();

        let inventory = self.build_inventory_display(category_service);

        let body_contents = column![entry_section, inventory];
        let body = container(body_contents).align_top(Fill);

        let unit_swap_button = iced::widget::Button::new(text(self.unit_system.to_string()))
            .on_press(application::Message::Inventory(Message::SwapUnits));
        let footer_contents = row![unit_swap_button];
        let footer_container = iced::widget::Container::new(footer_contents).align_left(Fill);
        let footer = footer(footer_container);

        column![header, body, footer].into()
    }

    pub fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::Input(msg) => {
                self.input.update(msg);
                None
            }
            Message::BeginEdit(item, category) => match self.edit_state {
                EditState::None => {
                    self.edit_state = EditState::Editing(item.id);
                    self.input
                        .begin_edit(&(item.body, category), self.unit_system);
                    None
                }
                EditState::Editing(item_id) if item.id == item_id => {
                    self.input.clear();
                    self.edit_state = EditState::None;
                    None
                }
                _ => unreachable!(
                    "Edit buttons to items not currently under edit should not be accesable"
                ),
            },
            Message::Save => {
                if let Ok(result) = self.input.output() {
                    match self.edit_state {
                        EditState::None => Some(application::Command::AddItem(
                            result.0,
                            result.1.map(|cat| cat.id),
                        )),
                        EditState::Editing(item_id) => Some(application::Command::UpdateItem(
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
            Message::InventoryUpdate(items) => {
                self.contents = items;
                self.edit_state = EditState::None;
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Inventory(Message::Input(msg))
}
