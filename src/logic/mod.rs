use std::{collections::HashMap, path::Path};

pub mod category;
pub mod config;
pub mod graph;
pub mod item;
pub mod quantity;
pub mod recipe;
use rusqlite::OptionalExtension;

use crate::{
    logic::{
        category::{Category, CategoryBody, CategoryID},
        item::{Item, ItemBody, ItemID},
    },
    persistence::{
        Database,
        repositories::{CategoryRepository, ItemRepository},
    },
};

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    db: Database,
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let db = match Database::new(path) {
            Ok(db) => db,
            Err(e) => panic!("DB Creation Error: {e}"),
        };

        Self { db }
    }
    #[must_use]
    pub fn get_items(&self) -> Vec<Item> {
        match self.db.item_db().get_range(0, 100) {
            Ok(items) => items,
            Err(e) => panic!("{e}"),
        }
    }
    #[must_use]
    pub fn get_item_mapping(&self, items: &Vec<Item>) -> HashMap<ItemID, CategoryID> {
        let ids = items.iter().map(|item| item.id);
        self.get_item_category_map(ids)
    }
    pub fn add_item(&self, item: &ItemBody) -> ItemID {
        match self.db.item_db().insert(&item) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn update_item(&self, item: Item) {
        if let Err(e) = self.db.item_db().update(&item) {
            panic!("{e}");
        };
    }
    pub fn delete_item(&self, item: Item) {
        if let Err(e) = self.db.item_db().delete(item) {
            panic!("{e}");
        };
    }
    #[must_use]
    pub fn get_categories(&self) -> Vec<Category> {
        match self.db.category_db().get_range(0, 100) {
            Ok(categories) => categories,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn add_category(&self, body: &CategoryBody) -> CategoryID {
        match self.db.category_db().insert(body) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn delete_category(&self, category: Category) {
        if let Err(e) = self.db.category_db().delete(category) {
            panic!("{e}")
        }
    }
    pub fn update_category(&self, category: &Category) {
        if let Err(e) = self.db.category_db().update(category) {
            panic!("{e}")
        }
    }

    fn create_category_item_mapping() -> String {
        "CREATE TABLE IF NOT EXISTS category_item(
            category_id INTEGER,
            item_id INTEGER,
            FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE CASCADE,
            FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
            UNIQUE(category_id, item_id)
        )"
        .to_string()
    }
    fn get_item_category_map(
        &self,
        ids: impl Iterator<Item = ItemID>,
    ) -> HashMap<ItemID, CategoryID> {
        let mut stmt = self
            .db
            .connection
            .prepare("SELECT item_id, category_id FROM category_item WHERE item_id = ?1;")
            .unwrap();
        let mut mapping = HashMap::new();
        for id in ids {
            if let Some((item_id, category_id)) = stmt
                .query_row((id,), |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
                .optional()
                .unwrap()
            {
                mapping.insert(item_id, category_id);
            }
        }
        mapping
    }
    pub fn add_item_category_mapping(&self, item_id: ItemID, category_id: CategoryID) {
        if let Err(e) = self.db.connection.execute(
            "INSERT INTO category_item(category_id, item_id) VALUES (?1, ?2)",
            (category_id, item_id),
        ) {
            panic!("Error inserting Item Mapping: {e}")
        }
    }
    pub fn update_item_category_mapping(&self, item_id: ItemID, category_id: Option<CategoryID>) {
        if let Err(e) = self
            .db
            .connection
            .execute("DELETE FROM category_item WHERE item_id = ?1", (item_id,))
        {
            panic!("Error deleting item -> category mapping: {e}");
        }
        if let Some(category_id) = category_id {
            self.add_item_category_mapping(item_id, category_id);
        }
    }
}
