use crate::{
    logic::quantity::{Quantity, Unit},
    presentation::widget::input::{Input, PickInputUpdate, StringInputUpdate},
};

use super::Error;

pub struct QuantityInput {
    id: String,
    input_quantity: String,
    input_unit: Unit,
    units: Vec<Unit>,
    
}
impl<'a> Input<'a, Quantity> for QuantityInput {
    fn new<F: Fn(String) -> crate::presentation::application::Message + 'static>(
        id: &str,
        on_input: F,
    ) -> Self {
        Self {
            id: id.to_string(),
            input_quantity: String::new(),
            input_unit: Unit::Milliliter,
            units: Unit::get_units(),
        }
    }

    fn display(&self) -> iced::Element<'a, crate::presentation::application::Message> {
        todo!()
    }

    fn get_output(&self) -> Result<Quantity, Error> {
        let unvalidated_quantity = self.input_quantity.trim().parse::<f32>();
        let quantity = match unvalidated_quantity {
            Ok(quantity) if quantity > 0.0 => quantity,
            _ => return Err(Error::QuantityInvalid),
        };
        Ok(Quantity::new(quantity, self.input_unit))
    }

    fn clear(&mut self) {
        self.input_quantity.clear();
    }
}

impl StringInputUpdate for QuantityInput {
    fn string_update(&mut self, input: String) {
        self.input_quantity = input;
    }
}
impl PickInputUpdate<Unit> for QuantityInput {
    fn pick_update(&mut self, input: Unit) {
        self.input_unit = input;
    }
}
