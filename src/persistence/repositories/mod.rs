use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use crate::{
    models::{
        Category, CategoryBody, CategoryID, Item, ItemBody, ItemID, Recipe, RecipeBody, RecipeID,
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
    pub(super) connection: &'a mut Connection,
}
pub struct ItemMappingDB<'a> {
    pub(super) connection: &'a Connection,
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
    fn graph(&self) -> impl GraphRepository;
    fn mapping(&self) -> impl ItemMappingRepository;
    fn insert(&self, body: &CategoryBody) -> Result<CategoryID, DBError>;
    fn update(&self, category: &Category) -> Result<(), DBError>;
    fn delete(&self, category: CategoryID) -> Result<(), DBError>;
    fn get_all(&self) -> Result<HashMap<CategoryID, CategoryBody>, DBError>;
    fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, DBError>;
}
pub trait RecipeRepository: Repository {
    fn insert(&mut self, body: &RecipeBody) -> Result<RecipeID, DBError>;
    fn update(&mut self, item: &Recipe) -> Result<(), DBError>;
    fn delete(&self, item: RecipeID) -> Result<(), DBError>;
    fn get_all(&self) -> Result<HashMap<RecipeID, RecipeBody>, DBError>;
}

pub trait GraphRepository: Repository {
    fn get(&self) -> Result<HashMap<CategoryID, HashSet<CategoryID>>, DBError>;
    fn insert(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError>;
    fn delete_node(&self, node: CategoryID) -> Result<(), DBError>;
    fn delete_edge(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError>;
}
