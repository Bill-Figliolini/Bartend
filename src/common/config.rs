//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use std::fmt::Display;
#[derive(Debug)]
pub struct Config {
    display_unit: UnitSystem,
}

#[derive(Debug, Clone, Copy)]
pub enum UnitSystem {
    Metric,
    Imperial,
}

impl Display for UnitSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            UnitSystem::Metric => "Metric (ml)",
            UnitSystem::Imperial => "Imperial (Oz)",
        };
        write!(f, "{text}")
    }
}
