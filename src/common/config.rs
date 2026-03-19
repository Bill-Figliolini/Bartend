//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use std::path::{Path, PathBuf};

use crate::common::quantity::UnitSystem;

//TODO:
// Going to try a restructuring of this to see how well it works.
// Unit System will be moved into Quantity, as that is where it belongs. X
// Config will contain wrapper types around the relevant configurations. X
// Each wrapper will allow for implementations in other modules as relevant.
// The primary example I thinking of at present is implementing a viewable trait in
// Presentation, to allow for separating out which data elements are responsible for which displays.

#[derive(Debug, Clone, Copy)]
pub struct DefaultUnitSystem(UnitSystem);

#[derive(Debug, Clone)]
pub struct DefaultDBPath(PathBuf);

#[derive(Debug, Clone)]
pub struct Config {
    db_path: DefaultDBPath,
    default_unit_system: DefaultUnitSystem,
}
impl Config {
    pub fn new() -> Self {
        let path = PathBuf::from("./bartend.db");
        Self {
            db_path: DefaultDBPath(path),
            default_unit_system: DefaultUnitSystem(UnitSystem::Metric),
        }
    }
    pub fn db_path(&self) -> &Path {
        self.db_path.0.as_path()
    }
    pub fn default_units(&self) -> UnitSystem {
        self.default_unit_system.0
    }
}
