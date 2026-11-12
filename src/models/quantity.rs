//! # Quantity
//! Handles preservation of quantities from the Presentation layer down to the DB
//! ### Invariants
//! Internally, all quantities are represented in Metric units, for consistency and ease of conversion.
//! ### Behavior
//! Quantities allow of ease of conversion between Imperial and Metric, both at creation and later access.
//! Quantities handle type checking, guaranteeing that inconsistent operations like adding a liquid and a mass do not occur.

use std::{fmt::Display, ops::SubAssign};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum Quantity {
    Volume { quantity: f32 },
    Mass { quantity: f32 },
    Count { quantity: f32, name: CountName },
}

const IMPERIAL_CONVERSION_VOLUME: f32 = 29.57;
const IMPERIAL_CONVERSION_MASS: f32 = 28.35;

impl Quantity {
    #[must_use]
    pub fn new(quantity: f32, unit: Unit) -> Self {
        match unit {
            Unit::Milliliter => Self::Volume { quantity },
            Unit::FluidOunce => Self::Volume {
                quantity: quantity * IMPERIAL_CONVERSION_VOLUME,
            },
            Unit::Gram => Self::Mass { quantity },
            Unit::MassOunce => Self::Mass {
                quantity: quantity * IMPERIAL_CONVERSION_MASS,
            },
            Unit::Dash => Self::Count {
                quantity,
                name: CountName::Dash,
            },
        }
    }
    #[must_use]
    pub const fn unit(&self, unit_system: UnitSystem) -> Unit {
        match unit_system {
            UnitSystem::Metric => match self {
                Self::Volume { quantity: _ } => Unit::Milliliter,
                Self::Mass { quantity: _ } => Unit::Gram,
                Self::Count { quantity: _, name } => match name {
                    CountName::Dash => Unit::Dash,
                },
            },
            UnitSystem::Imperial => match self {
                Self::Volume { quantity: _ } => Unit::FluidOunce,
                Self::Mass { quantity: _ } => Unit::MassOunce,
                Self::Count { quantity: _, name } => match name {
                    CountName::Dash => Unit::Dash,
                },
            },
        }
    }
    #[must_use]
    pub const fn value(&self, unit_system: UnitSystem) -> f32 {
        match unit_system {
            UnitSystem::Metric => match self {
                Self::Volume { quantity }
                | Self::Mass { quantity }
                | Self::Count { quantity, name: _ } => *quantity,
            },
            UnitSystem::Imperial => match self {
                Self::Volume { quantity } => *quantity / IMPERIAL_CONVERSION_VOLUME,
                Self::Mass { quantity } => {
                    //grams to oz
                    *quantity / IMPERIAL_CONVERSION_MASS
                }
                Self::Count { quantity, name: _ } => *quantity,
            },
        }
    }
    #[must_use]
    pub const fn db_format(&self) -> (f32, i32) {
        match self {
            Self::Volume { quantity } => (*quantity, 0),
            Self::Mass { quantity } => (*quantity, 1),
            Self::Count { quantity, name } => match name {
                CountName::Dash => (*quantity, 2),
            },
        }
    }
    #[must_use]
    pub fn from_db(unit_type: i32, quantity: f32) -> Self {
        match unit_type {
            0 => Self::Volume { quantity },
            1 => Self::Mass { quantity },
            2 => Self::Count {
                quantity,
                name: CountName::Dash,
            },
            _ => {
                eprintln!("Error reading quantity from DB, falling back to volume");
                Self::Volume { quantity }
            }
        }
    }

    #[must_use]
    pub fn readable(&self, unit_system: UnitSystem) -> String {
        format!("{} {}", self.value(unit_system), self.unit(unit_system))
    }
}

impl PartialEq for Quantity {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Volume {
                    quantity: l_quantity,
                },
                Self::Volume {
                    quantity: r_quantity,
                },
            )
            | (
                Self::Mass {
                    quantity: l_quantity,
                },
                Self::Mass {
                    quantity: r_quantity,
                },
            ) => l_quantity == r_quantity,
            (
                Self::Count {
                    quantity: l_quantity,
                    name: l_name,
                },
                Self::Count {
                    quantity: r_quantity,
                    name: r_name,
                },
            ) => l_name == r_name && l_quantity == r_quantity,
            _ => false,
        }
    }
}

impl PartialOrd for Quantity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (
                Self::Volume {
                    quantity: l_quantity,
                },
                Self::Volume {
                    quantity: r_quantity,
                },
            )
            | (
                Self::Mass {
                    quantity: l_quantity,
                },
                Self::Mass {
                    quantity: r_quantity,
                },
            ) => l_quantity.partial_cmp(r_quantity),
            (
                Self::Count {
                    quantity: l_quantity,
                    name: l_name,
                },
                Self::Count {
                    quantity: r_quantity,
                    name: r_name,
                },
            ) if l_name == r_name => l_quantity.partial_cmp(r_quantity),
            _ => None,
        }
    }
}

impl SubAssign for Quantity {
    fn sub_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (
                Self::Volume {
                    quantity: l_quantity,
                },
                Self::Volume {
                    quantity: r_quantity,
                },
            )
            | (
                Self::Mass {
                    quantity: l_quantity,
                },
                Self::Mass {
                    quantity: r_quantity,
                },
            ) => {
                *l_quantity -= r_quantity;
                *l_quantity = l_quantity.clamp(0.0, f32::INFINITY);
            }
            (
                Self::Count {
                    quantity: l_quantity,
                    name: l_name,
                },
                Self::Count {
                    quantity: r_quantity,
                    name: r_name,
                },
            ) if *l_name == r_name => {
                *l_quantity -= r_quantity;
                *l_quantity = l_quantity.clamp(0.0, f32::INFINITY);
            }
            _ => {
                debug_assert!(false, "Attempted to subtract mismatched values");
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountName {
    Dash,
}

//CONSIDERATION:
// Are these really needed? I could roll them into the Quantity class. But then I would need to come up with another way to represent them inside Quantities
// But that would create its own issues with regards to checking if a quantity is the same type. Perhaps the same pattern as counts could be used?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Milliliter,
    FluidOunce,
    Gram,
    MassOunce,
    Dash,
}

impl Unit {
    #[must_use]
    pub fn get_units() -> Vec<Unit> {
        vec![
            Unit::Milliliter,
            Unit::FluidOunce,
            Unit::Gram,
            Unit::MassOunce,
            Unit::Dash,
        ]
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Milliliter => write!(f, "ml"),
            Self::FluidOunce => write!(f, "fl oz"),
            Self::Gram => write!(f, "g"),
            Self::MassOunce => write!(f, "oz"),
            Self::Dash => write!(f, "dash"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitSystem {
    Metric,
    Imperial,
}

impl UnitSystem {
    pub const fn swap(&mut self) {
        *self = match self {
            Self::Metric => Self::Imperial,
            Self::Imperial => Self::Metric,
        };
    }
    #[must_use]
    pub const fn default_units(&self) -> Unit {
        match self {
            UnitSystem::Metric => Unit::Milliliter,
            UnitSystem::Imperial => Unit::FluidOunce,
        }
    }
}

impl Display for UnitSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Metric => "Metric (ml)",
            Self::Imperial => "Imperial (Oz)",
        };
        write!(f, "{text}")
    }
}
#[cfg(test)]
mod test {
    use super::*;
    mod measure {
        use super::*;
        mod count {
            use super::*;
            #[test]
            fn metric_quantity_does_not_alter_counts() {
                let quantity = 2.0;
                let count = Quantity::Count {
                    quantity,
                    name: CountName::Dash,
                };

                let count_as_metric = count.value(UnitSystem::Metric);

                assert!((quantity - count_as_metric).abs() < f32::EPSILON);
            }
            #[test]
            fn imperial_quantity_does_not_alter_counts() {
                let quantity = 2.0;
                let count = Quantity::Count {
                    quantity,
                    name: CountName::Dash,
                };

                let count_as_imperial = count.value(UnitSystem::Imperial);

                assert!((quantity - count_as_imperial).abs() < f32::EPSILON);
            }
        }
        mod volume {}
        mod mass {}
    }

    /// Compares by bit pattern, avoiding `float_cmp`.
    fn bits(value: f32) -> u32 {
        value.to_bits()
    }

    #[test]
    fn imperial_units_are_stored_as_metric() {
        let cases = [
            (Unit::Milliliter, Quantity::Volume { quantity: 2.0 }),
            (
                Unit::FluidOunce,
                Quantity::Volume {
                    quantity: 2.0 * IMPERIAL_CONVERSION_VOLUME,
                },
            ),
            (Unit::Gram, Quantity::Mass { quantity: 2.0 }),
            (
                Unit::MassOunce,
                Quantity::Mass {
                    quantity: 2.0 * IMPERIAL_CONVERSION_MASS,
                },
            ),
            (
                Unit::Dash,
                Quantity::Count {
                    quantity: 2.0,
                    name: CountName::Dash,
                },
            ),
        ];
        for (unit, expected) in cases {
            assert_eq!(Quantity::new(2.0, unit), expected, "{unit}");
        }
    }

    #[test]
    fn units_follow_the_unit_system() {
        let volume = Quantity::new(1.0, Unit::Milliliter);
        let mass = Quantity::new(1.0, Unit::Gram);
        let dash = Quantity::new(1.0, Unit::Dash);
        assert_eq!(volume.unit(UnitSystem::Metric), Unit::Milliliter);
        assert_eq!(volume.unit(UnitSystem::Imperial), Unit::FluidOunce);
        assert_eq!(mass.unit(UnitSystem::Metric), Unit::Gram);
        assert_eq!(mass.unit(UnitSystem::Imperial), Unit::MassOunce);
        assert_eq!(dash.unit(UnitSystem::Metric), Unit::Dash);
        assert_eq!(dash.unit(UnitSystem::Imperial), Unit::Dash);
    }

    #[test]
    fn values_convert_back_to_the_unit_entered() {
        let volume = Quantity::new(2.0, Unit::FluidOunce);
        let mass = Quantity::new(2.0, Unit::MassOunce);
        assert!((volume.value(UnitSystem::Imperial) - 2.0).abs() < 1e-5);
        assert!((mass.value(UnitSystem::Imperial) - 2.0).abs() < 1e-5);
        assert_eq!(
            bits(mass.value(UnitSystem::Metric)),
            bits(2.0 * IMPERIAL_CONVERSION_MASS)
        );
    }

    #[test]
    fn db_format_round_trips() {
        for unit in Unit::get_units() {
            let quantity = Quantity::new(3.0, unit);
            let (value, unit_type) = quantity.db_format();
            assert_eq!(Quantity::from_db(unit_type, value), quantity, "{unit}");
        }
    }

    #[test]
    fn unknown_db_unit_falls_back_to_volume() {
        assert_eq!(
            Quantity::from_db(42, 3.0),
            Quantity::Volume { quantity: 3.0 }
        );
    }

    #[test]
    fn readable_shows_value_and_unit() {
        let dashes = Quantity::new(2.0, Unit::Dash);
        assert_eq!(dashes.readable(UnitSystem::Metric), "2 dash");
        let volume = Quantity::new(50.0, Unit::Milliliter);
        assert_eq!(volume.readable(UnitSystem::Metric), "50 ml");
    }

    #[test]
    fn different_kinds_are_neither_equal_nor_ordered() {
        let volume = Quantity::new(1.0, Unit::Milliliter);
        let mass = Quantity::new(1.0, Unit::Gram);
        let dash = Quantity::new(1.0, Unit::Dash);
        assert_ne!(volume, mass);
        assert_ne!(mass, dash);
        assert_eq!(volume.partial_cmp(&mass), None);
        assert_eq!(dash.partial_cmp(&volume), None);
    }

    #[test]
    fn same_kinds_are_ordered_by_amount() {
        use std::cmp::Ordering;
        let less = Quantity::new(1.0, Unit::Gram);
        let more = Quantity::new(2.0, Unit::Gram);
        assert_eq!(less.partial_cmp(&more), Some(Ordering::Less));
        let one_dash = Quantity::new(1.0, Unit::Dash);
        let two_dashes = Quantity::new(2.0, Unit::Dash);
        assert_eq!(two_dashes.partial_cmp(&one_dash), Some(Ordering::Greater));
        assert_ne!(one_dash, two_dashes);
    }

    #[test]
    fn subtraction_never_goes_below_zero() {
        let mut volume = Quantity::new(50.0, Unit::Milliliter);
        volume -= Quantity::new(20.0, Unit::Milliliter);
        assert_eq!(volume, Quantity::new(30.0, Unit::Milliliter));
        volume -= Quantity::new(100.0, Unit::Milliliter);
        assert_eq!(volume, Quantity::new(0.0, Unit::Milliliter));

        let mut dashes = Quantity::new(3.0, Unit::Dash);
        dashes -= Quantity::new(1.0, Unit::Dash);
        assert_eq!(dashes, Quantity::new(2.0, Unit::Dash));
        dashes -= Quantity::new(5.0, Unit::Dash);
        assert_eq!(dashes, Quantity::new(0.0, Unit::Dash));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "mismatched")]
    fn subtracting_mismatched_kinds_panics_in_debug_builds() {
        let mut volume = Quantity::new(50.0, Unit::Milliliter);
        volume -= Quantity::new(20.0, Unit::Gram);
    }

    #[test]
    fn units_display_their_abbreviation() {
        let shown: Vec<String> = Unit::get_units().iter().map(ToString::to_string).collect();
        assert_eq!(shown, vec!["ml", "fl oz", "g", "oz", "dash"]);
    }

    #[test]
    fn unit_systems_swap_and_pick_default_units() {
        let mut system = UnitSystem::Metric;
        assert_eq!(system.default_units(), Unit::Milliliter);
        assert_eq!(system.to_string(), "Metric (ml)");
        system.swap();
        assert_eq!(system, UnitSystem::Imperial);
        assert_eq!(system.default_units(), Unit::FluidOunce);
        assert_eq!(system.to_string(), "Imperial (Oz)");
        system.swap();
        assert_eq!(system, UnitSystem::Metric);
    }
}
