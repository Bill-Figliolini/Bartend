mod category_service;
mod graph;
mod item_service;
mod recipe_service;

use std::{collections::HashMap, path::Path};

use crate::{
    models::{CategoryID, Item, ItemBody, ItemID, Recipe, RecipeBody, RecipeID},
    persistence::{
        Database,
        repositories::{ItemMappingRepository, ItemRepository, RecipeRepository},
    },
};

pub use category_service::CategoryService;

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    pub db: Database,
}

impl BarCollection {
    pub fn run() {}
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
    pub fn get_item_mapping(&self, items: &[Item]) -> HashMap<ItemID, CategoryID> {
        let ids: Vec<ItemID> = items.iter().map(|item| item.id).collect();
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
        if let Some(old_category) = old_category
            && let Err(e) = self.db.mapping_db().delete(item, &old_category)
        {
            panic!("{e}");
        }
        if let Some(category) = category
            && let Err(e) = self.db.mapping_db().insert(item, category)
        {
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
    pub fn get_recipes(&self) -> Vec<Recipe> {
        match self.db.recipe_db().get_range(0, 100) {
            Ok(recipes) => recipes,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn add_recipe(&self, body: &RecipeBody) -> RecipeID {
        match self.db.recipe_db().insert(body) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn delete_recipe(&self, recipe: Recipe) {
        if let Err(e) = self.db.recipe_db().delete(recipe) {
            panic!("{e}");
        }
    }
    pub fn update_recipe(&self, recipe: &Recipe) {
        if let Err(e) = self.db.recipe_db().update(recipe) {
            panic!("{e}");
        }
    }
}
