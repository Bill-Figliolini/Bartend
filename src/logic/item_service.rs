use std::collections::HashMap;

use crate::{
    logic::LogicError,
    models::{BartendError, Item, ItemBody, ItemID, ItemUse},
    persistence::repositories::ItemRepository,
};

#[derive(Debug)]
pub struct ItemService {
    items: HashMap<ItemID, ItemBody>,
}

impl ItemService {
    pub fn new(db: &impl ItemRepository) -> Result<Self, BartendError> {
        let items = db.get_all()?;
        Ok(ItemService { items })
    }

    pub fn get_page(&self, page: usize, page_size: usize) -> Vec<ItemID> {
        let page_start = page_size * page;
        self.get_sorted()
            .into_iter()
            .skip(page_start)
            .take(page_size)
            .collect()
    }

    pub fn get(&self, item: ItemID) -> Result<&ItemBody, BartendError> {
        match self.items.get(&item) {
            Some(body) => Ok(body),
            None => Err(LogicError::InvalidItem(item))?,
        }
    }

    fn get_sorted(&self) -> Vec<ItemID> {
        let mut entries: Vec<(String, ItemID)> = self
            .items
            .iter()
            .map(|(id, body)| (body.name.to_lowercase(), *id))
            .collect();
        entries.sort_unstable_by(|(a_name, a_id), (b_name, b_id)| {
            a_name.cmp(b_name).then_with(|| a_id.0.cmp(&b_id.0))
        });
        entries.into_iter().map(|(_, id)| id).collect()
    }

    fn get_mut(&mut self, item: ItemID) -> Result<&mut ItemBody, BartendError> {
        match self.items.get_mut(&item) {
            Some(body) => Ok(body),
            None => Err(LogicError::InvalidItem(item))?,
        }
    }

    pub fn use_items(
        &mut self,
        db: &impl ItemRepository,
        used_items: Vec<ItemUse>,
    ) -> Result<(), BartendError> {
        for usage in used_items {
            let updated_item = self.get_mut(usage.id)?;
            updated_item.quantity -= usage.quantity;
            let item = Item {
                id: usage.id,
                body: self.get(usage.id)?.clone(),
            };
            self.update(db, item)?;
        }
        Ok(())
    }

    pub fn add(
        &mut self,
        db: &impl ItemRepository,
        item: &ItemBody,
    ) -> Result<ItemID, BartendError> {
        let id = db.insert(item)?;
        self.items.insert(id, item.clone());
        Ok(id)
    }
    pub fn update(&mut self, db: &impl ItemRepository, item: Item) -> Result<(), BartendError> {
        db.update(&item)?;
        let item_location = self.get_mut(item.id)?;
        *item_location = item.body;
        Ok(())
    }
    pub fn delete(&mut self, db: &impl ItemRepository, item: ItemID) -> Result<(), BartendError> {
        db.delete(item)?;
        self.items.remove(&item);
        Ok(())
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Quantity, Unit};

    use super::*;

    struct TestDB {
        counter: i64,
    }
    impl TestDB {
        fn new() -> Self {
            Self { counter: 30 }
        }
    }
    impl ItemRepository for TestDB {
        fn insert(&self, _body: &ItemBody) -> Result<ItemID, crate::persistence::DBError> {
            let next = ItemID(self.counter);
            Ok(next)
        }

        fn update(&self, _item: &Item) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn delete(&self, _item: ItemID) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn get_all(&self) -> Result<HashMap<ItemID, ItemBody>, crate::persistence::DBError> {
            let mut result = HashMap::new();

            for i in 0..30 {
                result.insert(
                    ItemID(i),
                    ItemBody {
                        name: format!("item {}", i + 1),
                        quantity: Quantity::new(1.0, Unit::Milliliter),
                    },
                );
            }

            Ok(result)
        }
    }

    #[test]
    fn new_loads_in_whole_db() {
        let db = TestDB::new();
        let item_service = ItemService::new(&db).unwrap();

        assert_eq!(item_service.items.len(), 30);
    }

    #[test]
    fn delete_removes_value() {
        let db = TestDB::new();
        let mut item_service = ItemService::new(&db).unwrap();
        let item = ItemID(0);

        item_service.delete(&db, item).unwrap();

        assert!(!item_service.items.contains_key(&item));
    }
}
