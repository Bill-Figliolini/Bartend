use std::collections::HashSet;

use iced::widget::{column, row, text};

use crate::{
    logic::{
        category::Category,
        config::Config,
        item::ItemBody,
        quantity::{Quantity, Unit, UnitSystem},
    },
    presentation::{
        Viewable,
        application::Message,
        input_handling::{InputCollection, InputMessage},
        widget::input::{
            Error, Input, InputContents, InputOptionalContents,
            pick_input::{optional::OptionalPickInput, required::RequiredPickInput},
            text_input::{number_input::NumberInput, string_input::StringInput},
        },
    },
};

#[derive(Debug)]
pub struct ItemInput {
    name_input: StringInput<Message>,
    quantity_input: NumberInput<Message>,
    unit_input: RequiredPickInput<Unit, Message>,
    category_input: OptionalPickInput<Category, Message>,

    errors: HashSet<Error>,
}
impl Viewable<Message> for ItemInput {
    fn view(&self) -> iced::Element<'_, Message> {
        column![
            row![
                self.name_input.view(),
                self.quantity_input.view(),
                self.unit_input.view(),
                self.category_input.view()
            ],
            row(self
                .errors
                .iter()
                .map(|e| text!("{} ", e.to_string()).into()))
        ]
        .into()
    }
}
impl ItemInput {
    pub fn new(
        config: &Config,
        msg: fn(InputMessage) -> Message,
        categories: Vec<Category>,
    ) -> Self {
        let name_input = StringInput::new(
            move |id, str| msg(InputMessage::String(id, str)),
            "Name".to_string(),
            String::new(),
        );
        let quantity_input = NumberInput::new(
            move |id, str| msg(InputMessage::String(id, str)),
            "Quantity".to_string(),
            String::new(),
        );
        let units = match config.default_units() {
            UnitSystem::Metric => Unit::Milliliter,
            UnitSystem::Imperial => Unit::FluidOunce,
        };
        let unit_input = RequiredPickInput::new(
            move |id, unit| msg(InputMessage::Unit(id, unit)),
            Unit::get_units(),
            Some(units),
        );
        let category_input = OptionalPickInput::new(
            move |id, category| msg(InputMessage::Category(id, category)),
            categories,
            None,
        );
        Self {
            name_input,
            quantity_input,
            unit_input,
            category_input,
            errors: HashSet::new(),
        }
    }
}

impl InputCollection<(ItemBody, Option<Category>)> for ItemInput {
    fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::String(id, str) if &id == self.name_input.id() => {
                self.name_input.update(str)
            }
            InputMessage::String(id, str) if &id == self.quantity_input.id() => {
                self.quantity_input.update(str);
            }
            InputMessage::Unit(id, unit) if &id == self.unit_input.id() => {
                self.unit_input.update(unit);
            }
            InputMessage::Category(id, category) if &id == self.category_input.id() => {
                self.category_input.update(category);
            }
            _ => unreachable!("Item Input has recieved invalid mesage {msg:?}"),
        }
    }
    fn output(&mut self) -> Result<(ItemBody, Option<Category>), ()> {
        let name_result = self.name_input.get_output();
        let quantity_result = self.quantity_input.get_output();
        let units = self
            .unit_input
            .get_output()
            .expect("Unit Input starts initialized, and is therefore infalible here");
        let category = self
            .category_input
            .get_output()
            .expect("Optional Pickers are infalible");
        if !self.name_input.has_error() && !self.quantity_input.has_error() {
            let name = name_result.unwrap();
            let quantity = Quantity::new(quantity_result.unwrap(), units);
            self.clear();
            Ok((ItemBody { name, quantity }, category))
        } else {
            Err(())
        }
    }

    fn begin_edit(&mut self, edit: (ItemBody, Option<Category>), unit_system: UnitSystem) {
        self.clear();
        if let Some(category) = edit.1 {
            self.category_input.update(category);
        }
        self.name_input.update(edit.0.name);
        let quantity = edit.0.quantity.value(unit_system);
        let units = edit.0.quantity.unit(unit_system);
        self.quantity_input.update(quantity.to_string());
        self.unit_input.update(units);
    }
    fn clear(&mut self) {
        self.name_input.clear();
        self.quantity_input.clear();
        self.category_input.clear();
    }
}
