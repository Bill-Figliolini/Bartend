use std::collections::HashMap;

use rusqlite::Row;

use crate::{
    models::{Item, ItemBody, ItemID, Quantity},
    persistence::{
        DBError, ItemDB,
        repositories::{ItemRepository, Repository},
    },
};

impl<'a> ItemDB<'a> {
    fn from_db(row: &Row) -> Result<Item, rusqlite::Error> {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let quantity = Quantity::from_db(row.get(3)?, row.get(2)?);
        Ok(Item {
            id,
            body: ItemBody { name, quantity },
        })
    }
}

impl<'a> Repository for ItemDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS items(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    quantity REAL NOT NULL,
                    unit INTEGER NOT NULL
                );";
        self.connection.execute(query, ())?;
        Ok(())
    }
}

impl<'a> ItemRepository for ItemDB<'a> {
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
        Ok(rows.into_iter().fold(HashMap::new(), |mut acc, row| {
            match row {
                Ok(item) => acc.insert(item.id, item.body),
                Err(e) => panic!("Retrieving Items failled with error: {e}"),
            };
            acc
        }))
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::persistence::Database;

    use super::*;
    fn db_init() -> Database {
        let db = Database {
            connection: Connection::open_in_memory().unwrap(),
        };
        db.item_db().create_table().unwrap();
        db
    }

    #[test]
    fn table_creation_does_not_error() {
        let db = Database {
            connection: Connection::open_in_memory().unwrap(),
        };

        let result = db.item_db().create_table();

        assert!(result.is_ok());
        assert!(db.connection.table_exists(None, "items").unwrap())
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
    }
}
