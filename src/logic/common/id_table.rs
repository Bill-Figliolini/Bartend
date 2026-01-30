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
}

#[cfg(test)]
mod test {
    use super::*;
    #[derive(Hash, Eq, PartialEq, Clone, Copy)]
    struct TestKey(u32);
    struct TestValue(u32);

    mod insert {
        use super::*;
        #[test]
        fn adds_value_to_table_and_returns_index() {
            let mut table: IdTable<TestKey, TestValue> = IdTable::new(TestKey);
            let insertion_value = 42;
            let value = TestValue(insertion_value);

            let result_index = table.insert(value);

            let in_table = table.table.get(&result_index);
            assert!(in_table.is_some());
            assert_eq!(in_table.unwrap().0, insertion_value);
        }
    }
    mod get {
        use super::*;
    }
    mod get_disjoint_mut {
        use super::*;
    }
}
