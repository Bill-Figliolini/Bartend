//! # Item
//! The base element of this program, which all others work over.
//! Currently has no responsibilities, due to Bartend currently being structured in a data-oriented manner.
//! Instead, they are free floating structs to be operated on by free functions in the Presentation, Logic, and Persistance modules
//!
//! ## Potential Future Changes
//! Variants for mappings of IDs to quantities and names could be useful for Recipes, in a later version.

use crate::{logic::quantity::Quantity, persistence::Database};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(pub i64);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemID,
    pub name: String,
    pub quantity: Quantity,
}

impl Item {
    pub fn create() -> String {
        "CREATE TABLE IF NOT EXISTS items(
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            quantity REAL NOT NULL,
            unit INTEGER NOT NULL
        );"
        .to_string()
    }
    pub fn insert(name: &str, quantity: Quantity, db: &Database) -> ItemID {
        let (quantity, unit) = quantity.db_format();
        if let Err(e) = db.connection.execute(
            "INSERT INTO items(name, quantity, unit) VALUES (?1, ?2, ?3)",
            (name, quantity, unit),
        ) {
            panic!("Error inserting Item: {e}");
        };
        ItemID(db.connection.last_insert_rowid())
    }

    pub fn update(&self, db: &Database) {
        let id = self.id.0;
        let (quantity, unit) = self.quantity.db_format();
        if let Err(e) = db.connection.execute(
            "UPDATE items SET
            name = ?2,
            quantity = ?3,
            unit = ?4
            WHERE id = ?1",
            (id, self.name.clone(), quantity, unit),
        ) {
            panic!("Error while updating item: {e}");
        }
    }

    pub fn delete(self, db: &Database) {
        if let Err(e) = db
            .connection
            .execute("DELETE FROM items WHERE id = ?1", (self.id.0,))
        {
            panic!("Error Deleting Item: {e}");
        }
    }

    //TODO: refactor this; take in a query and a fn(&row) -> T
    pub fn get_range(offset: i64, quantity: i64, db: &Database) -> Vec<Item> {
        let query = format!("SELECT * FROM items LIMIT {quantity} OFFSET {offset}");
        let mut stmt = db.connection.prepare(&query).expect("Query must be valid");
        let rows = stmt
            .query_map([], |row| {
                let id = ItemID(row.get(0).unwrap());
                let name = row.get(1).unwrap();
                let quantity = Quantity::from_db(row.get(3).unwrap(), row.get(2).unwrap());
                Ok(Item { id, name, quantity })
            })
            .unwrap();
        rows.into_iter().fold(
            Vec::with_capacity((quantity - offset) as usize),
            |mut acc, row| {
                match row {
                    Ok(item) => acc.push(item),
                    Err(e) => panic!("Retrieving Items failled with error: {e}"),
                };
                acc
            },
        )
    }
}
