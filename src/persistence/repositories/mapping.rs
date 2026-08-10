use std::collections::HashMap;

use crate::{
    models::{CategoryID, ItemID},
    persistence::{
        DBError,
        repositories::{ItemMappingDB, ItemMappingRepository, Repository},
    },
};
impl<'a> Repository for ItemMappingDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS category_item(
            category_id INTEGER,
            item_id INTEGER,
            FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE CASCADE,
            FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
            UNIQUE(category_id, item_id)
        )";
        self.connection.execute(query, ())?;
        Ok(())
    }
}

impl<'a> ItemMappingRepository for ItemMappingDB<'a> {
    fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, DBError> {
        let mut stmt = self
            .connection
            .prepare("SELECT item_id, category_id FROM category_item;")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.into_iter().fold(HashMap::new(), |mut acc, row| {
            match row {
                Ok((item_id, category_id)) => acc.insert(item_id, category_id),
                Err(e) => panic!("error retrieving item mapping: {e}"),
            };
            acc
        }))
    }
    fn insert(&self, item_id: &ItemID, category_id: &CategoryID) -> Result<(), DBError> {
        self.connection.execute(
            "INSERT INTO category_item(category_id, item_id) VALUES (?1, ?2)",
            (category_id, item_id),
        )?;
        Ok(())
    }
    fn delete(&self, item: &ItemID, category: &CategoryID) -> Result<(), DBError> {
        self.connection.execute(
            "DELETE FROM category_item WHERE item_id = ?1 AND category_id = ?2",
            (item, category),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //Only table creation, as the other functions are dependent on other tables.
    // Will be included in Persistance integration tests
    use rusqlite::Connection;

    use crate::{
        models::{CategoryBody, ItemBody, Quantity},
        persistence::{
            Database,
            repositories::{CategoryRepository, ItemMappingRepository, ItemRepository, Repository},
        },
    };

    fn db_init() -> Database {
        Database::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn table_creation_not_error() {
        let database = Database {
            connection: Connection::open_in_memory().unwrap(),
        };

        let result = database.category_db().mapping().create_table();

        assert!(result.is_ok());
        assert!(
            database
                .connection
                .table_exists(None, "category_item")
                .unwrap()
        )
    }
    mod insertion {
        use super::*;
        #[test]
        fn writes_to_db() {
            let db = db_init();
            let item = ItemBody {
                name: "test".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
            };
            let category = CategoryBody {
                name: "testcat".to_string(),
            };
            let item_id = db.item_db().insert(&item).unwrap();
            let category_id = db.category_db().insert(&category).unwrap();

            let result = db.category_db().mapping().insert(&item_id, &category_id);

            assert!(result.is_ok());

            let in_db_result = db.category_db().mapping().get_map();
            assert!(in_db_result.is_ok());
            let in_db = in_db_result.unwrap();
            let db_map = in_db.get(&item_id);
            assert!(db_map.is_some());
            assert_eq!(db_map.unwrap(), &category_id);
        }
    }
    mod foreign_key {
        use super::*;
        #[test]
        fn row_deleted_on_item_delete() {
            let db = db_init();
            let item = ItemBody {
                name: "test".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
            };
            let category = CategoryBody {
                name: "testcat".to_string(),
            };
            let item_id = db.item_db().insert(&item).unwrap();
            let category_id = db.category_db().insert(&category).unwrap();

            let result = db.category_db().mapping().insert(&item_id, &category_id);

            assert!(result.is_ok());

            db.item_db().delete(item_id).unwrap();

            let in_db = db.category_db().mapping().get_map().unwrap();

            assert!(in_db.get(&item_id).is_none())
        }

        #[test]
        fn row_deleted_on_category_delete() {
            let db = db_init();
            let item = ItemBody {
                name: "test".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
            };
            let category = CategoryBody {
                name: "testcat".to_string(),
            };
            let item_id = db.item_db().insert(&item).unwrap();
            let category_id = db.category_db().insert(&category).unwrap();

            let result = db.category_db().mapping().insert(&item_id, &category_id);

            assert!(result.is_ok());

            db.category_db().delete(category_id).unwrap();

            let in_db = db.category_db().mapping().get_map().unwrap();

            assert!(in_db.get(&item_id).is_none())
        }
    }
    mod delete {
        use super::*;

        #[test]
        fn deletes_row_in_db() {
            let db = db_init();
            let item = ItemBody {
                name: "test".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
            };
            let category = CategoryBody {
                name: "testcat".to_string(),
            };
            let item_id = db.item_db().insert(&item).unwrap();
            let category_id = db.category_db().insert(&category).unwrap();

            let result = db.category_db().mapping().insert(&item_id, &category_id);
            assert!(result.is_ok());

            let result = db.category_db().mapping().delete(&item_id, &category_id);
            assert!(result.is_ok());

            let in_db = db.category_db().mapping().get_map().unwrap();

            assert!(in_db.get(&item_id).is_none())
        }
    }
}
