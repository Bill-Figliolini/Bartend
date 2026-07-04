mod category_service;
mod graph;
mod item_service;
mod recipe_service;

use std::path::Path;

use crate::{
    models::{Recipe, RecipeBody, RecipeID},
    persistence::{Database, repositories::RecipeRepository},
};

pub use self::{category_service::CategoryService, item_service::ItemService};

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
