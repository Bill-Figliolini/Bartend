use std::fmt::Display;

use iced::{
    Element,
    widget::{column, row},
};

use crate::{
    models::{Category, Config, ItemBody, Quantity, Unit, UnitSystem},
    presentation::{
        constants,
        input_handling::{Form, FormError, Validator},
        styling::Styling,
        widget::{
            detail::labeled,
            input::{NumberField, OptionalPick, OptionalPositiveField, RequiredPick, StringField},
        },
    },
};

/// An edit to one field of an [`ItemInput`].
#[derive(Debug, Clone)]
pub enum ItemMessage {
    Name(String),
    Quantity(String),
    Unit(Unit),
    FullSize(String),
    Category(Category),
}

/// A field of an [`ItemInput`], as named in its [`FormError`]. The category
/// is optional, so it never fails and has no entry here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemField {
    Name,
    Quantity,
    Unit,
    FullSize,
}

impl Display for ItemField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ItemField::Name => "Name",
            ItemField::Quantity => "Quantity",
            ItemField::Unit => "Unit",
            ItemField::FullSize => "Full size",
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
    /// How much a full one holds, in the same unit as the quantity. May be
    /// left blank.
    full_size: OptionalPositiveField,
    category: OptionalPick<Category>,
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
            name: StringField::new(""),
            quantity: NumberField::new(""),
            unit: RequiredPick::new(Unit::get_units(), Some(units), ""),
            full_size: OptionalPositiveField::new("Optional"),
            category: OptionalPick::new(categories, None, ""),
        }
    }
}

impl Form for ItemInput {
    type Message = ItemMessage;
    type Field = ItemField;
    /// The item, and the category chosen for it if any.
    type Output = (ItemBody, Option<Category>);
    type Value = (ItemBody, Option<Category>);

    fn update(&mut self, msg: ItemMessage) {
        match msg {
            ItemMessage::Name(text) => self.name.set(text),
            ItemMessage::Quantity(text) => self.quantity.set(text),
            ItemMessage::Unit(unit) => self.unit.select(unit),
            ItemMessage::FullSize(text) => self.full_size.set(text),
            ItemMessage::Category(category) => self.category.select(category),
        }
    }

    fn validate(&self) -> Result<(ItemBody, Option<Category>), FormError<ItemField>> {
        let mut validator = Validator::default();
        let name = validator.check(ItemField::Name, self.name.value());
        let quantity = validator.check(ItemField::Quantity, self.quantity.value());
        let unit = validator.check(ItemField::Unit, self.unit.value());
        let full_size = validator.check(ItemField::FullSize, self.full_size.value());
        let Ok(category) = self.category.value();
        match (name, quantity, unit, full_size) {
            (Some(name), Some(quantity), Some(unit), Some(full_size)) => {
                // Entered in the quantity's unit, so always the same kind.
                let body = ItemBody {
                    name,
                    quantity: Quantity::new(quantity, unit),
                    full_size: full_size.map(|full_size| Quantity::new(full_size, unit)),
                };
                Ok((body, category))
            }
            _ => Err(validator.into_error()),
        }
    }

    /// Keeps the chosen unit, so entering several items in the same unit
    /// does not mean picking it every time.
    fn clear(&mut self) {
        self.name.clear();
        self.quantity.clear();
        self.full_size.clear();
        self.category.clear();
    }

    /// Shows the item's quantity, and full size if it has one, in `units`.
    fn fill(&mut self, (item, category): &(ItemBody, Option<Category>), units: UnitSystem) {
        if let Some(category) = category {
            self.category.select(category.clone());
        }
        self.name.set(item.name.clone());
        self.quantity.set(item.quantity.value(units).to_string());
        self.unit.select(item.quantity.unit(units));
        // A full size of another kind than the stock can't be shown in the
        // quantity's unit; the services never save one.
        let full_size = item
            .full_size
            .filter(|full_size| full_size.same_kind(item.quantity))
            .map(|full_size| full_size.value(units).to_string());
        self.full_size.set(full_size.unwrap_or_default());
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    /// Each field under its label: the name, then the amount in stock with
    /// its unit and the full size, then the category.
    fn view(&self, styling: &Styling, show_errors: bool) -> Element<'_, ItemMessage> {
        let full_size_label = match self.unit.value() {
            Ok(unit) => format!("Full size ({unit})"),
            Err(_) => "Full size".to_string(),
        };
        let amounts = row![
            labeled(
                styling,
                "In stock",
                self.quantity
                    .view(styling, ItemMessage::Quantity, show_errors)
            ),
            labeled(
                styling,
                "Unit",
                self.unit.view(styling, ItemMessage::Unit, show_errors)
            ),
            labeled(
                styling,
                &full_size_label,
                self.full_size
                    .view(styling, ItemMessage::FullSize, show_errors)
            ),
        ]
        .spacing(constants::SPACING);
        column![
            labeled(
                styling,
                "Name",
                self.name.view(styling, ItemMessage::Name, show_errors)
            ),
            amounts,
            labeled(
                styling,
                "Category",
                self.category
                    .view(styling, ItemMessage::Category, show_errors)
            ),
        ]
        .spacing(constants::SPACING)
        .into()
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
        let input = item_input();
        assert_eq!(
            input.validate(),
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

        let (body, chosen) = input.validate().unwrap();
        assert_eq!(
            body,
            ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(750.0, Unit::Milliliter),
                full_size: None,
            }
        );
        assert_eq!(chosen, Some(category(1, "Gin")));
    }

    #[test]
    fn fields_display_their_names() {
        assert_eq!(ItemField::Name.to_string(), "Name");
        assert_eq!(ItemField::Quantity.to_string(), "Quantity");
        assert_eq!(ItemField::Unit.to_string(), "Unit");
        assert_eq!(ItemField::FullSize.to_string(), "Full size");
    }

    #[test]
    fn clearing_keeps_the_chosen_unit() {
        let mut input = item_input();
        input.update(ItemMessage::Unit(Unit::Dash));
        input.update(ItemMessage::Name("Angostura".to_string()));
        input.clear();
        let errors = input.validate().unwrap_err();
        assert_eq!(errors.error_for(&ItemField::Name), Some(Error::StringEmpty));
        assert_eq!(input.unit.value(), Ok(Unit::Dash));
    }

    #[test]
    fn editing_an_item_round_trips_in_either_unit_system() {
        let edit = (
            ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(750.0, Unit::Milliliter),
                full_size: None,
            },
            Some(category(1, "Gin")),
        );
        for system in [UnitSystem::Metric, UnitSystem::Imperial] {
            let mut input = item_input();
            input.fill(&edit, system);
            let (body, chosen) = input.validate().unwrap();
            assert_eq!(body.name, "Tanqueray");
            assert!((body.quantity.value(UnitSystem::Metric) - 750.0).abs() < 0.01);
            assert_eq!(chosen, Some(category(1, "Gin")));
        }
    }

    #[test]
    fn editing_shows_the_full_size_and_clearing_empties_it() {
        let full_size = Some(Quantity::new(750.0, Unit::Milliliter));
        let edit = (
            ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(300.0, Unit::Milliliter),
                full_size,
            },
            None,
        );
        let mut input = item_input();
        input.fill(&edit, UnitSystem::Metric);
        assert_eq!(input.full_size.text(), "750");
        assert_eq!(input.validate().unwrap().0.full_size, full_size);

        input.clear();
        input.update(ItemMessage::Name("Beefeater".to_string()));
        input.update(ItemMessage::Quantity("700".to_string()));
        assert_eq!(input.full_size.text(), "");
        assert_eq!(input.validate().unwrap().0.full_size, None);
    }

    #[test]
    fn full_size_round_trips_in_either_unit_system() {
        let edit = (
            ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(300.0, Unit::Milliliter),
                full_size: Some(Quantity::new(750.0, Unit::Milliliter)),
            },
            None,
        );
        for system in [UnitSystem::Metric, UnitSystem::Imperial] {
            let mut input = item_input();
            input.fill(&edit, system);
            let full_size = input.validate().unwrap().0.full_size.unwrap();
            assert!(
                (full_size.value(UnitSystem::Metric) - 750.0).abs() < 0.01,
                "{system:?}: {full_size:?}"
            );
        }
    }

    #[test]
    fn full_size_is_in_the_quantitys_unit() {
        let mut input = item_input();
        input.update(ItemMessage::Name("Sugar".to_string()));
        input.update(ItemMessage::Quantity("200".to_string()));
        input.update(ItemMessage::Unit(Unit::Gram));
        input.update(ItemMessage::FullSize("1000".to_string()));

        let (body, _) = input.validate().unwrap();
        assert_eq!(body.full_size, Some(Quantity::new(1000.0, Unit::Gram)));
        assert!(body.full_size_fits());
    }

    #[test]
    fn an_invalid_full_size_is_reported() {
        let mut input = item_input();
        input.update(ItemMessage::Name("Tanqueray".to_string()));
        input.update(ItemMessage::Quantity("700".to_string()));
        input.update(ItemMessage::FullSize("0".to_string()));

        assert_eq!(
            input.validate(),
            Err(FormError(vec![(
                ItemField::FullSize,
                Error::NumberNotPositive
            )]))
        );
    }

    #[test]
    fn a_full_size_of_another_kind_is_not_shown() {
        let edit = (
            ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(300.0, Unit::Milliliter),
                full_size: Some(Quantity::new(750.0, Unit::Gram)),
            },
            None,
        );
        let mut input = item_input();
        input.fill(&edit, UnitSystem::Metric);
        assert_eq!(input.full_size.text(), "");
    }

    #[test]
    fn chosen_unit_is_used() {
        let mut input = item_input();
        input.update(ItemMessage::Name("Angostura".to_string()));
        input.update(ItemMessage::Quantity("3".to_string()));
        input.update(ItemMessage::Unit(Unit::Dash));

        let (body, _) = input.validate().unwrap();
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
