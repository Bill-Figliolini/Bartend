use std::fmt::Display;

use iced::{Element, widget::row};

use crate::{
    models::{Category, Config, ItemBody, Quantity, Unit, UnitSystem},
    presentation::{
        input_handling::{FormError, Validator},
        widget::input::{NumberField, OptionalPick, RequiredPick, StringField},
    },
};

/// An edit to one field of an [`ItemInput`].
#[derive(Debug, Clone)]
pub enum ItemMessage {
    Name(String),
    Quantity(String),
    Unit(Unit),
    Category(Category),
}

/// A field of an [`ItemInput`], as named in its [`FormError`]. The category
/// is optional, so it never fails and has no entry here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemField {
    Name,
    Quantity,
    Unit,
}

impl Display for ItemField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ItemField::Name => "Name",
            ItemField::Quantity => "Quantity",
            ItemField::Unit => "Unit",
        };
        write!(f, "{text}")
    }
}

/// The form for adding or editing an inventory item, along with the
/// category it belongs to, if any.
#[derive(Debug)]
pub struct ItemInput {
    name: StringField,
    quantity: NumberField,
    unit: RequiredPick<Unit>,
    category: OptionalPick<Category>,
    /// Whether fields show their errors; see the [module docs](super).
    show_errors: bool,
}
impl ItemInput {
    /// The unit starts as the configured unit system's volume unit (ml or
    /// fl oz); `categories` are the ones offered for the item.
    pub fn new(config: &Config, categories: Vec<Category>) -> Self {
        let units = match config.default_units() {
            UnitSystem::Metric => Unit::Milliliter,
            UnitSystem::Imperial => Unit::FluidOunce,
        };
        Self {
            name: StringField::new("Name"),
            quantity: NumberField::new("Quantity"),
            unit: RequiredPick::new(Unit::get_units(), Some(units), ""),
            category: OptionalPick::new(categories, None, "Category"),
            show_errors: false,
        }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> Element<'_, ItemMessage> {
        let show = self.show_errors;
        row![
            self.name.view(ItemMessage::Name, show),
            self.quantity.view(ItemMessage::Quantity, show),
            self.unit.view(ItemMessage::Unit, show),
            self.category.view(ItemMessage::Category, show),
        ]
        .into()
    }
    pub fn update(&mut self, msg: ItemMessage) {
        match msg {
            ItemMessage::Name(text) => self.name.set(text),
            ItemMessage::Quantity(text) => self.quantity.set(text),
            ItemMessage::Unit(unit) => self.unit.select(unit),
            ItemMessage::Category(category) => self.category.select(category),
        }
    }
    /// The item, and the category chosen for it if any.
    pub fn output(&mut self) -> Result<(ItemBody, Option<Category>), FormError<ItemField>> {
        self.show_errors = true;
        let mut validator = Validator::default();
        let name = validator.check(ItemField::Name, self.name.value());
        let quantity = validator.check(ItemField::Quantity, self.quantity.value());
        let unit = validator.check(ItemField::Unit, self.unit.value());
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
    /// Fills the form from `edit`, showing its quantity in `unit_system`.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{
        input_handling::{
            FormError,
            test_support::{category, config},
        },
        widget::input::Error,
    };

    fn item_input() -> ItemInput {
        ItemInput::new(&config(), vec![category(1, "Gin")])
    }

    #[test]
    fn reports_every_invalid_field() {
        let mut input = item_input();
        assert_eq!(
            input.output(),
            Err(FormError(vec![
                (ItemField::Name, Error::StringEmpty),
                (ItemField::Quantity, Error::QuantityInvalid),
            ]))
        );
    }

    #[test]
    fn valid_item_is_returned_with_optional_category() {
        let mut input = item_input();
        input.update(ItemMessage::Name("Tanqueray".to_string()));
        input.update(ItemMessage::Quantity("750".to_string()));
        input.update(ItemMessage::Category(category(1, "Gin")));

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
    fn fields_display_their_names() {
        assert_eq!(ItemField::Name.to_string(), "Name");
        assert_eq!(ItemField::Quantity.to_string(), "Quantity");
        assert_eq!(ItemField::Unit.to_string(), "Unit");
    }

    #[test]
    fn clearing_keeps_the_chosen_unit() {
        let mut input = item_input();
        input.update(ItemMessage::Unit(Unit::Dash));
        input.output().unwrap_err();
        input.clear();
        assert!(!input.show_errors);
        assert_eq!(input.unit.value(), Ok(Unit::Dash));
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
        input.update(ItemMessage::Name("Angostura".to_string()));
        input.update(ItemMessage::Quantity("3".to_string()));
        input.update(ItemMessage::Unit(Unit::Dash));

        let (body, _) = input.output().unwrap();
        assert_eq!(body.quantity, Quantity::new(3.0, Unit::Dash));
    }

    #[test]
    fn imperial_config_defaults_to_fluid_ounces() {
        let mut config = config().editable();
        config.default_unit_system = UnitSystem::Imperial;
        let input = ItemInput::new(&config.commit(), Vec::new());
        assert_eq!(input.unit.value(), Ok(Unit::FluidOunce));
    }
}
