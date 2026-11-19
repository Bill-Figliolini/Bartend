use std::collections::HashMap;

use rusqlite::Row;

use crate::{
    models::{Item, ItemBody, ItemID, Quantity},
    persistence::{DBError, ItemDB, repositories::ItemRepository},
};

/// The columns [`ItemDB::from_db`] reads, in order.
const COLUMNS: &str = "id, name, quantity, unit, full_size, full_size_unit";

impl ItemDB<'_> {
    fn from_db(row: &Row<'_>) -> Result<Item, rusqlite::Error> {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let quantity = Quantity::from_db(row.get(3)?, row.get(2)?);
        let full_size: Option<f32> = row.get(4)?;
        let full_size_unit: Option<i32> = row.get(5)?;
        let full_size = full_size
            .zip(full_size_unit)
            .map(|(amount, unit)| Quantity::from_db(unit, amount));
        Ok(Item {
            id,
            body: ItemBody {
                name,
                quantity,
                full_size,
            },
        })
    }
}

/// The full size as its amount and unit columns, both NULL for none.
fn full_size_columns(item: &ItemBody) -> (Option<f32>, Option<i32>) {
    item.full_size.map(|full| full.db_format()).unzip()
}

impl ItemRepository for ItemDB<'_> {
    fn insert(&self, item: &ItemBody) -> Result<ItemID, DBError> {
        let (quantity, unit) = item.quantity.db_format();
        let (full_size, full_size_unit) = full_size_columns(item);
        self.connection.execute(
            "INSERT INTO items(name, quantity, unit, full_size, full_size_unit)
                VALUES (?1, ?2, ?3, ?4, ?5)",
            (&item.name, quantity, unit, full_size, full_size_unit),
        )?;
        Ok(ItemID(self.connection.last_insert_rowid()))
    }

    fn update(&self, item: &Item) -> Result<(), DBError> {
        let id = item.id.0;
        let (quantity, unit) = item.body.quantity.db_format();
        let (full_size, full_size_unit) = full_size_columns(&item.body);
        self.connection.execute(
            "UPDATE items SET
                name = ?2,
                quantity = ?3,
                unit = ?4,
                full_size = ?5,
                full_size_unit = ?6
                WHERE id = ?1",
            (
                id,
                &item.body.name,
                quantity,
                unit,
                full_size,
                full_size_unit,
            ),
        )?;
        Ok(())
    }

    fn delete(&self, item: ItemID) -> Result<(), DBError> {
        self.connection
            .execute("DELETE FROM items WHERE id = ?1", (item,))?;
        Ok(())
    }

    fn get_all(&self) -> Result<HashMap<ItemID, ItemBody>, DBError> {
        let query = format!("SELECT {COLUMNS} FROM items");
        let mut stmt = self.connection.prepare(&query)?;
        let rows = stmt.query_map([], ItemDB::from_db)?;
        let rows = rows
            .into_iter()
            .collect::<Result<Vec<Item>, rusqlite::Error>>()?;
        let items = rows.into_iter().map(|item| (item.id, item.body)).collect();
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::persistence::Database;

    use super::*;
    fn db_init() -> Database {
        Database::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    mod delete {
        use super::*;
        use rusqlite::OptionalExtension;
        #[test]
        fn removes_from_db() {
            let db = db_init();
            let item = ItemBody {
                name: "test".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
                full_size: None,
            };
            let result = db.item_db().insert(&item);
            assert!(result.is_ok());
            let id = result.unwrap();

            let _ = db.item_db().delete(id);

            let in_db = db
                .connection
                .query_one(
                    &format!("SELECT {COLUMNS} FROM items WHERE id=?1"),
                    (id,),
                    ItemDB::from_db,
                )
                .optional();
            assert!(in_db.is_ok());
            assert!(in_db.unwrap().is_none());
        }
    }

    mod insert {
        use super::*;
        #[test]
        fn returns_id_of_item() {
            let db = db_init();
            let item = ItemBody {
                name: "Test".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
                full_size: None,
            };

            let result = db.item_db().insert(&item);
            assert!(result.is_ok());
            let id = result.unwrap();

            let in_db = db
                .connection
                .query_one(
                    &format!("SELECT {COLUMNS} FROM items WHERE id=?1"),
                    (id,),
                    ItemDB::from_db,
                )
                .unwrap();

            assert_eq!(in_db.body, item);
        }
    }
    mod full_size {
        use super::*;
        use crate::models::Unit;

        fn reload(db: &Database, id: ItemID) -> ItemBody {
            db.item_db().get_all().unwrap().remove(&id).unwrap()
        }

        #[test]
        fn is_saved_and_loaded_with_its_kind() {
            let db = db_init();
            let mut item = ItemBody {
                name: "Sugar".to_string(),
                quantity: Quantity::new(200.0, Unit::Gram),
                full_size: Some(Quantity::new(1000.0, Unit::Gram)),
            };
            let id = db.item_db().insert(&item).unwrap();
            assert_eq!(reload(&db, id), item);

            item.full_size = Some(Quantity::new(500.0, Unit::Gram));
            db.item_db()
                .update(&Item {
                    id,
                    body: item.clone(),
                })
                .unwrap();
            assert_eq!(reload(&db, id), item);
        }

        #[test]
        fn can_be_cleared() {
            let db = db_init();
            let mut item = ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(700.0, Unit::Milliliter),
                full_size: Some(Quantity::new(750.0, Unit::Milliliter)),
            };
            let id = db.item_db().insert(&item).unwrap();

            item.full_size = None;
            db.item_db()
                .update(&Item {
                    id,
                    body: item.clone(),
                })
                .unwrap();

            assert_eq!(reload(&db, id), item);
        }
    }
    mod update {
        use super::*;
        #[test]
        fn updates_value_in_db() {
            let db = db_init();
            let old_item = ItemBody {
                name: "Test".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
                full_size: None,
            };
            let new_item = ItemBody {
                name: "This is a Test".to_string(),
                quantity: Quantity::Volume { quantity: 375.0 },
                full_size: None,
            };
            let result = db.item_db().insert(&old_item);
            assert!(result.is_ok());
            let id = result.unwrap();

            let update_result = db.item_db().update(&Item {
                id,
                body: new_item.clone(),
            });

            update_result.unwrap();

            let in_db = db
                .connection
                .query_one(
                    &format!("SELECT {COLUMNS} FROM items WHERE id=?1"),
                    (id,),
                    ItemDB::from_db,
                )
                .unwrap();

            assert_eq!(in_db.body, new_item);
        }
    }
    mod get_all {
        use super::*;
        #[test]
        fn gets_all_from_db() {
            let test_size: u16 = 100;
            let db = db_init();
            let mut ids = Vec::new();
            let items: Vec<ItemBody> = (0..test_size)
                .map(|num| ItemBody {
                    name: format!("test{num}"),
                    quantity: Quantity::Volume {
                        quantity: f32::from(num),
                    },
                    full_size: None,
                })
                .collect();
            for item in &items {
                ids.push(db.item_db().insert(item).unwrap());
            }

            let result = db.item_db().get_all();
            assert!(result.is_ok());
            let in_db = result.unwrap();
            assert_eq!(in_db.len(), ids.len());

            for id in ids {
                assert!(in_db.contains_key(&id));
                let category = in_db.get(&id).unwrap();
                assert!(items.contains(category));
            }
        }
    }
}
