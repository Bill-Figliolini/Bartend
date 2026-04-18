use std::path::Path;

use crate::{
    common::{
        category::{Category, CategoryID, CategoryManager},
        item::{Item, ItemID},
        quantity::Quantity,
    },
    persistence::Database,
};

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    db_handler: Box<Database>,
    category_manager: CategoryManager,
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            db_handler: Box::new(Database::new(path)),
            category_manager: CategoryManager::new(),
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
    pub fn get_categories(&self) -> Vec<Category> {
        self.category_manager.get_categories()
    }
    pub fn add_category(&mut self, name: String) {
        self.category_manager
            .add_category(self.db_handler.as_ref(), name);
    }
    pub fn delete_category(&mut self, category_id: CategoryID) {
        self.category_manager
            .remove_category(self.db_handler.as_ref(), category_id);
    }
}
