use std::fmt::Display;

use rusqlite::{ToSql, types::FromSql};

use crate::persistence::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryID(pub i64);
#[derive(Debug, Clone)]
pub struct Category {
    id: CategoryID,
    name: String,
}
impl Category {
    fn new(id: CategoryID, name: String) -> Self {
        Self { id, name }
    }

    pub fn insert(name: String, db: &Database) -> CategoryID {
        todo!()
    }

    pub fn id(&self) -> CategoryID {
        self.id
    }
    pub fn create() -> String {
        "CREATE TABLE IF NOT EXISTS category(
            id INTEGER PRIMARY KEY,
            name STRING NOT NULL
        );"
        .to_string()
    }
    pub fn update(&self, db: &Database) {
        if let Err(e) = db.connection.execute(
            "
            UPDATE category SET
            name = ?2
            WHERE id = ?1
        ",
            (self.id.0, self.name.clone()),
        ) {
            panic!("Error Updating Category: {e}");
        };
    }
    pub fn delete(self) -> String {
        format!("DELETE * FROM category WHERE id={}", self.id.0)
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for CategoryID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
        Ok(CategoryID(value))
    }
}
