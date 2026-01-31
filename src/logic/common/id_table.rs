//! IdTable
//! Creates a mapping from an Internal ID to a provided struct.

use crate::logic::common::id_generator::IdGenerator;
use std::{collections::HashMap, fmt::Display, hash::Hash};
pub struct IdTable<KeyType, ValueType>
where
    KeyType: Hash + Eq + PartialEq + Copy,
{
    table: HashMap<KeyType, ValueType>,
    id_generator: IdGenerator,
    id_constructor: fn(u32) -> KeyType,
}

impl<K, V> IdTable<K, V>
where
    K: Hash + Eq + PartialEq + Copy,
{
    pub fn new(id_constructor: fn(u32) -> K) -> Self {
        IdTable {
            table: HashMap::new(),
            id_generator: IdGenerator::new(),
            id_constructor,
        }
    }
    pub fn insert(&mut self, input: V) -> K {
        let key = (self.id_constructor)(self.id_generator.get_next_id());
        _ = self.table.insert(key, input);
        key
    }
    pub fn get(&self, key: &K) -> &V {
        let result = self.table.get(key);
        if result.is_none() {
            panic!("ERROR: INVALID ID IN CIRCULATION");
        }
        result.unwrap()
    }
    pub fn get_disjoint_mut<const N: usize>(&mut self, keys: [&K; N]) -> [&mut V; N] {
        self.table.get_disjoint_mut(keys).map(|i| i.unwrap())
    }
    pub fn remove(&mut self, key: K) {
        self.table.remove(&key);
        //TODO: Idea for later to save memory: Save deleted Keys in Id_generator to be reissued
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[derive(Hash, Eq, PartialEq, Clone, Copy)]
    struct TestKey(u32);

    mod insert {
        use super::*;
        #[test]
        fn adds_value_to_table_and_returns_index() {
            let mut table: IdTable<TestKey, u32> = IdTable::new(TestKey);
            let value = 42;

            let result_index = table.insert(value);

            let in_table = table.table.get(&result_index);
            assert!(in_table.is_some());
            assert_eq!(*in_table.unwrap(), value);
        }
    }
    mod get {
        use super::*;

        #[test]
        fn returns_value_at_provided_index() {
            let mut table: IdTable<TestKey, u32> = IdTable::new(TestKey);
            let value = 5309;

            let index = table.insert(value);
            let result_value = table.get(&index);

            assert_eq!(*result_value, value);
        }
    }
    mod get_disjoint_mut {

        use super::*;

        #[test]
        fn returns_value_at_provided_index() {
            let mut table: IdTable<TestKey, u32> = IdTable::new(TestKey);
            let values = vec![10, 20, 30];
            let intended_result = vec![10, 21, 31];
            let mut indices = vec![];
            for value in values {
                let index = table.insert(value);
                indices.push(index);
            }
        }
    }
}
