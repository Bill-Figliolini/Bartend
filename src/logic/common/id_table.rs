//! IdTable
//! Creates a mapping from an Internal ID to a provided struct.

use crate::logic::common::id_generator::IdGenerator;
use std::{collections::HashMap, hash::Hash};
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
    pub fn get(&self, key: &K) -> Option<&V> {
        self.table.get(key)
    }
    pub fn get_disjoint_mut<const N: usize>(&mut self, keys: [&K; N]) -> [Option<&mut V>; N] {
        self.table.get_disjoint_mut(keys)
    }
    pub fn remove(mut self, key: K) {
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

            assert!(result_value.is_some());
            assert_eq!(*result_value.unwrap(), value);
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
