//! Module for handling user provided ingredients and items.
//!
//! Each Item is Composed of a Name and a Quantity
//! Each Item is Stored inside and owned by Items, a HashTable
//! Items are given unique IDs, so that they can be edited.
//!

use crate::logic::common::id_table::IdTable;
#[derive(Debug)]
struct Item {
    name: String,
    quantity: u32,
}

impl Item {
    fn new(name: String, quantity: u32) -> Item {
        Item { name, quantity }
    }
}
#[derive(Debug)]
pub(super) struct Items {
    table: IdTable<ItemID, Item>,
}
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(super) struct ItemID(u32);

impl Items {
    pub fn new() -> Self {
        Self {
            table: IdTable::new(ItemID),
        }
    }
    fn insert(&mut self, name: String, quantity: u32) -> ItemID {
        let item = Item::new(name, quantity);
        self.table.insert(item)
    }
    fn delete(&mut self, id: ItemID) {
        self.table.remove(id);
    }
    fn get(&self, id: &ItemID) -> &Item {
        self.table.get(id)
    }
    fn update_quantities<const N: usize>(&mut self, ids: [&ItemID; N], quantities: [&u32; N]) {
        let values = self.table.get_disjoint_mut(ids);
        for i in 0..N {
            values[i].quantity -= quantities[i];
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn batch_updates_table() {
        let mut items = Items::new();
        let mut ids = vec![];
        for char in "abc".chars() {
            let index = items.insert(char.to_string(), 750);
            ids.push(index);
        }
        let changed_ids = [&ids[0], &ids[2]];
        let changed_quantities = [&200, &100];

        items.update_quantities(changed_ids, changed_quantities);

        assert_eq!(items.get(&ids[0]).quantity, 550);
        assert_eq!(items.get(&ids[1]).quantity, 750);
        assert_eq!(items.get(&ids[2]).quantity, 650);
    }
}
