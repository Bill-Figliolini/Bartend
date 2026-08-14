use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use crate::{
    models::{
        Category, CategoryBody, CategoryID, Item, ItemBody, ItemID, Recipe, RecipeBody, RecipeID,
    },
    persistence::DBError,
};

pub mod category;
pub mod ingredients;
pub mod item;
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

pub trait ItemRepository {
    fn insert(&self, body: &ItemBody) -> Result<ItemID, DBError>;
    fn update(&self, item: &Item) -> Result<(), DBError>;
    fn delete(&self, item: ItemID) -> Result<(), DBError>;
    fn get_all(&self) -> Result<HashMap<ItemID, ItemBody>, DBError>;
}
pub trait CategoryRepository {
    fn insert(&self, body: &CategoryBody) -> Result<CategoryID, DBError>;
    fn update(&self, category: &Category) -> Result<(), DBError>;
    fn delete(&self, category: CategoryID) -> Result<(), DBError>;
    fn get_all(&self) -> Result<HashMap<CategoryID, CategoryBody>, DBError>;
    fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, DBError>;

    fn get_graph(&self) -> Result<HashMap<CategoryID, HashSet<CategoryID>>, DBError>;
    fn insert_relation(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError>;
    fn delete_node(&self, node: CategoryID) -> Result<(), DBError>;
    fn delete_edge(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError>;
    fn map_insert(&self, item: &ItemID, category: &CategoryID) -> Result<(), DBError>;
    fn map_delete(&self, item: &ItemID, category: &CategoryID) -> Result<(), DBError>;
}
pub trait RecipeRepository {
    fn insert(&self, body: &RecipeBody) -> Result<RecipeID, DBError>;
    fn update(&self, item: &Recipe) -> Result<(), DBError>;
    fn delete(&self, item: RecipeID) -> Result<(), DBError>;
    fn get_all(&self) -> Result<HashMap<RecipeID, RecipeBody>, DBError>;
}
