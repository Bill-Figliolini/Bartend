use std::{collections::HashMap, path::Path};

pub mod category;
pub mod config;
pub mod graph;
pub mod item;
pub mod quantity;
pub mod recipe;

use crate::{
    logic::{
        category::{Category, CategoryBody, CategoryID},
        item::{Item, ItemBody, ItemID},
    },
    persistence::{
        Database,
        repositories::{CategoryRepository, ItemMappingRepository, ItemRepository},
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
        let ids: Vec<ItemID> = items.iter().map(|item| item.id.clone()).collect();
        match self.db.mapping_db().get_map(&ids) {
            Ok(output) => output,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn add_item_mapping(&self, item: &ItemID, category: &CategoryID) {
        if let Err(e) = self.db.mapping_db().insert(item, category) {
            panic!("{e}");
        }
    }
    pub fn update_item_mapping(&self, item: &ItemID, category: &Option<CategoryID>) {
        let old_category = match self.db.mapping_db().get_single(item) {
            Ok(category_id) => category_id,
            Err(e) => panic!("{e}"),
        };
        if let Some(old_category) = old_category {
            if let Err(e) = self.db.mapping_db().delete(item, &old_category) {
                panic!("{e}");
            }
        }
        if let Some(category) = category {
            if let Err(e) = self.db.mapping_db().delete(item, category) {
                panic!("{e}");
            }
        }
    }
    pub fn delete_item_mapping(&self, item: &ItemID, category: &CategoryID) {
        if let Err(e) = self.db.mapping_db().delete(item, category) {
            panic!("{e}");
        }
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
}
