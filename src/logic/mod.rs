use crate::logic::item::{Item, ItemID, Items};

mod common;
mod item;

/// Defines potential manners in which the quantity of an ingredient can be defined.
/// Mass and Volume are handled by uom measures
/// Count is an i32 that can be multiplied into a float and interpreted by the user
pub enum Quantity {
    Mass(u32),
    Volume(u32),
    Count(u32),
}

enum Error {
    MismatchedUnits,
}

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
    pub fn get_items<'a>(&self) -> Vec<[String; 2]> {
        let mut results = Vec::with_capacity(self.item_ids.len());
        for id in self.item_ids.iter() {
            let item = self.inventory.get(&id);
            results.push(item.get_displayables());
        }
        results
    }
}
#[cfg(test)]
mod tests {
    use super::*;
}
