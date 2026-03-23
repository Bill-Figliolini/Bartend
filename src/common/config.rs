//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

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
pub struct Config {
    config_dir: PathBuf,
    default_unit_system: DefaultUnitSystem,
}
pub enum ConfigError {
    UnableToAccessDir,
    UnableToCreateDir,
}

const CONFIG_DIR_NAME: &str = "Bartend";
const DB_NAME: &str = "Bartend.db";

impl Config {
    pub fn load() -> Result<Config, ConfigError> {
        let base_config_dir = dirs::config_dir();
        let config_dir = match base_config_dir {
            Some(dir) => dir,
            None => match env::current_dir() {
                Ok(dir) => dir,
                Err(_) => return Err(ConfigError::UnableToAccessDir),
            },
        }
        .join(Path::new(CONFIG_DIR_NAME));

        if !config_dir.exists() {
            if let Err(_) = fs::create_dir(&config_dir) {
                return Err(ConfigError::UnableToCreateDir);
            }
        }

        Ok(Self {
            config_dir,
            default_unit_system: DefaultUnitSystem(UnitSystem::Metric),
        })
    }
    pub fn db_path(&self) -> PathBuf {
        self.config_dir.join(Path::new(DB_NAME))
    }
    pub fn default_units(&self) -> UnitSystem {
        self.default_unit_system.0
    }
}
