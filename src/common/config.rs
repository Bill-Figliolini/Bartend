//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use std::fmt::Display;
#[derive(Debug)]
pub struct Config {
    display_unit: DisplayUnit,
}

#[derive(Debug, Clone, Copy)]
pub enum DisplayUnit {
    Metric,
    Imperial,
}

impl Display for DisplayUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DisplayUnit::Metric => "Metric (ml)",
            DisplayUnit::Imperial => "Imperial (Oz)",
        };
        write!(f, "{text}")
    }
}
