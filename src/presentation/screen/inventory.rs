use std::{collections::HashSet, mem::take};

use iced::{
    Element, Theme,
    widget::{column, row, table, text, text_input},
};

use crate::{
    persistence::{Item, ItemID},
    presentation::{
        application,
        widget::{sidebar::button, text_style::title},
    },
};

#[derive(Debug)]
pub struct Inventory {
    input_name: String,
    input_quantity: String,
    contents: Vec<Item>,
    edit_state: EditState,
    errors: HashSet<Error>,
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
    SaveNewItem,
    BeginEdit(Item),
    NameUpdate(String),
    QuantityUpdate(String),
}
impl Inventory {
    pub fn new(item_list: Vec<Item>) -> Self {
        Self {
            input_name: String::new(),
            input_quantity: String::new(),
            contents: item_list,
            edit_state: EditState::None,
            errors: HashSet::with_capacity(2),
        }
    }
    pub(super) fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::NameUpdate(new) => {
                self.input_name = new;
                None
            }
            Message::QuantityUpdate(new) => {
                self.input_quantity = new;
                None
            }
            Message::BeginEdit(item) => {
                self.edit_state = EditState::Editing(item.id);
                self.input_name = item.name;
                self.input_quantity = item.quantity.to_string();
                None
            }
            Message::SaveNewItem => {
                self.errors.clear();

                if self.input_name.is_empty() {
                    self.errors.insert(Error::NameError);
                }
                let quantity = self.input_quantity.trim().parse::<f32>();
                let quantity = match quantity {
                    Ok(quantity) => {
                        if quantity <= 0.0 {
                            self.errors.insert(Error::QuantityError);
                        }
                        quantity
                    }
                    Err(_) => {
                        self.errors.insert(Error::QuantityError);
                        0.0
                    }
                };

                if self.errors.is_empty() {
                    let name = take(&mut self.input_name);
                    match self.edit_state {
                        EditState::None => Some(application::Command::AddItem(name, quantity)),
                        EditState::Editing(item_id) => {
                            Some(application::Command::UpdateItem(Item {
                                id: item_id,
                                name,
                                quantity,
                            }))
                        }
                    }
                } else {
                    None
                }
            }
        }
    }
    pub(super) fn view(&self) -> Element<'_, application::Message> {
        let title = title("Inventory");

        let entry_header = text("New Item:");
        let name_input = text_input("Name", &self.input_name)
            .id("name-input")
            .on_input(|str: String| application::Message::Inventory(Message::NameUpdate(str)));
        let quantity_input = text_input("Quantity", &self.input_quantity)
            .id("quantity-input")
            .on_input(|str: String| application::Message::Inventory(Message::QuantityUpdate(str)));

        let confirm_button = button("Save", || {
            application::Message::Inventory(Message::SaveNewItem)
        });
        let entry_row = row![name_input, quantity_input, confirm_button].spacing(5);

        let mut error_row = row![];
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

        let name_column = table::column(text("Name"), |item: &Item| text(&item.name));
        let quantity_column = table::column(text("Quantity"), |item: &Item| text(&item.quantity));
        //Something is wrong in the design here. Might be a misunderstanding of how to handle the edit state
        let edit_column = table::column(text("Edit").width(50), |item: &Item| {
            match self.edit_state {
                EditState::None => button("Edit", || {
                    application::Message::Inventory(Message::BeginEdit(item.clone()))
                }),
                EditState::Editing(item_id) if item.id == item_id => {
                    button("Cancel", || application::Message::RefreshItems)
                }
                EditState::Editing(_) => text("Edit").into(),
            }
        });
        let delete_column = table::column(text("Delete").width(50), |item: &Item| {
            button("X", || application::Message::DeleteItem(item.id.clone()))
        });
        let columns = vec![name_column, quantity_column, edit_column, delete_column];
        let inventory = table(columns, &self.contents);

        let body = column![entry_header, entry_row, error_row, inventory];
        column![title, body].into()
    }
}
