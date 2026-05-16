//! # Quantity
//! Handles preservation of quantities from the Presentation layer down to the DB
//! ### Invariants
//! Internally, all quantities are represented in Metric units, for consistency and ease of conversion.
//! ### Behavior
//! Quantities allow of ease of conversion between Imperial and Metric, both at creation and later access.
//! Quantities handle type checking, guaranteeing that inconsistent operations like adding a liquid and a mass do not occur.

use std::fmt::Display;

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
            _ => unreachable!("Quantity was stored with invalid type {unit_type} in db"),
        }
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

                assert_eq!(quantity, count_as_metric);
            }
            #[test]
            fn imperial_quantity_does_not_alter_counts() {
                let quantity = 2.0;
                let count = Quantity::Count {
                    quantity,
                    name: CountName::Dash,
                };

                let count_as_imperial = count.value(UnitSystem::Imperial);

                assert_eq!(quantity, count_as_imperial);
            }
        }
        mod volume {}
        mod mass {}
    }
}
