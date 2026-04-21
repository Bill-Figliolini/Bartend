use std::path::Path;

use crate::{
    common::{
        category::{Category, CategoryID, CategoryManager},
        item::{Item, ItemID},
        quantity::Quantity,
    },
    persistence::{DBCreate, DBUnit, Database},
};

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    db: Box<Database>,
    category_manager: CategoryManager,
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let db = Box::new(Database::new(path));
        Item::create(&db);
        CategoryManager::create(&db);
        Self {
            db,
            category_manager: CategoryManager::new(),
        }
    }
    pub fn get_items(&self) -> Vec<Item> {
        self.db.get_all_items()
    }
    pub fn add_item(&self, name: &str, quantity: Quantity) -> ItemID {
        self.db.add_item(name, quantity)
    }
    pub fn update_item(&self, item: Item) {
        item.update(&self.db);
    }
    pub fn delete_item(&self, item: Item) {
        item.delete(&self.db);
    }
    pub fn get_categories(&self) -> Vec<Category> {
        self.category_manager.get_categories()
    }
    pub fn add_category(&mut self, name: String) -> CategoryID {
        Category::insert(name, &self.db)
    }
    pub fn delete_category(&mut self, category: Category) {
        category.delete(&self.db);
    }
    pub fn update_category(&mut self, category: Category) {
        category.update(&self.db);
    }
}
