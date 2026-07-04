use std::collections::HashMap;

use rusqlite::OptionalExtension;

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
