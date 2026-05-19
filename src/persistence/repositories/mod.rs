use rusqlite::Connection;

use crate::{
    logic::{
        category::{Category, CategoryBody, CategoryID},
        item::{Item, ItemBody, ItemID},
    },
    persistence::DBError,
};

pub mod category;
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
    fn create_table(&self) -> Result<(), DBError>;
    fn insert(&self, body: &ItemBody) -> Result<ItemID, DBError>;
    fn update(&self, item: &Item) -> Result<(), DBError>;
    fn delete(&self, item: Item) -> Result<(), DBError>;
    fn get_range(&self, offset: usize, limit: usize) -> Result<Vec<Item>, DBError>;
}
pub trait CategoryRepository {
    fn create_table(&self) -> Result<(), DBError>;
    fn insert(&self, body: &CategoryBody) -> Result<CategoryID, DBError>;
    fn update(&self, category: &Category) -> Result<(), DBError>;
    fn delete(&self, category: Category) -> Result<(), DBError>;
    fn get_range(&self, offset: usize, limit: usize) -> Result<Vec<Category>, DBError>;
}
pub trait RecipeRepository {}
