use crate::logic::item::{ItemID, Items};

mod common;
mod item;
///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    inventory: Items,
    item_ids: Vec<ItemID>,
}

impl BarCollection {
    pub fn new() -> Self {
        Self {
            inventory: Items::new(),
            item_ids: Vec::new(),
        }
    }
    pub fn get_items(&self) -> Vec<[String; 2]> {
        let mut results = Vec::with_capacity(self.item_ids.len());
        for id in self.item_ids.iter() {
            let item = self.inventory.get(id);
            results.push(item.get_displayables());
        }
        results
    }
    pub fn add_item(&mut self, name: String, quantity: u32) {
        let id = self.inventory.insert(name, quantity);
        self.item_ids.push(id);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    mod on_start {}
    mod in_operation {}
}
