mod schema;

use std::{fmt::Display, path::Path};

use crate::persistence::{PersistenceError, Repository, sqlite::schema::Schema};
use rusqlite::{self, Connection};
use sql_query_builder::{self, CreateTable, Insert};

struct DB {
    connection: Connection,
}
impl DB {
    fn new(path: impl AsRef<Path>) -> Self {
        let connection = Connection::open(path);
        match connection {
            Ok(connection) => Self { connection },
            Err(e) => {
                panic!("DB could not be opened! {}", e)
            }
        }
    }
    fn get(&self) {
        todo!()
    }
}

impl Repository for DB {}

//There's a design decision here that I had not considered.
// I could formulate each table as a struct of its names and fields, and use that to handle creation and insertion,
// instead of doing it manually.
fn create_tables(db: &DB) {
    //Initialize the Items Tables

    let create_items = CreateTable::new()
        .create_table_if_not_exists("items")
        .column("id INTEGER PRIMARY KEY")
        .column("name TEXT NOT NULL")
        .column("quantity REAL NOT NULL")
        .as_string();
    let result = db.connection.execute(&create_items, ());
    if let Err(e) = result {
        panic!("DB Initialization error: {}", e);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod table_creation {
        use super::*;
        #[test]
        fn items() {
            let dir = Path::new("/tmp/bartend.db");
            let db = DB::new(dir);

            create_tables(&db);

            assert!(db.connection.table_exists(None, "items").unwrap());
            assert!(db.connection.column_exists(None, "items", "id").unwrap());
            assert!(db.connection.column_exists(None, "items", "name").unwrap());
            assert!(
                db.connection
                    .column_exists(None, "items", "quantity")
                    .unwrap()
            );
            let _ = std::fs::remove_file(dir);
        }
    }
}
