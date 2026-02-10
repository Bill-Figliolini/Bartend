mod schema;

use std::{
    fmt::{Display, format},
    path::Path,
};

use crate::persistence::{Item, ItemID, Repository, sqlite::schema::Schema};
use rusqlite::{self, Connection};
use sql_query_builder::{self, CreateTable, Insert};

struct DB {
    connection: Connection,
    items_schema: Schema,
}
impl Repository for DB {
    fn new(path: impl AsRef<Path>) -> Self {
        let items_schema = Schema::new("items")
            .column("id")
            .column("name")
            .column("quantity");

        let connection = Connection::open(path);
        match connection {
            Ok(connection) => Self {
                connection,
                items_schema,
            },
            Err(e) => {
                panic!("DB could not be opened! {}", e)
            }
        }
    }

    fn add_item(&self, name: &str, quantity: f32) -> ItemID {
        let table = self.items_schema.name();
        let table_columns = self.items_schema.columns();
        let query = Insert::new()
            .insert_into(&format!(
                "{} ({},{})",
                table, table_columns[1], table_columns[2]
            ))
            .values("(?1, ?2)")
            .debug()
            .as_string();
        let result = self.connection.execute(&query, (name, quantity));
        if let Err(e) = result {
            eprintln!("Item Insertion Error: {}", e);
        }
        ItemID(self.connection.last_insert_rowid())
    }

    fn get_all_items(&self) -> Vec<Item> {
        todo!()
    }
}

//There's a design decision here that I had not considered.
// I could formulate each table as a struct of its names and fields, and use that to handle creation and insertion,
// instead of doing it manually.
fn create_tables(db: &DB) {
    let create_items = CreateTable::new()
        .create_table_if_not_exists(db.items_schema.name())
        .column(&format!(
            "{} INTEGER PRIMARY KEY",
            db.items_schema.columns()[0]
        ))
        .column(&format!("{} TEXT NOT NULL", db.items_schema.columns()[1]))
        .column(&format!("{} REAL NOT NULL", db.items_schema.columns()[2]))
        .as_string();
    let result = db.connection.execute(&create_items, ());
    if let Err(e) = result {
        panic!("DB Initialization error: {}", e);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;
    mod table_creation {
        use super::*;
        #[test]
        fn items() {
            let dir = TempDir::new().unwrap();
            let file = dir.path().join("bartend.db");
            let db = DB::new(file);

            create_tables(&db);

            let items_name = db.items_schema.name();
            assert!(db.connection.table_exists(None, items_name).unwrap());
            for column in db.items_schema.columns() {
                assert!(
                    db.connection
                        .column_exists(None, items_name, column)
                        .unwrap()
                )
            }
        }
    }
    mod items {
        use super::*;
        #[test]
        fn insert() {
            let dir = TempDir::new().unwrap();
            let file = dir.path().join("bartend.db");
            let db = DB::new(file);
            create_tables(&db);

            let id = db.add_item("test", 750.0);

            assert_eq!(id.0, 1);
        }
        #[test]
        fn get() {
            let dir = TempDir::new().unwrap();
            let file = dir.path().join("bartend.db");
            let db = DB::new(file);
            create_tables(&db);
        }
    }
}
