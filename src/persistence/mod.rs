use crate::common::{
    item::{Item, ItemID},
    quantity::Quantity,
};
use std::path::Path;
pub mod sqlite;

pub trait Repository {
    fn new(file: impl AsRef<Path>) -> Self;
    fn add_item(&self, name: &str, quantity: Quantity) -> ItemID;
    fn get_item(&self, id: ItemID) -> Option<Item>;
    fn update_item(&self, item: Item);
    fn delete_item(&self, id: ItemID);
    fn get_all_items(&self) -> Vec<Item>;
}
