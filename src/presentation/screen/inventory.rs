use std::mem::take;

use iced::{
    Element,
    Length::Fill,
    widget::{column, container, pick_list, row, rule, table, text, text_input},
};

use crate::{
    logic::{
        category::Category,
        config::Config,
        item::{Item, ItemID},
        quantity::{Quantity, Unit, UnitSystem},
    },
    presentation::{
        application, constants,
        screen::Composition,
        widget::{footer::footer, header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Inventory {
    input_name: String,
    input_quantity: String,
    input_unit: Unit,
    input_category: Option<Category>,

    contents: Vec<Item>,
    unit_system: UnitSystem,

    edit_state: EditState,
    errors: Vec<Error>,
}
#[derive(Debug)]
enum EditState {
    None,
    Editing(ItemID),
}
#[derive(Debug, Hash, PartialEq, Eq)]
enum Error {
    NameError,
    QuantityError,
}
#[derive(Debug, Clone)]
pub enum Message {
    Save,
    SwapUnits,
    BeginEdit(Item),
    NameUpdate(String),
    QuantityUpdate(String),
    UnitUpdate(Unit),
    CategoryUpdate(Option<Category>),

    //Variants for Application's use
    InventoryUpdate(Vec<Item>),
}
impl Inventory {
    fn save_item(&mut self, quantity: f32) -> application::Command {
        let quantity = Quantity::new(quantity, self.input_unit);
        let name = take(&mut self.input_name);
        self.input_quantity.clear();
        match self.edit_state {
            EditState::None => application::Command::AddItem(name, quantity),
            EditState::Editing(item_id) => application::Command::UpdateItem(Item {
                id: item_id,
                name,
                quantity,
            }),
        }
    }

    pub(super) fn build_item_entry_section(&self) -> Element<'_, application::Message> {
        let entry_header = match self.edit_state {
            EditState::None => text("New Item:"),
            EditState::Editing(_) => text("Edit Item:"),
        };
        let name_input = text_input("Name", &self.input_name)
            .id("name-input")
            .on_input(|str: String| application::Message::Inventory(Message::NameUpdate(str)));
        let quantity_input = text_input("Quantity", &self.input_quantity)
            .id("quantity-input")
            .on_input(|str: String| application::Message::Inventory(Message::QuantityUpdate(str)));
        let units = vec![
            Unit::Milliliter,
            Unit::FluidOunce,
            Unit::Gram,
            Unit::MassOunce,
            Unit::Dash,
        ];
        let unit_select = pick_list(units, Some(self.input_unit), |unit: Unit| {
            application::Message::Inventory(Message::UnitUpdate(unit))
        });

        let categories: Vec<Category> = Vec::new();
        let category_select = pick_list(
            categories,
            self.input_category.clone(),
            |category: Category| {
                application::Message::Inventory(Message::CategoryUpdate(Some(category)))
            },
        );

        let confirm_button = iced::widget::Button::new("Save")
            .on_press(application::Message::Inventory(Message::Save));
        let entry_row = row![
            name_input,
            quantity_input,
            unit_select,
            category_select,
            confirm_button
        ]
        .spacing(5);

        let mut error_row = row![].spacing(20);
        for error in &self.errors {
            match error {
                Error::NameError => {
                    error_row = error_row.push(text!("Name Must Not Be Empty"));
                }
                Error::QuantityError => {
                    error_row =
                        error_row.push(text!("Quantity must be a positive, non-zero number"));
                }
            }
        }

        let input_divider = rule::horizontal(constants::DIV_SIZE);
        iced::widget::container(column![entry_header, entry_row, error_row, input_divider]).into()
    }

    fn build_inventory_display(&self) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(200), |item: &Item| text(&item.name));
        let quantity_column = table::column(text("Quantity"), |item: &Item| {
            text!(
                "{} {}",
                item.quantity.value(self.unit_system),
                item.quantity.unit(self.unit_system).to_string()
            )
        });
        //Something is wrong in the design here. Might be a misunderstanding of how to handle the edit state
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |item: &Item| match self.edit_state {
                EditState::None => iced::widget::Button::new(text("Edit").center())
                    .on_press(application::Message::Inventory(Message::BeginEdit(
                        item.clone(),
                    )))
                    .width(edit_column_width),
                EditState::Editing(item_id) if item.id == item_id => {
                    iced::widget::Button::new(text("Cancel").center())
                        .on_press(application::Message::RefreshItems)
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
        let columns = vec![name_column, quantity_column, edit_column, delete_column];
        table(columns, &self.contents).into()
    }
}

impl Composition<Message> for Inventory {
    fn new(config: &Config) -> Self {
        let unit_system = config.default_units();
        let input_unit = match &unit_system {
            UnitSystem::Metric => Unit::Milliliter,
            UnitSystem::Imperial => Unit::FluidOunce,
        };
        Self {
            // Input Handlers
            input_name: String::new(),
            input_quantity: String::new(),
            input_unit,
            input_category: None,

            // Display Managers
            contents: Vec::new(),
            unit_system,

            // Input State
            edit_state: EditState::None,
            errors: Vec::with_capacity(2),
        }
    }

    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::NameUpdate(new) => {
                self.input_name = new;
                None
            }
            Message::QuantityUpdate(new) => {
                self.input_quantity = new;
                None
            }
            Message::UnitUpdate(new) => {
                self.input_unit = new;
                None
            }
            Message::CategoryUpdate(new) => {
                self.input_category = if self.input_category == new {
                    None
                } else {
                    new
                };
                None
            }
            Message::BeginEdit(item) => {
                self.edit_state = EditState::Editing(item.id);
                self.input_name = item.name;
                self.input_quantity = item.quantity.value(self.unit_system).to_string();
                self.input_unit = item.quantity.unit(self.unit_system);
                None
            }
            Message::Save => {
                self.errors.clear();

                if self.input_name.is_empty() {
                    self.errors.push(Error::NameError);
                }
                let quantity = self.input_quantity.trim().parse::<f32>();
                let quantity = match quantity {
                    Ok(quantity) if quantity > 0.0 => quantity,
                    _ => {
                        self.errors.push(Error::QuantityError);
                        0.0
                    }
                };
                if self.errors.is_empty() {
                    Some(self.save_item(quantity))
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

    fn view(&self) -> Element<'_, application::Message> {
        let title_text = title("Inventory");
        let header = header(title_text);

        let entry_section = self.build_item_entry_section();

        let inventory = self.build_inventory_display();

        let body_contents = column![entry_section, inventory];
        let body = container(body_contents).align_top(Fill);

        let unit_swap_button = iced::widget::Button::new(text(self.unit_system.to_string()))
            .on_press(application::Message::Inventory(Message::SwapUnits));
        let footer_contents = row![unit_swap_button];
        let footer_container = iced::widget::Container::new(footer_contents).align_left(Fill);
        let footer = footer(footer_container);

        column![header, body, footer].into()
    }
}
