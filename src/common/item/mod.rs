//! # Item
//! The base element of this program, which all others work over.
//! Currently has no responsibilities, due to Bartend currently being structured in a data-oriented manner.
//! Instead, they are free floating structs to be operated on by free functions in the Presentation, Logic, and Persistance modules
//!
//! ## Potential Future Changes
//! Variants for mappings of IDs to quantities and names could be useful for Recipes, in a later version.

use crate::{common::quantity::Quantity, persistence::Database};

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
    pub fn insert(name: String, quantity: Quantity) -> String {
        let (quantity, unit) = quantity.db_format();
        format!("INSERT INTO items(name, quantity, unit) VALUES ({name}, {quantity}, {unit})")
            .to_string()
    }

    pub fn update(&self) -> String {
        let id = self.id.0;
        let (quantity, unit) = self.quantity.db_format();
        format!(
            "UPDATE items SET
            name = {},
            quantity = {quantity},
            unit = {unit}
            WHERE id = {id}",
            self.name
        )
        .to_string()
    }

    pub fn delete(self) -> String {
        format!("DELETE FROM items WHERE id = {}", self.id.0)
    }
}
