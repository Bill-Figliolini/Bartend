//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

//TODO:
// Going to try a restructuring of this to see how well it works.
// Unit System will be moved into Quantity, as that is where it belongs.
// Config will contain wrapper types around the relevant configurations.
// Each wrapper will allow for implementations in other modules as relevant.
// The primary example I thinking of at present is implementing a viewable trait in
// Presentation, to allow for separating out which data elements are responsible for which displays.

#[derive(Debug)]
pub struct Config {
    db_path: PathBuf,
    default_unit_system: UnitSystem,
}
impl Config {
    pub fn new() -> Self {
        let path = PathBuf::from("./bartend.db");
        Self {
            db_path: path,
            default_unit_system: UnitSystem::Metric,
        }
    }
    pub fn db_path(&self) -> &Path {
        self.db_path.as_path()
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
