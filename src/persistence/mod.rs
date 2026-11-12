mod migrations;
pub mod repositories;

use std::{error::Error, fmt::Display, path::Path};

use rusqlite::{Connection, Error::SqliteFailure, ErrorCode, ToSql, types::FromSql};

use crate::{
    models::{CategoryID, ItemID, RecipeID},
    persistence::{
        migrations::{LATEST, MIGRATIONS, VERSION_PRAGMA},
        repositories::{CategoryDB, ItemDB, RecipeDB},
    },
};
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct DBVersion(i64);

#[derive(Debug, PartialEq, Clone)]
pub enum DBError {
    NotABartendDB,
    FutureSchema {
        found: DBVersion,
        supported: DBVersion,
    },
    RestrictViolation,
    InvalidUnit,
    External(String),
}

impl From<rusqlite::Error> for DBError {
    fn from(value: rusqlite::Error) -> Self {
        match value {
            SqliteFailure(e, _) if e.code == ErrorCode::ConstraintViolation => {
                DBError::RestrictViolation
            }
            _ => DBError::External(value.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct Database {
    pub(in crate::persistence) connection: Connection,
}

fn migrate(db: &Connection) -> Result<(), DBError> {
    let current_version: DBVersion = db.pragma_query_value(None, VERSION_PRAGMA, |v| v.get(0))?;
    if current_version.0 == 0 {
        let table_count_query = "
            SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';";
        let result: i64 = db.query_one(table_count_query, [], |r| r.get(0))?;
        if result != 0 {
            return Err(DBError::NotABartendDB);
        }
    } else if current_version > LATEST {
        return Err(DBError::FutureSchema {
            found: current_version,
            supported: LATEST,
        });
    }
    for m in MIGRATIONS
        .iter()
        .filter(|m| i64::from(m.version) > current_version.0)
    {
        let tx = db.unchecked_transaction()?;
        tx.execute_batch(m.sql)?;
        tx.pragma_update(None, VERSION_PRAGMA, m.version)?;
        tx.commit()?;
    }
    Ok(())
}
impl Database {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DBError> {
        let connection = Connection::open(path)?;
        Database::new(connection)
    }
    pub fn new(connection: Connection) -> Result<Self, DBError> {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&connection)?;
        let db = Self { connection };
        Ok(db)
    }
    #[must_use]
    pub fn item_db(&self) -> ItemDB<'_> {
        ItemDB {
            connection: &self.connection,
        }
    }
    #[must_use]
    pub fn category_db(&self) -> CategoryDB<'_> {
        CategoryDB {
            connection: &self.connection,
        }
    }
    #[must_use]
    pub fn recipe_db(&self) -> RecipeDB<'_> {
        RecipeDB {
            connection: &self.connection,
        }
    }
}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DB ERROR: ")?;
        match self {
            DBError::External(error) => write!(f, "External DB Error: {error}"),
            DBError::RestrictViolation => write!(f, "Attempted to delete Restricted Value"),
            DBError::NotABartendDB => write!(f, "Attempted to read a DB from another application"),
            DBError::FutureSchema { found, supported } => write!(
                f,
                "Attempted to read db from version {found} of Bartend. Only version {supported} is supported"
            ),
            DBError::InvalidUnit => write!(f, "Attempted to read invalid unit"),
        }
    }
}
impl Error for DBError {}

impl Display for DBVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
impl ToSql for DBVersion {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl FromSql for DBVersion {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_i64()?;
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{ItemBody, Quantity, Unit},
        persistence::repositories::ItemRepository,
    };

    #[test]
    fn a_saved_database_reopens_with_its_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bartend.db");
        let body = ItemBody {
            name: "Tanqueray".to_string(),
            quantity: Quantity::new(750.0, Unit::Milliliter),
        };
        let id = Database::load(&path)
            .unwrap()
            .item_db()
            .insert(&body)
            .unwrap();

        let reopened = Database::load(&path).unwrap();

        assert_eq!(reopened.item_db().get_all().unwrap()[&id], body);
    }

    #[test]
    fn non_constraint_sqlite_errors_are_external() {
        let error = DBError::from(rusqlite::Error::InvalidQuery);
        assert!(matches!(error, DBError::External(_)));
    }

    #[test]
    fn errors_describe_the_problem() {
        let cases = [
            (
                DBError::External("disk full".to_string()),
                "DB ERROR: External DB Error: disk full",
            ),
            (
                DBError::RestrictViolation,
                "DB ERROR: Attempted to delete Restricted Value",
            ),
            (
                DBError::NotABartendDB,
                "DB ERROR: Attempted to read a DB from another application",
            ),
            (
                DBError::FutureSchema {
                    found: DBVersion(3),
                    supported: DBVersion(1),
                },
                "DB ERROR: Attempted to read db from version 3 of Bartend. Only version 1 is supported",
            ),
            (
                DBError::InvalidUnit,
                "DB ERROR: Attempted to read invalid unit",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }
}
