use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering::Relaxed},
};

#[derive(Debug)]
struct IdGenerator {
    counter: AtomicU32,
}

impl IdGenerator {
    const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }
    fn get_next_id(&self) -> u32 {
        self.counter.fetch_add(1, Relaxed)
    }
}

//All members of Item must have display implemented
#[derive(Debug)]
pub struct Item<'a> {
    name: &'a str,
    quantity: &'a u32,
}

impl<'a> Item<'a> {
    pub fn get_displayables(&'a self) -> [String; 2] {
        let name = self.name.to_string();
        let quantity = self.quantity.to_string();
        [name, quantity]
    }
}

impl<'a> Item<'a> {
    fn new(name: &'a String, quantity: &'a u32) -> Self {
        Item { name, quantity }
    }
}

#[derive(Debug)]
pub struct Items {
    names: HashMap<ItemID, String>,
    quantities: HashMap<ItemID, u32>,
    id_generator: IdGenerator,
}
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(u32);

impl Items {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            quantities: HashMap::new(),
            id_generator: IdGenerator::new(),
        }
    }
    pub fn insert(&mut self, name: String, quantity: u32) -> ItemID {
        let id = self.id_generator.get_next_id();
        self.quantities.insert(ItemID(id), quantity);
        self.names.insert(ItemID(id), name);
        ItemID(id)
    }
    pub fn delete(&mut self, id: ItemID) {
        self.names.remove(&id);
        self.quantities.remove(&id);
    }
    pub fn get(&self, id: ItemID) -> Item<'_> {
        let name = self
            .names
            .get(&id)
            .expect("Unused IDs should not remain in use");
        let quantity = self
            .quantities
            .get(&id)
            .expect("Unused IDs should not remain in use");

        Item::new(name, quantity)
    }
    fn update_quantities<const N: usize>(&mut self, ids: [&ItemID; N], quantities: [&u32; N]) {
        let values = self.quantities.get_disjoint_mut(ids);
        let values = values.map(|i| i.expect("Unused IDs should not remain in use"));
        for i in 0..N {
            *values[i] -= quantities[i];
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    mod id_gen {
        use super::*;
        #[test]
        fn begins_at_0() {
            let generator = IdGenerator::new();
            assert_eq!(generator.get_next_id(), 0)
        }
        #[test]
        fn increments_by_one() {
            let generator = IdGenerator::new();
            assert_eq!(generator.get_next_id(), 0);
            assert_eq!(generator.get_next_id(), 1)
        }
    }

    mod items {
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

            assert_eq!(*items.get(ids[0]).quantity, 550);
            assert_eq!(*items.get(ids[1]).quantity, 750);
            assert_eq!(*items.get(ids[2]).quantity, 650);
        }
    }
    mod item {
        use super::*;
        #[test]
        fn can_can_be_unwrapped_into_displayables() {
            let mut items = Items::new();
            let name = "a".to_string();
            let quantity = 750;
            let index = items.insert(name.clone(), quantity);
            let item = items.get(index);

            let internals = item.get_displayables();

            assert!(internals[0].contains(&name));
            assert!(internals[1].contains(&quantity.to_string()));
        }
    }
}
