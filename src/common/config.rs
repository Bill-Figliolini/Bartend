//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    config_dir: PathBuf,
    db_path: PathBuf,
    default_unit_system: UnitSystem,
}
pub enum ConfigError {
    UnableToAccessDir,
    UnableToCreateDir,
}

const CONFIG_DIR_NAME: &str = "Bartend";
const DEFAULT_CONFIG_FILE_NAME: &str = "Bartend.json";
const DEFAULT_DB_NAME: &str = "Bartend.db";

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
        if config_dir.exists() {
            //Load in config
            let db_path = config_dir.join(DEFAULT_DB_NAME);
            Ok(Self {
                config_dir,
                db_path,
                default_unit_system: UnitSystem::Metric,
            })
        } else {
            //Need to configure default settings
            Config::initialize(config_dir)
        }
    }

    fn initialize(config_dir: PathBuf) -> Result<Config, ConfigError> {
        if let Err(_) = fs::create_dir(&config_dir) {
            return Err(ConfigError::UnableToCreateDir);
        }
        let db_path = config_dir.join(DEFAULT_DB_NAME);

        Ok(Self {
            config_dir,
            db_path,
            default_unit_system: UnitSystem::Metric,
        })
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
    pub fn default_units(&self) -> UnitSystem {
        self.default_unit_system
    }
}
