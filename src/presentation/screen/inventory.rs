use std::{
    collections::{HashMap, HashSet},
    mem::take,
};

use iced::{
    Element,
    Length::Fill,
    widget::{column, container, pick_list, row, rule, table, text, text_input},
};

use crate::{
    logic::{
        category::{Category, CategoryID},
        config::Config,
        item::{Item, ItemID},
        quantity::{Quantity, Unit, UnitSystem},
    },
    presentation::{
        Composition, application, constants,
        widget::{
            footer::footer,
            header::header,
            input::{Error, Input, InputString, name_input::NameInput, quantity_unload},
            text_style::title,
        },
    },
};

#[derive(Debug)]
pub struct Inventory {
    input_name: NameInput,
    input_quantity: String,
    input_unit: Unit,
    input_category: Option<Category>,

    contents: Vec<Item>,
    categories: Vec<Category>,
    item_category_mapping: HashMap<ItemID, CategoryID>,
    unit_system: UnitSystem,

    edit_state: EditState,
    errors: HashSet<Error>,
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
    NameUpdate(String),
    QuantityUpdate(String),
    UnitUpdate(Unit),
    CategoryUpdate(Option<Category>),

    //Variants for Application's use
    InventoryUpdate(Vec<Item>),
    CategoryMappingUpdate(HashMap<ItemID, CategoryID>),
    CategoryListInitialization(Vec<Category>),
}
impl Inventory {
    fn save(&mut self, name: String, quantity: Quantity) -> application::Command {
        let category_id = match take(&mut self.input_category) {
            Some(category) => Some(category.id),
            None => None,
        };
        match self.edit_state {
            EditState::None => application::Command::AddItem(name, quantity, category_id),
            EditState::Editing(item_id) => application::Command::UpdateItem(
                Item {
                    id: item_id,
                    name,
                    quantity,
                },
                category_id,
            ),
        }
    }

    fn build_item_entry_section(&self) -> Element<'_, application::Message> {
        let entry_header = match self.edit_state {
            EditState::None => text("New Item:"),
            EditState::Editing(_) => text("Edit Item:"),
        };
        let name_input = self.input_name.display();
        let quantity_input = text_input("Quantity", &self.input_quantity)
            .id("quantity-input")
            .on_input(|str: String| application::Message::Inventory(Message::QuantityUpdate(str)));
        let units = Unit::get_units();
        let unit_select = pick_list(units, Some(self.input_unit), |unit: Unit| {
            application::Message::Inventory(Message::UnitUpdate(unit))
        });

        let category_select = pick_list(
            self.categories.clone(),
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

        let error_row = row(self
            .errors
            .iter()
            .map(|error| text!("{} ", error.to_string()).into()));

        let input_divider = rule::horizontal(constants::DIV_SIZE);
        iced::widget::container(column![entry_header, entry_row, error_row, input_divider]).into()
    }

    fn build_inventory_display(&self) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(Fill), |item: &Item| text(&item.name));
        let quantity_column = table::column(text("Quantity"), |item: &Item| {
            text!(
                "{:.0} {}",
                item.quantity.value(self.unit_system),
                item.quantity.unit(self.unit_system).to_string()
            )
        });
        let category_column = table::column(text("Category").width(200), |item: &Item| match self
            .item_category_mapping
            .get(&item.id)
        {
            Some(cat_id) => text(
                self.categories
                    .iter()
                    .find(|category| category.id == *cat_id)
                    .unwrap()
                    .name
                    .clone(),
            ),
            None => text!("None"),
        });
        //Something is wrong in the design here. Might be a misunderstanding of how to handle the edit state
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |item: &Item| match self.edit_state {
                EditState::None => {
                    let category = match self.item_category_mapping.get(&item.id) {
                        Some(id_ref) => {
                            let id = *id_ref;
                            Some(
                                self.categories
                                    .iter()
                                    .find(|category| category.id == id)
                                    .unwrap()
                                    .clone(),
                            )
                        }
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

    fn clear_inputs(&mut self) {
        self.input_name.clear();
        self.input_quantity.clear();
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
            input_name: NameInput::new("name-input", |str: String| {
                application::Message::Inventory(Message::NameUpdate(str))
            }),
            input_quantity: String::new(),
            input_unit,
            input_category: None,

            // Display Managers
            contents: Vec::new(),
            categories: Vec::new(),
            item_category_mapping: HashMap::new(),
            unit_system,

            // Input State
            edit_state: EditState::None,
            errors: HashSet::new(),
        }
    }

    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::NameUpdate(new) => {
                self.input_name.update(new);
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
            Message::BeginEdit(item, category_id) => {
                self.edit_state = EditState::Editing(item.id);
                self.input_name.update(item.name);
                self.input_quantity = item.quantity.value(self.unit_system).to_string();
                self.input_unit = item.quantity.unit(self.unit_system);
                self.input_category = category_id;
                None
            }
            Message::Save => {
                self.errors.clear();
                let name_result = self.input_name.get_output();
                let quantity_result = quantity_unload(&self.input_quantity, &self.input_unit);

                if let Err(ref e) = name_result {
                    self.errors.insert(e.clone());
                };
                if let Err(ref e) = quantity_result {
                    self.errors.insert(e.clone());
                };

                if self.errors.is_empty() {
                    self.clear_inputs();
                    let name = name_result.unwrap();
                    let quantity = quantity_result.unwrap();
                    Some(self.save(name, quantity))
                } else {
                    None
                }
            }
            Message::InventoryUpdate(items) => {
                self.contents = items;
                self.edit_state = EditState::None;
                self.errors.clear();
                self.input_name.clear();
                self.input_quantity.clear();
                self.input_category = None;
                self.input_unit = match self.unit_system {
                    UnitSystem::Metric => Unit::Milliliter,
                    UnitSystem::Imperial => Unit::FluidOunce,
                };
                None
            }
            Message::CategoryMappingUpdate(mapping) => {
                self.item_category_mapping = mapping;
                None
            }
            Message::CategoryListInitialization(categories) => {
                self.categories = categories;
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
