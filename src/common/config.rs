//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    display_unit: UnitSystem,
}
impl Config {
    pub fn new() -> Self {
        let path = PathBuf::from("./bartend.db");
        Self {
            path,
            display_unit: UnitSystem::Metric,
        }
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
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
