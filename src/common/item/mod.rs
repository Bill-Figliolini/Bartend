//! # Item
//! The base element of this program, which all others work over.
//! Currently has no responsibilities, due to Bartend currently being structured in a data-oriented manner.
//! Instead, they are free floating structs to be operated on by free functions in the Presentation, Logic, and Persistance modules
//!
//! ## Potential Future Changes
//! Variants for mappings of IDs to quantities and names could be useful for Recipes, in a later version.

use crate::{
    common::quantity::Quantity,
    persistence::{DBCreate, DBUnit, Database},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(pub i64);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemID,
    pub name: String,
    pub quantity: Quantity,
}

impl Item {
    fn insert(name: String, quantity: Quantity) -> ItemID {
        todo!()
    }
    fn read(id: ItemID, db: &Database) -> Self {
        todo!()
    }
}

impl DBCreate for Item {
    fn create(db: &Database) {
        let create_items = "
            CREATE TABLE IF NOT EXISTS items(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit INTEGER NOT NULL
            );"
        .to_string();
        let result = db.connection.execute(&create_items, ());
        if let Err(e) = result {
            panic!("DB Initialization error: {e}");
        }
    }
}

impl DBUnit for Item {
    fn update(self, db: &Database) {
        let id = self.id.0;
        let query = "UPDATE items SET
                        name = ?2,
                        quantity = ?3,
                        unit = ?4
                        WHERE id = ?1"
            .to_string();
        let (quantity, unit) = self.quantity.db_format();

        if let Err(e) = db
            .connection
            .execute(&query, (id, self.name, quantity, unit))
        {
            panic!("Update item failed with error: {e}");
        }
    }

    fn delete(self, db: &Database) {
        let id = self.id.0;
        let query = "DELETE FROM items WHERE id = ?1".to_string();

        if let Err(e) = db.connection.execute(&query, (id,)) {
            panic!("Delete_item failed with error: {e}");
        }
    }
}
