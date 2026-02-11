mod schema;

use std::path::Path;

use crate::persistence::{Item, ItemID, Repository, sqlite::schema::Schema};
use rusqlite::{self, Connection};
use sql_query_builder as sql;

#[derive(Debug)]
pub struct DB {
    connection: Connection,
    items_schema: Schema,
}

impl DB {
    fn create_tables(connection: &Connection, items_schema: &Schema) {
        let create_items = sql::CreateTable::new()
            .create_table_if_not_exists(items_schema.name())
            .column(&format!(
                "{} INTEGER PRIMARY KEY",
                items_schema.columns()[0]
            ))
            .column(&format!("{} TEXT NOT NULL", items_schema.columns()[1]))
            .column(&format!("{} REAL NOT NULL", items_schema.columns()[2]))
            .as_string();
        let result = connection.execute(&create_items, ());
        if let Err(e) = result {
            panic!("DB Initialization error: {e}");
        }
    }
}
impl Repository for DB {
    fn new(path: impl AsRef<Path>) -> Self {
        let items_schema = Schema::new("items")
            .column("id")
            .column("name")
            .column("quantity");

        let connection = Connection::open(path);
        let connection = match connection {
            Ok(connection) => connection,
            Err(e) => {
                panic!("DB could not be opened! {e}")
            }
        };

        Self::create_tables(&connection, &items_schema);

        Self {
            connection,
            items_schema,
        }
    }

    fn add_item(&self, name: &str, quantity: f32) -> ItemID {
        let query = sql::Insert::new()
            .insert_into(&self.items_schema.get_autoinsert_statement())
            .values("(?1, ?2)")
            .debug()
            .as_string();
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
        let query = sql::Select::new()
            .select("*")
            .from(self.items_schema.name())
            .where_clause(&format!("id={id}"))
            .as_string();
        todo!()
    }

    fn get_all_items(&self) -> Vec<Item> {
        let query = sql::Select::new()
            .select(&self.items_schema.columns_string())
            .from(self.items_schema.name())
            .as_string();
        let mut stmt = self.connection.prepare(&query).unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(Item {
                    id: ItemID(row.get(0).unwrap()),
                    name: row.get(1).unwrap(),
                    quantity: row.get(2).unwrap(),
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

            let items_name = db.items_schema.name();
            assert!(db.connection.table_exists(None, items_name).unwrap());
            for column in db.items_schema.columns() {
                assert!(
                    db.connection
                        .column_exists(None, items_name, column)
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

            let id = db.add_item("test", 750.0);

            assert_eq!(id.0, 1);
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
