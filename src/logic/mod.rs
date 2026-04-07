use std::path::Path;

use crate::{
    common::{
        item::{Item, ItemID},
        quantity::Quantity,
    },
    persistence::sqlite::DB,
};

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    db_handler: Box<DB>,
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            db_handler: Box::new(DB::new(path)),
        }
    }
    pub fn get_items(&self) -> Vec<Item> {
        self.db_handler.get_all_items()
    }
    pub fn add_item(&self, name: &str, quantity: Quantity) -> ItemID {
        self.db_handler.add_item(name, quantity)
    }
    pub fn update_item(&self, item: Item) {
        self.db_handler.update_item(item);
    }
    pub fn delete_item(&self, item_id: ItemID) {
        self.db_handler.delete_item(item_id);
    }
}
