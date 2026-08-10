pub mod repositories;

use std::{error::Error, fmt::Display, path::Path};

use rusqlite::{Connection, ToSql, types::FromSql};

use crate::{
    models::{CategoryID, ItemID, RecipeID},
    persistence::repositories::{CategoryDB, ItemDB, RecipeDB, Repository},
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
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DBError> {
        let connection = Connection::open(path)?;
        Database::new(connection)
    }
    pub fn new(connection: Connection) -> Result<Self, DBError> {
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
}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::External(error) => write!(f, "External DB Error: {error}"),
        }
    }
}
impl Error for DBError {}

impl ToSql for RecipeID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl FromSql for RecipeID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_i64()?;
        Ok(Self(value))
    }
}

impl ToSql for ItemID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for ItemID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_i64()?;
        Ok(Self(value))
    }
}

impl ToSql for CategoryID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for CategoryID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_i64()?;
        Ok(Self(value))
    }
}
