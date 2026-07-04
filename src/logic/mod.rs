mod category_service;
mod graph;
mod item_service;
mod recipe_service;

use std::path::Path;

use crate::{
    models::{Item, ItemBody, ItemID, Recipe, RecipeBody, RecipeID},
    persistence::{
        Database,
        repositories::{ItemRepository, RecipeRepository},
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
