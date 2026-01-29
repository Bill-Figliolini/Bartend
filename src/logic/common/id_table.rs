use std::collections::HashMap;

use crate::logic::common::id_generator::IdGenerator;

pub struct IdTable<KeyType, ValueType> {
    table: HashMap<KeyType, ValueType>,
    id_generator: IdGenerator,
    id_constructor: fn(u32) -> KeyType,
}

impl<K, V> IdTable<K, V> {
    pub fn new(id_constructor: fn(u32) -> K) -> Self {
        IdTable {
            table: HashMap::new(),
            id_generator: IdGenerator::new(),
            id_constructor,
        }
    }
    pub fn insert(&mut self, input: V) -> K {
        let next_key = self.id_generator.get_next_id();
        (self.id_constructor)(next_key)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    struct TestKey(u32);
    struct TestValue();

    mod insert {
        use super::*;
        #[test]
        fn adds_value_to_table_and_returns_index() {
            let mut table: IdTable<TestKey, TestValue> = IdTable::new(TestKey);
            let value = TestValue();
            let result_index = table.insert(value);
        }
    }
}
