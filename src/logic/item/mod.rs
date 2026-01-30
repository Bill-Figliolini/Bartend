//! Module for handling user provided ingredients and items.
//!
//! Each Item is Composed of a Name and a Quantity
//! Each Item is Stored inside and owned by Items, a HashTable
//! Items are given unique IDs, so that they can be edited.
//!

use fnv::FnvHashMap;

use crate::logic::{Error, Quantity, common::id_table::IdTable};

pub(super) struct Items {
    table: IdTable<ItemID, Item>,
}
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct ItemID(u32);

struct Item {
    name: String,
    volume: u32,
}

impl Items {
    fn new() -> Self {
        Self {
            table: IdTable::new(ItemID),
        }
    }
    fn insert(&mut self, name: String, quantity: Quantity) {}
}

#[cfg(test)]
mod test {
    use super::*;
}
