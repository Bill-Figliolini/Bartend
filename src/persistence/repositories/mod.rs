use std::collections::HashMap;

use rusqlite::Connection;

use crate::{
    models::{
        Category, CategoryBody, CategoryID, Ingredient, Item, ItemBody, ItemID, Recipe, RecipeBody,
        RecipeID,
    },
    persistence::DBError,
};

pub mod category;
pub mod graph;
pub mod ingredients;
pub mod item;
pub mod mapping;
pub mod recipe;

pub struct ItemDB<'a> {
    pub(super) connection: &'a Connection,
}
pub struct CategoryDB<'a> {
    pub(super) connection: &'a Connection,
}
pub struct RecipeDB<'a> {
    pub(super) connection: &'a Connection,
}
pub struct ItemMappingDB<'a> {
    pub(super) connection: &'a Connection,
}
struct IngredientDB<'a> {
    connection: &'a Connection,
}
pub trait Repository {
    fn create_table(&self) -> Result<(), DBError>;
}

pub trait ItemRepository: Repository {
    fn insert(&self, body: &ItemBody) -> Result<ItemID, DBError>;
    fn update(&self, item: &Item) -> Result<(), DBError>;
    fn delete(&self, item: ItemID) -> Result<(), DBError>;
    fn get_all(&self) -> Result<HashMap<ItemID, ItemBody>, DBError>;
}
pub trait ItemMappingRepository: Repository {
    fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, DBError>;
    fn insert(&self, item: &ItemID, category: &CategoryID) -> Result<(), DBError>;
    fn delete(&self, item: &ItemID, category: &CategoryID) -> Result<(), DBError>;
}
pub trait CategoryRepository: Repository {
    fn insert(&self, body: &CategoryBody) -> Result<CategoryID, DBError>;
    fn update(&self, category: &Category) -> Result<(), DBError>;
    fn delete(&self, category: CategoryID) -> Result<(), DBError>;
    fn get_all(&self) -> Result<HashMap<CategoryID, CategoryBody>, DBError>;
}
pub trait RecipeRepository: Repository {
    fn insert(&self, body: &RecipeBody) -> Result<RecipeID, DBError>;
    fn update(&self, item: &Recipe) -> Result<(), DBError>;
    fn delete(&self, item: RecipeID) -> Result<(), DBError>;
    fn get_range(&self, offset: usize, limit: usize) -> Result<Vec<Recipe>, DBError>;
}
pub trait IngredientRepository: Repository {
    fn insert(
        &self,
        recipe: &RecipeID,
        index: &usize,
        ingredient: &Ingredient,
    ) -> Result<(), DBError>;
    fn delete(&self, recipe: &RecipeID) -> Result<(), DBError>;
    fn get(&self, recipe: &RecipeID) -> Result<Vec<Ingredient>, rusqlite::Error>;
}

pub trait GraphRepository: Repository {}
