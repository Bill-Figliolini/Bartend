use std::path::Path;

pub mod category;
pub mod config;
pub mod graph;
pub mod item;
pub mod quantity;
use crate::{
    logic::{
        category::{Category, CategoryID},
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
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let db = Box::new(Database::new(path));
        let db_initializers = [
            "PRAGMA foreign_keys = ON".to_string(),
            Item::create(),
            Category::create(),
        ];

        if let Err(e) = db
            .connection
            .execute_batch(db_initializers.join(";\n").as_ref())
        {
            panic!("Error initializing DB: {e}");
        }
        Self { db }
    }
    #[must_use]
    pub fn get_items(&self) -> Vec<Item> {
        Item::get_range(0, 100, &self.db)
    }
    pub fn add_item(&self, name: &str, quantity: Quantity) -> ItemID {
        Item::insert(name, quantity, &self.db)
    }
    pub fn update_item(&self, item: Item) {
        item.update(&self.db);
    }
    pub fn delete_item(&self, item: Item) {
        item.delete(&self.db);
    }
    #[must_use]
    pub fn get_categories(&self) -> Vec<Category> {
        Category::get_range(0, 100, &self.db)
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
