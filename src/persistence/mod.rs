use crate::common::{
    item::{Item, ItemID},
    quantity::Quantity,
};
pub mod sqlite;

pub trait Repository {
    fn add_item(&self, name: &str, quantity: Quantity) -> ItemID;
    fn get_item(&self, id: ItemID) -> Option<Item>;
    fn update_item(&self, item: Item);
    fn delete_item(&self, id: ItemID);
    fn get_all_items(&self) -> Vec<Item>;
}
