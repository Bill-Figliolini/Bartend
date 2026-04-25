use std::fmt::Display;

use rusqlite::{ToSql, types::FromSql};

use crate::persistence::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryID(pub i64);
#[derive(Debug, Clone)]
pub struct Category {
    pub id: CategoryID,
    pub name: String,
}
impl Category {
    pub fn insert(name: String, db: &Database) -> CategoryID {
        if let Err(e) = db
            .connection
            .execute("INSERT INTO category(name) VALUES(?1)", (name,))
        {
            panic!("Error inserting Category: {e}");
        }
        CategoryID(db.connection.last_insert_rowid())
    }
    #[must_use]
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
    pub fn delete(self, db: &Database) {
        if let Err(e) = db
            .connection
            .execute("DELETE FROM category WHERE id=?1", (self.id,))
        {
            panic!("Error deleting Category: {e}");
        }
    }
    pub fn get_range(offset: usize, quantity: usize, db: &Database) -> Vec<Category> {
        let query = format!("SELECT * FROM category LIMIT {quantity} OFFSET {offset}");
        let mut stmt = db.connection.prepare(&query).expect("Query must be valid");
        let rows = stmt
            .query_map([], |row| {
                let id = row.get(0).unwrap();
                let name = row.get(1).unwrap();
                Ok(Self { id, name })
            })
            .unwrap();
        rows.into_iter()
            .fold(Vec::with_capacity(quantity), |mut acc, row| {
                match row {
                    Ok(item) => acc.push(item),
                    Err(e) => panic!("Retrieving Items failled with error: {e}"),
                }
                acc
            })
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
        Ok(Self(value))
    }
}
