pub mod mock_items;
mod sqlite;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(usize);

struct Item {
    id: ItemID,
    name: String,
    quantity: f64,
}

pub trait Repository {
    fn new() -> Self;
    fn add_item(&mut self, name: String, quantity: f32) -> ItemID;
    fn get_all_items(&self) -> Vec<[String; 2]>;
}
