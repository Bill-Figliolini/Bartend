use iced::widget::row;

use crate::{
    models::{Category, Config, ItemBody, Quantity, Unit, UnitSystem},
    presentation::{
        application::Message,
        input_handling::{Field, FormError, InputMessage, UnknownTarget, Validator, route},
        widget::input::{NumberField, OptionalPick, RequiredPick, StringField},
    },
};

#[derive(Debug)]
pub struct ItemInput {
    name: StringField,
    quantity: NumberField,
    unit: RequiredPick<Unit>,
    category: OptionalPick<Category>,
    msg: fn(InputMessage) -> Message,
    /// Set once a save has been attempted; see `CategoryInput::show_errors`.
    show_errors: bool,
}
impl ItemInput {
    pub fn new(
        config: &Config,
        msg: fn(InputMessage) -> Message,
        categories: Vec<Category>,
    ) -> Self {
        let units = match config.default_units() {
            UnitSystem::Metric => Unit::Milliliter,
            UnitSystem::Imperial => Unit::FluidOunce,
        };
        Self {
            name: StringField::new("Name"),
            quantity: NumberField::new("Quantity"),
            unit: RequiredPick::new(Unit::get_units(), Some(units), ""),
            category: OptionalPick::new(categories, None, "Category"),
            msg,
            show_errors: false,
        }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> iced::Element<'_, Message> {
        let show = self.show_errors;
        row![
            self.name
                .view(route(self.msg, self.name.id(), InputMessage::String), show),
            self.quantity.view(
                route(self.msg, self.quantity.id(), InputMessage::String),
                show
            ),
            self.unit
                .view(route(self.msg, self.unit.id(), InputMessage::Unit), show),
            self.category.view(
                route(self.msg, self.category.id(), InputMessage::Category),
                show
            ),
        ]
        .into()
    }
    pub fn update(&mut self, msg: InputMessage) -> Result<(), UnknownTarget> {
        match msg {
            InputMessage::String(id, str) if &id == self.name.id() => {
                self.name.set(str);
            }
            InputMessage::String(id, str) if &id == self.quantity.id() => {
                self.quantity.set(str);
            }
            InputMessage::Unit(id, unit) if &id == self.unit.id() => {
                self.unit.select(unit);
            }
            InputMessage::Category(id, category) if &id == self.category.id() => {
                self.category.select(category);
            }
            msg => return Err(UnknownTarget(msg)),
        }
        Ok(())
    }
    pub fn output(&mut self) -> Result<(ItemBody, Option<Category>), FormError> {
        self.show_errors = true;
        let mut validator = Validator::default();
        let name = validator.check(Field::Name, self.name.value());
        let quantity = validator.check(Field::Quantity, self.quantity.value());
        let unit = validator.check(Field::Unit, self.unit.value());
        let Ok(category) = self.category.value();
        match (name, quantity, unit) {
            (Some(name), Some(quantity), Some(unit)) => {
                let quantity = Quantity::new(quantity, unit);
                self.clear();
                Ok((ItemBody { name, quantity }, category))
            }
            _ => Err(validator.into_error()),
        }
    }

    /// Keeps the chosen unit, so entering several items in the same unit
    /// does not mean picking it every time.
    pub fn clear(&mut self) {
        self.name.clear();
        self.quantity.clear();
        self.category.clear();
        self.show_errors = false;
    }
    pub fn begin_edit(&mut self, edit: &(ItemBody, Option<Category>), unit_system: UnitSystem) {
        self.clear();
        if let Some(ref category) = edit.1 {
            self.category.select(category.clone());
        }
        self.name.set(edit.0.name.clone());
        let quantity = edit.0.quantity.value(unit_system);
        let units = edit.0.quantity.unit(unit_system);
        self.quantity.set(quantity.to_string());
        self.unit.select(units);
    }
    #[cfg(test)]
    pub fn name_id(&self) -> iced::widget::Id {
        self.name.id().clone()
    }
    #[cfg(test)]
    pub fn quantity_id(&self) -> iced::widget::Id {
        self.quantity.id().clone()
    }
    #[cfg(test)]
    pub fn category_id(&self) -> iced::widget::Id {
        self.category.id().clone()
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
    fn clearing_keeps_the_chosen_unit() {
        let mut input = item_input();
        let unit = input.unit.id().clone();
        input.update(InputMessage::Unit(unit, Unit::Dash)).unwrap();
        input.output().unwrap_err();
        input.clear();
        assert!(!input.show_errors);
        assert_eq!(input.unit.value(), Ok(Unit::Dash));
    }

    #[test]
    fn message_for_unknown_field_is_rejected() {
        let mut input = item_input();
        let stale = InputMessage::Unit(Id::unique(), Unit::Gram);
        assert!(input.update(stale).is_err());
    }

    #[test]
    fn editing_an_item_round_trips_in_either_unit_system() {
        let edit = (
            ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(750.0, Unit::Milliliter),
            },
            Some(category(1, "Gin")),
        );
        for system in [UnitSystem::Metric, UnitSystem::Imperial] {
            let mut input = item_input();
            input.begin_edit(&edit, system);
            let (body, chosen) = input.output().unwrap();
            assert_eq!(body.name, "Tanqueray");
            assert!((body.quantity.value(UnitSystem::Metric) - 750.0).abs() < 0.01);
            assert_eq!(chosen, Some(category(1, "Gin")));
        }
    }

    #[test]
    fn chosen_unit_is_used() {
        let mut input = item_input();
        let (name, quantity, unit) = (
            input.name.id().clone(),
            input.quantity.id().clone(),
            input.unit.id().clone(),
        );
        set_text(&mut input, &name, "Angostura");
        set_text(&mut input, &quantity, "3");
        input.update(InputMessage::Unit(unit, Unit::Dash)).unwrap();

        let (body, _) = input.output().unwrap();
        assert_eq!(body.quantity, Quantity::new(3.0, Unit::Dash));
    }

    #[test]
    fn imperial_config_defaults_to_fluid_ounces() {
        let mut config = config().editable();
        config.default_unit_system = UnitSystem::Imperial;
        let input = ItemInput::new(&config.commit(), msg, Vec::new());
        assert_eq!(input.unit.value(), Ok(Unit::FluidOunce));
    }
}
