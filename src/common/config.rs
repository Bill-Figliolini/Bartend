//! # Config
//! Provides program defaults. If a page or operation overrides them, they should not be modified.

use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::PathBuf,
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
    path: PathBuf,
    db_path: PathBuf,
    default_unit_system: UnitSystem,
}
#[derive(Debug)]
pub enum ConfigError {
    UnableToAccessSystemConfigDir,
    UnableToAccessSystemDataDir,
    ReadError,
    WriteError,
    UnableToAccessConfigFile,
    UnableToCreateConfigFile,
    UnableToCreateDataDir,
}

const CONFIG_DIR_NAME: &str = "Bartend";
const DEFAULT_CONFIG_FILE_NAME: &str = "Bartend.json";
const DEFAULT_DB_NAME: &str = "Bartend.db";

impl Config {
    /// Loads a config if it exists at path, and creates one at path if it does not yet exist.
    /// If path is None, will load/create a config file in the system config folder.
    /// `db_path` provides an initial db location if config file does not exist.
    ///
    /// # Errors
    ///
    /// This function will return an error if it can not read or write to the system config or data directories.
    pub fn load(path: Option<PathBuf>, db_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let config_path = match path {
            Some(path) => path,
            None => build_config_path()?,
        };
        let config = if config_path.exists() {
            let Ok(file) = File::open(config_path) else {
                return Err(ConfigError::UnableToAccessConfigFile);
            };
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(config) => config,
                Err(_) => return Err(ConfigError::ReadError),
            }
        } else {
            let db_path = match db_path {
                Some(path) => path,
                None => build_db_path()?,
            };

            let config = Self {
                path: config_path,
                db_path,
                default_unit_system: UnitSystem::Metric,
            };

            if let Err(_) = config.save() {
                return Err(ConfigError::WriteError);
            }
            config
        };
        Ok(config)
    }

    /// Saves [`Config`] to disk at Config.path.
    ///
    /// # Errors
    ///
    /// This function will return an error if it can not write to config.path or if the json file is malformed.
    pub fn save(&self) -> Result<(), ConfigError> {
        let file_result = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path);
        let Ok(file) = file_result else {
            return Err(ConfigError::WriteError);
        };
        let writer = BufWriter::new(file);
        match serde_json::to_writer(writer, &self) {
            Ok(()) => Ok(()),
            Err(_) => Err(ConfigError::WriteError),
        }
    }
    #[must_use]
    pub const fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
    #[must_use]
    pub const fn default_units(&self) -> UnitSystem {
        self.default_unit_system
    }
}

/// Builds the default config file path
///
/// # Errors
///
/// This function will return an error if unable to write to system config directory.
fn build_config_path() -> Result<PathBuf, ConfigError> {
    let Some(config_dir) = dirs::config_dir() else {
        return Err(ConfigError::UnableToAccessSystemConfigDir);
    };
    let config_dir = config_dir.join(CONFIG_DIR_NAME);
    if !config_dir.exists()
        && let Err(_) = fs::create_dir(&config_dir)
    {
        return Err(ConfigError::UnableToCreateConfigFile);
    }
    Ok(config_dir.join(DEFAULT_CONFIG_FILE_NAME))
}

/// Builds the default db path
///
/// # Errors
///
/// This function will return an error if unable to write to system data directory.
fn build_db_path() -> Result<PathBuf, ConfigError> {
    let Some(db_dir) = dirs::data_dir() else {
        return Err(ConfigError::UnableToAccessSystemDataDir);
    };
    let db_dir = db_dir.join(CONFIG_DIR_NAME);
    if !db_dir.exists()
        && let Err(_) = fs::create_dir(&db_dir)
    {
        return Err(ConfigError::UnableToCreateDataDir);
    }
    Ok(db_dir.join(DEFAULT_DB_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    mod open {
        use super::*;

        #[test]
        fn creates_config_file_on_non_existent_file() {
            let test_dir = tempdir().unwrap();
            let test_file = test_dir.path().join("dne.json");
            let test_db = test_dir.path().join("dne.db");
            _ = Config::load(Some(test_file.clone()), Some(test_db)).unwrap();

            assert!(test_file.exists());
        }
    }
    mod save {
        use super::*;
        #[test]
        fn overwrites_contents() {
            let test_dir = tempdir().unwrap();
            let test_file = test_dir.path().join("contents.json");
            let test_db = test_dir.path().join("contents.db");
            let config = Config {
                path: test_file.clone(),
                db_path: test_db,
                default_unit_system: UnitSystem::Metric,
            };

            config.save().unwrap();
            config.save().unwrap();

            let reader = BufReader::new(File::open(test_file).unwrap());
            let result_config: Config = serde_json::from_reader(reader).unwrap();
            assert_eq!(config.path, result_config.path);
            assert_eq!(config.db_path, result_config.db_path);
        }
    }
}
