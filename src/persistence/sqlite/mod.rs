mod schema;

use std::path::Path;

use crate::{
    common::item::{Item, ItemID},
    persistence::Repository,
};
use rusqlite::{self, Connection, OptionalExtension};

#[derive(Debug)]
pub struct DB {
    connection: Connection,
}

impl DB {
    fn create_tables(connection: &Connection) {
        let create_units = "CREATE TABLE IF NOT EXISTS units(
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
            )"
        .to_string();
        let result = connection.execute(&create_units, ());
        if let Err(e) = result {
            panic!("DB Initialization error: {e}");
        }
        let unit_insert = "INSERT INTO units(name) VALUES (?1)";
        let units = [
            "Volume".to_string(),
            "Mass".to_string(),
            "Dashes".to_string(),
        ];
        for unit in units {
            let result = connection.execute(&unit_insert, (unit,));
            if let Err(e) = result {
                panic!("DB Initialization error: {e}");
            }
        }
        let create_items = "CREATE TABLE IF NOT EXISTS items(
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            quantity REAL NOT NULL
            );"
        .to_string();
        let result = connection.execute(&create_items, ());
        if let Err(e) = result {
            panic!("DB Initialization error: {e}");
        }
    }
}
impl Repository for DB {
    fn new(path: impl AsRef<Path>) -> Self {
        let connection = match Connection::open(path) {
            Ok(connection) => connection,
            Err(e) => {
                panic!("DB could not be opened! {e}")
            }
        };

        Self::create_tables(&connection);

        Self { connection }
    }

    fn add_item(&self, name: &str, quantity: f32) -> ItemID {
        let query = format!("INSERT INTO items(name, quantity) VALUES (?1, ?2)",);
        let result = self.connection.execute(&query, (name, quantity));
        match result {
            Ok(_) => ItemID(self.connection.last_insert_rowid()),
            Err(e) => {
                panic!("Item Insertion Error: {e}");
            }
        }
    }

    fn get_item(&self, id: ItemID) -> Option<Item> {
        let id = id.0;
        let query = format!("SELECT * FROM items WHERE id = ?1");
        self.connection
            .query_row(&query, [(id)], |row| {
                Ok(Item {
                    id: ItemID(row.get(0).unwrap()),
                    name: row.get(1).unwrap(),
                    quantity: row.get(2).unwrap(),
                })
            })
            .optional()
            .unwrap()
    }

    fn update_item(&self, item: Item) {
        let id = item.id.0;
        let query = format!(
            "UPDATE items SET
            name = ?2,
            quantity = ?3
            WHERE id = ?1"
        );

        if let Err(e) = self
            .connection
            .execute(&query, (id, item.name, item.quantity))
        {
            panic!("Update item failed with error: {e}");
        }
    }

    fn delete_item(&self, id: ItemID) {
        let id = id.0;
        let query = "DELETE FROM items WHERE id = ?1".to_string();

        if let Err(e) = self.connection.execute(&query, (id,)) {
            panic!("Delete_item failed with error: {e}");
        }
    }

    fn get_all_items(&self) -> Vec<Item> {
        let query = "SELECT * FROM items".to_string();
        let mut stmt = self
            .connection
            .prepare(&query)
            .expect("query must be valid sql");
        let rows = stmt
            .query_map([], |row| {
                Ok(Item {
                    id: ItemID(row.get(0).expect("idx 0 corresponds to id")),
                    name: row.get(1).expect("idx 1 corresponds to name"),
                    quantity: row.get(2).expect("idx 2 corresponds to quantity"),
                })
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
mod test {
    use super::*;
    use tempfile::TempDir;
    mod table_creation {
        use super::*;
        #[test]
        fn items() {
            let dir = TempDir::new().unwrap();
            let file = dir.path().join("bartend.db");
            let db = DB::new(file);
            let items_name = "items";
            let columns = vec!["id", "name", "quantity"];

            assert!(db.connection.table_exists(None, items_name).unwrap());
            for column in columns {
                assert!(
                    db.connection
                        .column_exists(None, items_name, &column)
                        .unwrap()
                )
            }
        }
    }
    mod items {
        use super::*;
        #[test]
        fn insert() {
            let dir = TempDir::new().unwrap();
            let file = dir.path().join("bartend.db");
            let db = DB::new(file);
            let name = "test";
            let quantity = 750.0;

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
            let db = DB::new(file);
            let id = db.add_item("test", 750.0);
            let mut item = db.get_item(id).unwrap();
            let new_name = "word".to_string();
            let new_quantity = 600.0;
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
            let db = DB::new(file);

            let id = db.add_item("test", 750.0);
            db.delete_item(id);
            let item = db.get_item(id);

            assert!(item.is_none())
        }
        #[test]
        fn get_all() {
            let dir = TempDir::new().unwrap();
            let file = dir.path().join("bartend.db");
            let db = DB::new(file);
            let pre_items = vec![
                Item {
                    id: ItemID(1),
                    name: "test1".to_string(),
                    quantity: 750.0,
                },
                Item {
                    id: ItemID(2),
                    name: "test2".to_string(),
                    quantity: 375.0,
                },
            ];
            _ = db.add_item(&pre_items[0].name, pre_items[0].quantity);
            _ = db.add_item(&pre_items[1].name, pre_items[1].quantity);

            let items = db.get_all_items();

            for item in items {
                assert!(
                    pre_items.iter().any(|pre_item| &pre_item.name == &item.name
                        && &pre_item.quantity == &item.quantity)
                );
            }
        }
    }
}
