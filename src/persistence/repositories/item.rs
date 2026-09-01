use std::collections::HashMap;

use rusqlite::Row;

use crate::{
    models::{Item, ItemBody, ItemID, Quantity},
    persistence::{DBError, ItemDB, repositories::ItemRepository},
};

impl ItemDB<'_> {
    fn from_db(row: &Row<'_>) -> Result<Item, rusqlite::Error> {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let quantity = Quantity::from_db(row.get(3)?, row.get(2)?);
        Ok(Item {
            id,
            body: ItemBody { name, quantity },
        })
    }
}

impl ItemRepository for ItemDB<'_> {
    fn insert(&self, item: &ItemBody) -> Result<ItemID, DBError> {
        let (quantity, unit) = item.quantity.db_format();
        self.connection.execute(
            "INSERT INTO items(name, quantity, unit) VALUES (?1, ?2, ?3)",
            (&item.name, quantity, unit),
        )?;
        Ok(ItemID(self.connection.last_insert_rowid()))
    }

    fn update(&self, item: &Item) -> Result<(), DBError> {
        let id = item.id.0;
        let (quantity, unit) = item.body.quantity.db_format();
        self.connection.execute(
            "UPDATE items SET
                name = ?2,
                quantity = ?3,
                unit = ?4
                WHERE id = ?1",
            (id, &item.body.name, quantity, unit),
        )?;
        Ok(())
    }

    fn delete(&self, item: ItemID) -> Result<(), DBError> {
        self.connection
            .execute("DELETE FROM items WHERE id = ?1", (item,))?;
        Ok(())
    }

    fn get_all(&self) -> Result<HashMap<ItemID, ItemBody>, DBError> {
        let query = "SELECT * FROM items";
        let mut stmt = self.connection.prepare(query)?;
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
            };
            let result = db.item_db().insert(&item);
            assert!(result.is_ok());
            let id = result.unwrap();

            let _ = db.item_db().delete(id);

            let in_db = db
                .connection
                .query_one("SELECT * FROM items WHERE id=?1", (id,), |row| {
                    ItemDB::from_db(row)
                })
                .optional();
            assert!(in_db.is_ok());
            assert!(in_db.unwrap().is_none())
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
            };

            let result = db.item_db().insert(&item);
            assert!(result.is_ok());
            let id = result.unwrap();

            let in_db = db
                .connection
                .query_one("SELECT * FROM items WHERE id=?1", (id,), ItemDB::from_db)
                .unwrap();

            assert_eq!(in_db.body, item)
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
            };
            let new_item = ItemBody {
                name: "This is a Test".to_string(),
                quantity: Quantity::Volume { quantity: 375.0 },
            };
            let result = db.item_db().insert(&old_item);
            assert!(result.is_ok());
            let id = result.unwrap();

            let update_result = db.item_db().update(&Item {
                id,
                body: new_item.clone(),
            });

            assert!(update_result.is_ok());

            let in_db = db
                .connection
                .query_one("SELECT * FROM items WHERE id=?1", (id,), ItemDB::from_db)
                .unwrap();

            assert_eq!(in_db.body, new_item)
        }
    }
    mod get_all {
        use super::*;
        #[test]
        fn gets_all_from_db() {
            let test_size = 100;
            let db = db_init();
            let mut ids = Vec::new();
            let items: Vec<ItemBody> = (0..test_size)
                .map(|num| ItemBody {
                    name: format!("test{num}"),
                    quantity: Quantity::Volume {
                        quantity: num as f32,
                    },
                })
                .collect();
            for item in items.iter() {
                ids.push(db.item_db().insert(item).unwrap());
            }

            let result = db.item_db().get_all();
            assert!(result.is_ok());
            let in_db = result.unwrap();
            assert_eq!(in_db.len(), ids.len());

            for id in ids {
                assert!(in_db.get(&id).is_some());
                let category = in_db.get(&id).unwrap();
                assert!(items.contains(category));
            }
        }
    }
}
