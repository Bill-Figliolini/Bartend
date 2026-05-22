pub mod repositories;

use std::{error::Error, fmt::Display, path::Path};

use rusqlite::Connection;

use crate::persistence::repositories::{
    CategoryDB, CategoryRepository, ItemDB, ItemRepository, RecipeDB, RecipeRepository, Repository,
};

#[derive(Debug)]
pub enum DBError {
    External(rusqlite::Error),
}

impl From<rusqlite::Error> for DBError {
    fn from(value: rusqlite::Error) -> Self {
        DBError::External(value)
    }
}

#[derive(Debug)]
pub struct Database {
    pub connection: Connection,
}

impl<'a> Database {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, DBError> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self { connection };
        db.item_db().create_table()?;
        db.category_db().create_table()?;
        db.recipe_db().create_table()?;
        Ok(db)
    }
    #[must_use]
    pub fn item_db(&'a self) -> ItemDB<'a> {
        ItemDB {
            connection: &self.connection,
        }
    }
    #[must_use]
    pub fn category_db(&'a self) -> CategoryDB<'a> {
        CategoryDB {
            connection: &self.connection,
        }
    }
    #[must_use]
    pub fn recipe_db(&'a self) -> RecipeDB<'a> {
        RecipeDB {
            connection: &self.connection,
        }
    }
    #[must_use]
    pub fn item_category_db(&'a self) {
        todo!()
    }
}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::External(error) => write!(f, "External DB Error: {error}"),
        }
    }
}
impl Error for DBError {}
