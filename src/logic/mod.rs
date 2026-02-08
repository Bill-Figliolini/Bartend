use crate::persistence::{Repository, mock_items::Items};

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    inventory: Items,
}

impl BarCollection {
    pub fn new() -> Self {
        Self {
            inventory: Items::new(""),
        }
    }
    pub fn get_items(&self) -> Vec<[String; 2]> {
        self.inventory.get_all_items()
    }
    pub fn add_item(&mut self, name: String, quantity: f32) {
        self.inventory.add_item(name, quantity);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    mod on_start {}
    mod in_operation {}
}
