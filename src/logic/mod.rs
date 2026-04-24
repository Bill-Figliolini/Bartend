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
    db: Box<Database>,
    category_manager: CategoryManager,
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let db = Box::new(Database::new(path));
        let mut db_initializers = Vec::new();
        db_initializers.push(Item::create());
        db_initializers.extend(CategoryManager::create());
        db.bulk_execute(db_initializers.as_ref());
        Self {
            db,
            category_manager: CategoryManager::new(),
        }
    }
    pub fn get_items(&self) -> Vec<Item> {
        todo!()
    }
    pub fn add_item(&self, name: &str, quantity: Quantity) -> ItemID {
        todo!()
    }
    pub fn update_item(&self, item: Item) {
        self.db.execute(&item.update());
    }
    pub fn delete_item(&self, item: Item) {
        self.db.execute(&item.delete());
    }
    pub fn get_categories(&self) -> Vec<Category> {
        self.category_manager.get_categories()
    }
    pub fn add_category(&mut self, name: String) -> CategoryID {
        todo!()
    }
    pub fn delete_category(&mut self, category: Category) {
        let stmts = Vec::new();
        self.db.bulk_execute(&stmts);
    }
    pub fn update_category(&mut self, category: Category) {
        self.db.execute(&category.update());
    }
}
