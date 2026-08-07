use std::collections::HashMap;

use crate::{
    models::{Category, CategoryBody, CategoryID, ItemID},
    persistence::{
        DBError,
        repositories::{
            CategoryDB, CategoryRepository, ItemMappingDB, ItemMappingRepository, Repository,
            graph::GraphDB,
        },
    },
};

use super::GraphRepository;

impl<'a> CategoryDB<'a> {
    #[must_use]
    pub fn mapping_db(&'a self) -> ItemMappingDB<'a> {
        ItemMappingDB {
            connection: self.connection,
        }
    }
}

impl<'a> Repository for CategoryDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS category(
                id INTEGER PRIMARY KEY,
                name STRING NOT NULL
            );";
        self.connection.execute(query, ())?;
        self.mapping_db().create_table()?;
        self.graph().create_table()?;
        Ok(())
    }
}
impl<'a> CategoryRepository for CategoryDB<'a> {
    fn graph(&self) -> impl GraphRepository {
        GraphDB {
            connection: self.connection,
        }
    }
    fn mapping(&self) -> impl ItemMappingRepository {
        self.mapping_db()
    }
    fn insert(&self, body: &CategoryBody) -> Result<CategoryID, DBError> {
        self.connection
            .execute("INSERT INTO category(name) VALUES(?1)", (&body.name,))?;
        Ok(CategoryID(self.connection.last_insert_rowid()))
    }
    fn update(&self, category: &Category) -> Result<(), DBError> {
        self.connection.execute(
            "
                UPDATE category SET
                name = ?2
                WHERE id = ?1
            ",
            (category.id.0, &category.body.name),
        )?;
        Ok(())
    }
    fn delete(&self, category: CategoryID) -> Result<(), DBError> {
        self.connection
            .execute("DELETE FROM category WHERE id=?1", (&category,))?;
        Ok(())
    }

    fn get_all(&self) -> Result<HashMap<CategoryID, CategoryBody>, DBError> {
        let query = "SELECT * FROM category";
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let id = row.get(0).unwrap();
            let name = row.get(1).unwrap();
            Ok(Category {
                id,
                body: CategoryBody { name },
            })
        })?;
        Ok(rows.into_iter().fold(HashMap::new(), |mut acc, row| {
            match row {
                Ok(item) => acc.insert(item.id, item.body),
                Err(e) => panic!("Retrieving Items failled with error: {e}"),
            };
            acc
        }))
    }

    fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, DBError> {
        self.mapping_db().get_map()
    }
}

#[cfg(test)]
mod test {
    use rusqlite::Connection;

    use super::*;
    use crate::persistence::Database;
    #[test]
    fn table_created_successfully() {
        let database = Database {
            connection: Connection::open_in_memory().unwrap(),
        };
        let table_name = "category";

        let result = database.category_db().create_table();

        assert!(result.is_ok());
        assert!(database.connection.table_exists(None, table_name).unwrap())
    }
}
