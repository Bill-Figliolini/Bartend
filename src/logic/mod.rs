use std::path::Path;

pub mod category;
pub mod config;
pub mod item;
pub mod quantity;
use crate::{
    logic::{
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
        if let Err(e) = db
            .connection
            .execute_batch(db_initializers.join(";\n").as_ref())
        {
            panic!("Error initializing DB: {e}");
        };
        Self {
            db,
            category_manager: CategoryManager::new(),
        }
    }
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
    pub fn get_categories(&self) -> Vec<Category> {
        self.category_manager.get_categories()
    }
    pub fn add_category(&mut self, name: String) -> CategoryID {
        todo!()
    }
    pub fn delete_category(&mut self, category: Category) {
        todo!()
    }
    pub fn update_category(&mut self, category: Category) {
        todo!()
    }
}
