use std::{collections::HashSet, mem::take};

use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{column, container, row, text, text_input},
};

use crate::{
    logic::{self, BarCollection},
    presentation::widget::{
        sidebar::{self, button},
        text_style::title,
    },
};

pub fn run() -> iced::Result {
    iced::application(Bartend::new, Bartend::update, Bartend::view)
        .title(Bartend::title)
        .window_size((500.0, 600.0))
        .run()
}

#[derive(Debug)]
enum Screen {
    Inventory(State),
    Settings,
}

#[derive(Debug)]
struct State {
    input_name: String,
    input_quantity: String,
    errors: HashSet<StateError>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum StateError {
    NameError,
    QuantityError,
}

impl State {
    fn new() -> Self {
        Self {
            input_name: String::new(),
            input_quantity: String::new(),
            errors: HashSet::with_capacity(2),
        }
    }
}

#[derive(Debug)]
struct Bartend {
    screen: Screen,
    bar_collection: logic::BarCollection,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenInventory,
    OpenSettings,
    Inventory(InventoryMessage),
}

#[derive(Debug, Clone)]
enum InventoryMessage {
    SaveNewItem,
    NameUpdate(String),
    QuantityUpdate(String),
}

impl Bartend {
    fn new() -> Self {
        let path = "./bartend.db";
        Self {
            screen: Screen::Inventory(State::new()),
            bar_collection: BarCollection::new(path),
        }
    }

    fn title(&self) -> String {
        format!("Bartend")
    }
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match &mut self.screen {
            Screen::Inventory(state) => match message {
                Message::OpenSettings => {
                    self.screen = Screen::Settings;
                    Task::none()
                }
                Message::OpenInventory => Task::none(),
                Message::Inventory(message) => match message {
                    InventoryMessage::NameUpdate(new) => {
                        state.input_name = new;
                        Task::none()
                    }
                    InventoryMessage::QuantityUpdate(new) => {
                        state.input_quantity = new;
                        Task::none()
                    }

                    InventoryMessage::SaveNewItem => {
                        state.errors.clear();

                        if state.input_name.is_empty() {
                            state.errors.insert(StateError::NameError);
                        }
                        let quantity = state.input_quantity.parse::<f32>();
                        let quantity = match quantity {
                            Ok(quantity) => {
                                if quantity <= 0.0 {
                                    state.errors.insert(StateError::QuantityError);
                                }
                                quantity
                            }
                            Err(_) => {
                                state.errors.insert(StateError::QuantityError);
                                0.0
                            }
                        };

                        if state.errors.is_empty() {
                            self.bar_collection
                                .add_item(&take(&mut state.input_name), quantity);
                            state.input_quantity.clear();
                        }
                        Task::none()
                    }
                },
            },
            Screen::Settings => match message {
                Message::OpenInventory => {
                    self.screen = Screen::Inventory(State::new());
                    Task::none()
                }
                Message::OpenSettings => Task::none(),
                Message::Inventory(_) => unreachable!(),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = column![
            title("Sidebar"),
            sidebar::button("Inventory", || Message::OpenInventory),
            sidebar::button("Settings", || Message::OpenSettings),
        ]
        .width(300)
        .padding(10);

        let screen = match &self.screen {
            Screen::Inventory(state) => {
                let title = title("Inventory");

                let entry_header = text("New Item:");
                let name_input = text_input("Name", &state.input_name)
                    .id("name-input")
                    .on_input(|str: String| Message::Inventory(InventoryMessage::NameUpdate(str)));
                let quantity_input = text_input("Quantity", &state.input_quantity)
                    .id("quantity-input")
                    .on_input(|str: String| {
                        Message::Inventory(InventoryMessage::QuantityUpdate(str))
                    });

                let confirm_button =
                    button("Save", || Message::Inventory(InventoryMessage::SaveNewItem));
                let entry_row = row![name_input, quantity_input, confirm_button].spacing(5);

                let mut error_row = row![];
                for error in &state.errors {
                    match error {
                        StateError::NameError => {
                            error_row = error_row.push(text!("Name Must Not Be Empty"));
                        }
                        StateError::QuantityError => {
                            error_row = error_row
                                .push(text!("Quantity must be a positive, non-zero number"));
                        }
                    }
                }

                let inventory_header = text("Inventory");
                let items = self.bar_collection.get_items();
                let mut inventory = column![];
                for item in items {
                    let row = text!["{}: {} remain", item.name, item.quantity];
                    inventory = inventory.push(row);
                }

                let body = column![
                    entry_header,
                    entry_row,
                    error_row,
                    inventory_header,
                    inventory
                ];
                column![title, body]
            }
            Screen::Settings => {
                let title = title("Settings");

                column![title]
            }
            _ => todo!(),
        };
        container(
            column![row![sidebar, container(screen).padding(10).width(Fill)].spacing(10),]
                .spacing(10),
        )
        .height(Fill)
        .width(Fill)
        .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
