use std::collections::HashMap;

use crate::{
    logic::{LogicError, stock::totals_by_key},
    models::{BartendError, Item, ItemBody, ItemID, ItemUse, Quantity},
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

    #[cfg(test)]
    pub fn get_page(&self, page: usize, page_size: usize) -> Vec<ItemID> {
        let page_start = page_size * page;
        self.sorted_ids()
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

    /// Every id, ordered by name (case-insensitively), then by id.
    pub fn sorted_ids(&self) -> Vec<ItemID> {
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

    /// Takes each use out of its item's stock. An item may be used more than
    /// once; its uses are totalled first.
    ///
    /// # Errors
    /// Nothing is changed if any item is unknown, would run out
    /// ([`LogicError::InsufficientStock`]), or is used in a different kind of
    /// quantity than it is stocked in ([`LogicError::IncompatibleUnits`]).
    /// A database failure part way through can still leave earlier items
    /// updated.
    pub fn use_items(
        &mut self,
        db: &impl ItemRepository,
        used_items: Vec<ItemUse>,
    ) -> Result<(), BartendError> {
        let totals = self.checked_totals(used_items)?;
        for (id, used) in totals {
            let mut body = self.get(id)?.clone();
            body.quantity -= used;
            self.update(db, Item { id, body })?;
        }
        Ok(())
    }

    /// The total used of each item, provided all of it is in stock.
    fn checked_totals(
        &self,
        used_items: Vec<ItemUse>,
    ) -> Result<Vec<(ItemID, Quantity)>, BartendError> {
        let uses = used_items
            .into_iter()
            .map(|usage| (usage.id, usage.quantity));
        let totals = totals_by_key(uses).map_err(LogicError::IncompatibleUnits)?;
        for &(item, needed) in &totals {
            let available = self.get(item)?.quantity;
            match available.covers(needed) {
                Some(true) => {}
                Some(false) => Err(LogicError::InsufficientStock {
                    item,
                    needed,
                    available,
                })?,
                None => Err(LogicError::IncompatibleUnits(item))?,
            }
        }
        Ok(totals)
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

    #[cfg(test)]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Unit;

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

    fn milliliters(quantity: f32) -> Quantity {
        Quantity::new(quantity, Unit::Milliliter)
    }

    #[test]
    fn pages_are_sorted_by_name() {
        let db = TestDB::new();
        let item_service = ItemService::new(&db).unwrap();

        // Names sort as text: "item 1", "item 10", "item 11", ...
        assert_eq!(
            item_service.get_page(0, 3),
            vec![ItemID(0), ItemID(9), ItemID(10)]
        );
        assert_eq!(
            item_service.get_page(9, 3),
            vec![ItemID(6), ItemID(7), ItemID(8)]
        );
        assert_eq!(item_service.get_page(10, 3), Vec::new());
    }

    #[test]
    fn getting_an_unknown_item_errors() {
        let item_service = ItemService::new(&TestDB::new()).unwrap();
        assert!(matches!(
            item_service.get(ItemID(99)),
            Err(BartendError::Logic(LogicError::InvalidItem(ItemID(99))))
        ));
    }

    #[test]
    fn add_caches_the_new_item() {
        let db = TestDB::new();
        let mut item_service = ItemService::new(&db).unwrap();
        let body = ItemBody {
            name: "Tanqueray".to_string(),
            quantity: milliliters(750.0),
        };

        let id = item_service.add(&db, &body).unwrap();

        assert_eq!(item_service.get(id).unwrap(), &body);
        assert_eq!(item_service.item_count(), 31);
    }

    #[test]
    fn update_replaces_the_cached_item() {
        let db = TestDB::new();
        let mut item_service = ItemService::new(&db).unwrap();
        let body = ItemBody {
            name: "renamed".to_string(),
            quantity: milliliters(5.0),
        };

        item_service
            .update(
                &db,
                Item {
                    id: ItemID(3),
                    body: body.clone(),
                },
            )
            .unwrap();

        assert_eq!(item_service.get(ItemID(3)).unwrap(), &body);
    }

    #[test]
    fn updating_an_unknown_item_errors() {
        let db = TestDB::new();
        let mut item_service = ItemService::new(&db).unwrap();
        let ghost = Item {
            id: ItemID(99),
            body: ItemBody::default(),
        };
        assert!(matches!(
            item_service.update(&db, ghost),
            Err(BartendError::Logic(LogicError::InvalidItem(ItemID(99))))
        ));
    }

    #[test]
    fn using_items_subtracts_and_stores_the_new_quantity() {
        let db = crate::persistence::Database::new(rusqlite::Connection::open_in_memory().unwrap())
            .unwrap();
        let mut item_service = ItemService::new(&db.item_db()).unwrap();
        let gin = item_service
            .add(
                &db.item_db(),
                &ItemBody {
                    name: "Tanqueray".to_string(),
                    quantity: milliliters(750.0),
                },
            )
            .unwrap();
        let shot = ItemUse {
            id: gin,
            quantity: milliliters(50.0),
        };

        item_service
            .use_items(&db.item_db(), vec![shot, shot])
            .unwrap();

        assert_eq!(item_service.get(gin).unwrap().quantity, milliliters(650.0));
        let stored = ItemService::new(&db.item_db()).unwrap();
        assert_eq!(stored.get(gin).unwrap().quantity, milliliters(650.0));
    }

    /// An in-memory database holding one 100 ml gin, and its service.
    fn stocked_gin() -> (crate::persistence::Database, ItemService, ItemID) {
        let db = crate::persistence::Database::new(rusqlite::Connection::open_in_memory().unwrap())
            .unwrap();
        let mut item_service = ItemService::new(&db.item_db()).unwrap();
        let gin = item_service
            .add(
                &db.item_db(),
                &ItemBody {
                    name: "Tanqueray".to_string(),
                    quantity: milliliters(100.0),
                },
            )
            .unwrap();
        (db, item_service, gin)
    }

    #[test]
    fn using_more_than_in_stock_changes_nothing() {
        let (db, mut item_service, gin) = stocked_gin();
        let pour = ItemUse {
            id: gin,
            quantity: milliliters(60.0),
        };

        let result = item_service.use_items(&db.item_db(), vec![pour, pour]);

        assert!(
            matches!(
                result,
                Err(BartendError::Logic(LogicError::InsufficientStock { item, needed, available }))
                    if item == gin && needed == milliliters(120.0) && available == milliliters(100.0)
            ),
            "{result:?}"
        );
        assert_eq!(item_service.get(gin).unwrap().quantity, milliliters(100.0));
        let stored = ItemService::new(&db.item_db()).unwrap();
        assert_eq!(stored.get(gin).unwrap().quantity, milliliters(100.0));
    }

    #[test]
    fn using_exactly_the_stock_empties_the_item() {
        let (db, mut item_service, gin) = stocked_gin();
        // Eleven pours of 100/11 ml total a hair over 100 ml in f32.
        let pour = ItemUse {
            id: gin,
            quantity: milliliters(100.0 / 11.0),
        };
        let pours = vec![pour; 11];
        let total = pours
            .iter()
            .try_fold(milliliters(0.0), |total, pour| {
                total.checked_add(pour.quantity)
            })
            .unwrap();
        assert_eq!(total.covers(milliliters(100.0)), Some(true));
        assert!(total > milliliters(100.0), "rounding should overshoot");

        item_service.use_items(&db.item_db(), pours).unwrap();

        assert_eq!(item_service.get(gin).unwrap().quantity, milliliters(0.0));
    }

    #[test]
    fn using_an_item_in_other_units_changes_nothing() {
        let (db, mut item_service, gin) = stocked_gin();
        let by_mass = ItemUse {
            id: gin,
            quantity: Quantity::new(5.0, Unit::Gram),
        };
        let by_volume = ItemUse {
            id: gin,
            quantity: milliliters(5.0),
        };

        for uses in [vec![by_mass], vec![by_volume, by_mass]] {
            let result = item_service.use_items(&db.item_db(), uses);
            assert!(
                matches!(
                    result,
                    Err(BartendError::Logic(LogicError::IncompatibleUnits(item))) if item == gin
                ),
                "{result:?}"
            );
        }
        assert_eq!(item_service.get(gin).unwrap().quantity, milliliters(100.0));
    }

    #[test]
    fn using_an_unknown_item_errors() {
        let db = TestDB::new();
        let mut item_service = ItemService::new(&db).unwrap();
        let ghost = ItemUse {
            id: ItemID(99),
            quantity: milliliters(1.0),
        };
        assert!(item_service.use_items(&db, vec![ghost]).is_err());
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
