pub mod mock_items;

use std::path::Path;

use rusqlite::{self, Connection};
use sql_query_builder::{self, CreateTable};
struct DB {
    db: Connection,
}

pub enum DBErr {
    FailedToOpenDB(String),
    FailedToExecute,
}

impl DB {
    fn new() -> Result<Self, DBErr> {
        let path = Path::new("./bartend.db");
        let db = Connection::open(path);
        match db {
            Ok(db) => {
                _ = create_tables(&db);
                Ok(Self { db })
            }
            Err(_) => Err(DBErr::FailedToOpenDB(path.display().to_string())),
        }
    }
    fn get(&self) {}
}

//There's a desing decision here that I had not considered.
// I could formulate each table as a struct of its names and fields, and use that to handle creation and insertion, 
// instead of doing it manually.
fn create_tables(db: &Connection) -> Result<(), DBErr> {
    //Initialize the Items Tables

    let create_items_query = CreateTable::new()
        .create_table_if_not_exists("items")
        .column("id INTEGER PRIMARY KEY")
        .column("name TEXT NOT NULL")
        .as_string();

    let _ = db.execute(&create_items_query, ());

    let _ = db.execute(
        "CREATE TABLE IF NOT EXISTS item_quantities (
            id INTEGER PRIMARY KEY,
            quantity REAL NOT NULL,
            item_id INTEGER REFERENCES items(id)
        )",
        (),
    );

    Ok(())
}
