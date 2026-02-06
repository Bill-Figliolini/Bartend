mod schema;

use std::{fmt::Display, path::Path};

use crate::persistence::{PersistenceError, sqlite::schema::Schema};
use rusqlite::{self, Connection};
use sql_query_builder::{self, CreateTable, Insert};

struct DB {
    db: Connection,
}
impl DB {
    fn new() -> Result<Self, PersistenceError> {
        todo!()
    }
    fn get(&self) {
        todo!()
    }
}

//There's a design decision here that I had not considered.
// I could formulate each table as a struct of its names and fields, and use that to handle creation and insertion,
// instead of doing it manually.
fn create_tables(db: &DB) -> Result<(), PersistenceError> {
    //Initialize the Items Tables

    let create_items = CreateTable::new()
        .create_table_if_not_exists("items")
        .column("id INTEGER PRIMARY KEY")
        .column("name TEXT NOT NULL")
        .as_string();
    db.db.execute(&create_items, ());

    let create_item_quantities = CreateTable::new()
        .create_table_if_not_exists("item_quantities")
        .column("id INTEGER PRIMARY KEY")
        .column("quantity REAL NOT NULL")
        .foreign_key("FOREIGN KEY(item_id) REFERENCES items(id)")
        .to_string();
    db.db.execute(&create_item_quantities, ());

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile;
    mod table_creation {

        use super::*;

        #[test]
        fn item() {}
    }
}
