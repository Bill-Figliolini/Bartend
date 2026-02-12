use std::path::Path;

pub mod sqlite;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(i64);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemID,
    pub name: String,
    pub quantity: f32,
}

pub trait Repository {
    fn new(file: impl AsRef<Path>) -> Self;
    fn add_item(&self, name: &str, quantity: f32) -> ItemID;
    fn get_item(&self, id: ItemID) -> Option<Item>;
    fn delete_item(&self, id: ItemID);
    fn get_all_items(&self) -> Vec<Item>;
}
