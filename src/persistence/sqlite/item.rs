use crate::{
    common::{
        item::{Item, ItemID},
        quantity::Quantity,
    },
    persistence::sqlite::Database,
};
use rusqlite::{self, Connection, OptionalExtension};

pub(super) fn create_item_table(connection: &Connection) {
    let create_items = "CREATE TABLE IF NOT EXISTS items(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit INTEGER NOT NULL
                );"
    .to_string();
    let result = connection.execute(&create_items, ());
    if let Err(e) = result {
        panic!("DB Initialization error: {e}");
    }
}

impl Database {
    pub fn add_item(&self, name: &str, quantity: Quantity) -> ItemID {
        let query = "INSERT INTO items(name, quantity, unit) VALUES (?1, ?2, ?3)".to_string();
        let (quantity, unit) = quantity.db_format();
        let result = self.connection.execute(&query, (name, quantity, unit));
        match result {
            Ok(_) => ItemID(self.connection.last_insert_rowid()),
            Err(e) => {
                panic!("Item Insertion Error: {e}");
            }
        }
    }
    pub fn get_item(&self, id: ItemID) -> Option<Item> {
        let id = id.0;
        let query = "SELECT * FROM items WHERE id = ?1".to_string();
        self.connection
            .query_row(&query, [(id)], |row| {
                let id = ItemID(row.get(0).expect("idx 0 corresponds to id"));
                let name = row.get(1).expect("idx 1 corresponds to name");
                let quantity = match row.get(3).unwrap() {
                    0 => Quantity::Volume {
                        quantity: row.get(2).unwrap(),
                    },
                    1 => Quantity::Mass {
                        quantity: row.get(2).unwrap(),
                    },
                    2 => Quantity::Count {
                        quantity: row.get(2).unwrap(),
                        name: crate::common::quantity::CountName::Dash,
                    },
                    _ => panic!("Item inserted with invalid unit!"),
                };
                Ok(Item { id, name, quantity })
            })
            .optional()
            .unwrap()
    }

    pub fn update_item(&self, item: Item) {
        let id = item.id.0;
        let query = "UPDATE items SET
                name = ?2,
                quantity = ?3,
                unit = ?4
                WHERE id = ?1"
            .to_string();
        let (quantity, unit) = item.quantity.db_format();

        if let Err(e) = self
            .connection
            .execute(&query, (id, item.name, quantity, unit))
        {
            panic!("Update item failed with error: {e}");
        }
    }

    pub fn delete_item(&self, id: ItemID) {
        let id = id.0;
        let query = "DELETE FROM items WHERE id = ?1".to_string();

        if let Err(e) = self.connection.execute(&query, (id,)) {
            panic!("Delete_item failed with error: {e}");
        }
    }

    pub fn get_all_items(&self) -> Vec<Item> {
        let query = "SELECT * FROM items".to_string();
        let mut stmt = self
            .connection
            .prepare(&query)
            .expect("query must be valid sql");
        let rows = stmt
            .query_map([], |row| {
                let id = ItemID(row.get(0).expect("idx 0 corresponds to id"));
                let name = row.get(1).expect("idx 1 corresponds to name");
                let quantity = match row.get(3).unwrap() {
                    0 => Quantity::Volume {
                        quantity: row.get(2).unwrap(),
                    },
                    1 => Quantity::Mass {
                        quantity: row.get(2).unwrap(),
                    },
                    2 => Quantity::Count {
                        quantity: row.get(2).unwrap(),
                        name: crate::common::quantity::CountName::Dash,
                    },
                    _ => panic!("Item inserted with invalid unit!"),
                };
                Ok(Item { id, name, quantity })
            })
            .unwrap();
        let mut items = Vec::new();
        for row in rows {
            match row {
                Ok(item) => items.push(item),
                Err(e) => eprint!("Error retrieving Items: {e}"),
            }
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    #[test]
    fn insert() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("bartend.db");
        let db = Database::new(file);
        let name = "test";
        let quantity = Quantity::Volume { quantity: 750.0 };

        let id = db.add_item(name, quantity);
        let item = db.get_item(id);

        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.id, id);
        assert_eq!(&item.name, name);
        assert_eq!(item.quantity, quantity);
    }
    #[test]
    fn update() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("bartend.db");
        let db = Database::new(file);
        let quantity = Quantity::Volume { quantity: 750.0 };
        let id = db.add_item("test", quantity);
        let mut item = db.get_item(id).unwrap();
        let new_name = "word".to_string();
        let new_quantity = Quantity::Mass { quantity: 600.0 };
        item.name = new_name.clone();
        item.quantity = new_quantity;

        db.update_item(item);
        let item = db.get_item(id);

        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.name, new_name);
        assert_eq!(item.quantity, new_quantity);
    }
    #[test]
    fn delete() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("bartend.db");
        let db = Database::new(file);
        let quantity = Quantity::Count {
            quantity: 2.0,
            name: crate::common::quantity::CountName::Dash,
        };

        let id = db.add_item("test", quantity);
        db.delete_item(id);
        let item = db.get_item(id);

        assert!(item.is_none())
    }
    #[test]
    fn get_all() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("bartend.db");
        let db = Database::new(file);
        let pre_items = vec![
            Item {
                id: ItemID(1),
                name: "test1".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
            },
            Item {
                id: ItemID(2),
                name: "test2".to_string(),
                quantity: Quantity::Volume { quantity: 375.0 },
            },
        ];
        _ = db.add_item(&pre_items[0].name, pre_items[0].quantity);
        _ = db.add_item(&pre_items[1].name, pre_items[1].quantity);

        let items = db.get_all_items();

        for item in items {
            assert!(pre_items.iter().any(
                |pre_item| &pre_item.name == &item.name && &pre_item.quantity == &item.quantity
            ));
        }
    }
}
