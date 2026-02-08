use std::path::Path;

pub(super) mod mock_items;
mod sqlite;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(i64);

pub struct Item {
    id: ItemID,
    name: String,
    quantity: f64,
}

pub trait Repository {
    fn new(file: impl AsRef<Path>) -> Self;
    fn add_item(&self, name: &str, quantity: f32) -> ItemID;
    fn get_all_items(&self) -> Vec<Item>;
}
