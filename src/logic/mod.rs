use std::path::Path;

use crate::persistence::{Item, ItemID, Repository, sqlite::DB};

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    inventory: DB,
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            inventory: DB::new(path),
        }
    }
    pub fn get_items(&self) -> Vec<Item> {
        self.inventory.get_all_items()
    }
    pub fn add_item(&self, name: &str, quantity: f32) -> ItemID {
        self.inventory.add_item(name, quantity)
    }
    pub fn delete_item(&self, item: ItemID) {
        self.inventory.delete_item(item);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    mod on_start {}
    mod in_operation {}
}
