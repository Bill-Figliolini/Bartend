use crate::persistence::{ItemID, Repository};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

#[derive(Debug)]
struct IdGenerator {
    counter: AtomicUsize,
}

impl IdGenerator {
    const fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
    fn get_next_id(&self) -> usize {
        self.counter.fetch_add(1, Relaxed)
    }
}

//All members of Item must have display implemented
#[derive(Debug)]
pub struct Item<'a> {
    name: &'a str,
    quantity: &'a f32,
}

impl<'a> Item<'a> {
    pub fn get_displayables(&'a self) -> [String; 2] {
        let name = self.name.to_string();
        let quantity = self.quantity.to_string();
        [name, quantity]
    }
}

impl<'a> Item<'a> {
    fn new(name: &'a String, quantity: &'a f32) -> Self {
        Item { name, quantity }
    }
}

#[derive(Debug)]
pub struct Items {
    names: HashMap<ItemID, String>,
    quantities: HashMap<ItemID, f32>,
    id_generator: IdGenerator,
}

impl Items {
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
    fn update_quantities<const N: usize>(&mut self, ids: [&ItemID; N], quantities: [&f32; N]) {
        let values = self.quantities.get_disjoint_mut(ids);
        let values = values.map(|i| i.expect("Unused IDs should not remain in use"));
        for i in 0..N {
            *values[i] -= quantities[i];
        }
    }
}
impl Repository for Items {
    fn new() -> Self {
        Self {
            names: HashMap::new(),
            quantities: HashMap::new(),
            id_generator: IdGenerator::new(),
        }
    }
    fn add_item(&mut self, name: String, quantity: f32) -> ItemID {
        let id = ItemID(self.id_generator.get_next_id());
        self.quantities.insert(id, quantity);
        self.names.insert(id, name);
        id
    }
    fn get_all_items(&self) -> Vec<[String; 2]> {
        let mut result = Vec::with_capacity(self.names.len());
        for id in self.names.keys() {
            result.push(self.get(*id).get_displayables());
        }
        result
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
                let index = items.add_item(char.to_string(), 750.0);
                ids.push(index);
            }
            let changed_ids = [&ids[0], &ids[2]];
            let changed_quantities = [&200.0, &100.0];

            items.update_quantities(changed_ids, changed_quantities);

            assert_eq!(*items.get(ids[0]).quantity, 550.0);
            assert_eq!(*items.get(ids[1]).quantity, 750.0);
            assert_eq!(*items.get(ids[2]).quantity, 650.0);
        }
    }
    mod item {
        use super::*;
        #[test]
        fn can_can_be_unwrapped_into_displayables() {
            let mut items = Items::new();
            let name = "a".to_string();
            let quantity = 750.0;
            let index = items.add_item(name.clone(), quantity);
            let item = items.get(index);

            let internals = item.get_displayables();

            assert!(internals[0].contains(&name));
            assert!(internals[1].contains(&quantity.to_string()));
        }
    }
}
