use crate::common::item::{Item, ItemID};
use std::path::Path;
pub mod sqlite;

pub trait Repository {
    fn new(file: impl AsRef<Path>) -> Self;
    fn add_item(&self, name: &str, quantity: f32) -> ItemID;
    fn get_item(&self, id: ItemID) -> Option<Item>;
    fn update_item(&self, item: Item);
    fn delete_item(&self, id: ItemID);
    fn get_all_items(&self) -> Vec<Item>;
}

#[cfg(test)]
mod test {
    use super::*;
}
