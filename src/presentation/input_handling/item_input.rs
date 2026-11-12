use iced::widget::row;

use crate::{
    models::{Category, Config, ItemBody, Quantity, Unit, UnitSystem},
    presentation::{
        Viewable,
        application::Message,
        input_handling::{Field, FormError, InputMessage, UnknownTarget, Validator},
        widget::input::{
            Input, InputContents, NumberInput, OptionalPickInput, RequiredPickInput, StringInput,
        },
    },
};

#[derive(Debug)]
pub struct ItemInput {
    name: StringInput<Message>,
    quantity: NumberInput<Message>,
    unit: RequiredPickInput<Unit, Message>,
    category: OptionalPickInput<Category, Message>,
}
impl ItemInput {
    pub fn new(
        config: &Config,
        msg: fn(InputMessage) -> Message,
        categories: Vec<Category>,
    ) -> Self {
        let name = StringInput::new(
            move |id, str| msg(InputMessage::String(id, str)),
            "Name".to_string(),
            String::new(),
        );
        let quantity = NumberInput::new(
            move |id, str| msg(InputMessage::String(id, str)),
            "Quantity".to_string(),
            String::new(),
        );
        let units = match config.default_units() {
            UnitSystem::Metric => Unit::Milliliter,
            UnitSystem::Imperial => Unit::FluidOunce,
        };
        let unit = RequiredPickInput::new(
            move |id, unit| msg(InputMessage::Unit(id, unit)),
            Unit::get_units(),
            Some(units),
            String::new(),
        );
        let category = OptionalPickInput::new(
            move |id, category| msg(InputMessage::Category(id, category)),
            categories,
            None,
            "Category".to_string(),
        );
        Self {
            name,
            quantity,
            unit,
            category,
        }
    }
    pub fn view(&self) -> iced::Element<'_, Message> {
        row![
            self.name.view(),
            self.quantity.view(),
            self.unit.view(),
            self.category.view()
        ]
        .into()
    }
    pub fn update(&mut self, msg: InputMessage) -> Result<(), UnknownTarget> {
        match msg {
            InputMessage::String(id, str) if &id == self.name.id() => {
                self.name.update(str);
            }
            InputMessage::String(id, str) if &id == self.quantity.id() => {
                self.quantity.update(str);
            }
            InputMessage::Unit(id, unit) if &id == self.unit.id() => {
                self.unit.update(unit);
            }
            InputMessage::Category(id, category) if &id == self.category.id() => {
                self.category.update(category);
            }
            msg => return Err(UnknownTarget(msg)),
        }
        Ok(())
    }
    pub fn output(&mut self) -> Result<(ItemBody, Option<Category>), FormError> {
        let mut validator = Validator::default();
        let name = validator.check(Field::Name, self.name.get_output());
        let quantity = validator.check(Field::Quantity, self.quantity.get_output());
        let unit = validator.check(Field::Unit, self.unit.get_output());
        let category = self.category.get_output();
        match (name, quantity, unit) {
            (Some(name), Some(quantity), Some(unit)) => {
                let quantity = Quantity::new(quantity, unit);
                self.clear();
                Ok((ItemBody { name, quantity }, category))
            }
            _ => Err(validator.into_error()),
        }
    }

    pub fn clear(&mut self) {
        self.name.clear();
        self.quantity.clear();
        self.category.clear();
    }
    pub fn begin_edit(&mut self, edit: &(ItemBody, Option<Category>), unit_system: UnitSystem) {
        self.clear();
        if let Some(ref category) = edit.1 {
            self.category.update(category.clone());
        }
        self.name.update(edit.0.name.clone());
        let quantity = edit.0.quantity.value(unit_system);
        let units = edit.0.quantity.unit(unit_system);
        self.quantity.update(quantity.to_string());
        self.unit.update(units);
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::Id;

    use super::*;
    use crate::presentation::{
        input_handling::{
            FormError,
            test_support::{category, config, msg},
        },
        widget::input::Error,
    };

    fn item_input() -> ItemInput {
        ItemInput::new(&config(), msg, vec![category(1, "Gin")])
    }

    fn set_text(input: &mut ItemInput, id: &Id, text: &str) {
        input
            .update(InputMessage::String(id.clone(), text.to_string()))
            .unwrap();
    }

    #[test]
    fn reports_every_invalid_field() {
        let mut input = item_input();
        assert_eq!(
            input.output(),
            Err(FormError(vec![
                (Field::Name, Error::StringEmpty),
                (Field::Quantity, Error::QuantityInvalid),
            ]))
        );
    }

    #[test]
    fn valid_item_is_returned_with_optional_category() {
        let mut input = item_input();
        let (name, quantity) = (input.name.id().clone(), input.quantity.id().clone());
        set_text(&mut input, &name, "Tanqueray");
        set_text(&mut input, &quantity, "750");
        let category_id = input.category.id().clone();
        input
            .update(InputMessage::Category(category_id, category(1, "Gin")))
            .unwrap();

        let (body, chosen) = input.output().unwrap();
        assert_eq!(
            body,
            ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(750.0, Unit::Milliliter),
            }
        );
        assert_eq!(chosen, Some(category(1, "Gin")));
    }

    #[test]
    fn message_for_unknown_field_is_rejected() {
        let mut input = item_input();
        let stale = InputMessage::Unit(Id::unique(), Unit::Gram);
        assert!(input.update(stale).is_err());
    }
}
