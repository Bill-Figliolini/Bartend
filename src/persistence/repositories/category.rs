use std::collections::HashMap;

use rusqlite::Row;

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
    fn from_db(row: &Row) -> Result<Category, rusqlite::Error> {
        let id = row.get(0)?;
        let name = row.get(1)?;
        Ok(Category {
            id,
            body: CategoryBody { name },
        })
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
        let rows = stmt.query_map([], CategoryDB::from_db)?;
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
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::persistence::Database;

    fn db_init() -> Database {
        let db = Database {
            connection: Connection::open_in_memory().unwrap(),
        };
        db.category_db().create_table().unwrap();
        db.item_db().create_table().unwrap();
        db
    }

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
    mod insertion {
        use super::*;
        #[test]
        fn returns_id_of_entry() {
            let db = db_init();
            let category = CategoryBody {
                name: "Test".to_string(),
            };

            let result = db.category_db().insert(&category);
            assert!(result.is_ok());
            let id = result.unwrap();

            let in_db =
                db.connection
                    .query_one("SELECT * FROM category WHERE id=?1", (id,), |row| {
                        CategoryDB::from_db(row)
                    });
            assert!(in_db.is_ok());
            assert_eq!(in_db.unwrap().body, category);
        }
    }
    mod update {
        use super::*;
        #[test]
        fn updates_value_in_db() {
            let db = db_init();
            let old_category = CategoryBody {
                name: "Old".to_string(),
            };
            let new_category = CategoryBody {
                name: "New".to_string(),
            };
            let result = db.category_db().insert(&old_category);

            assert!(result.is_ok());
            let id = result.unwrap();

            let _ = db.category_db().update(&Category {
                id,
                body: new_category.clone(),
            });

            let in_db =
                db.connection
                    .query_one("SELECT * FROM category WHERE id=?1", (id,), |row| {
                        CategoryDB::from_db(row)
                    });
            assert!(in_db.is_ok());
            assert_eq!(in_db.unwrap().body, new_category);
        }
    }
    mod delete {
        use super::*;
        use rusqlite::OptionalExtension;
        #[test]
        fn does_not_have_error() {
            let db = db_init();
            let category = CategoryBody {
                name: "test".to_string(),
            };
            let id = db.category_db().insert(&category).unwrap();

            let result = db.category_db().delete(id);
            eprintln!("{:?}", result);
            assert!(result.is_ok())
        }
        #[test]
        fn removes_from_db() {
            let db = db_init();
            let category = CategoryBody {
                name: "test".to_string(),
            };
            let result = db.category_db().insert(&category);
            assert!(result.is_ok());
            let id = result.unwrap();

            let _ = db.category_db().delete(id);

            let in_db = db
                .connection
                .query_one("SELECT * FROM category WHERE id=?1", (id,), |row| {
                    CategoryDB::from_db(row)
                })
                .optional();
            assert!(in_db.is_ok());
            assert!(in_db.unwrap().is_none())
        }
    }

    mod get_all {
        use super::*;
        #[test]
        fn gets_all_from_db() {
            let test_size = 100;
            let db = db_init();
            let mut ids = Vec::new();
            let categories: Vec<CategoryBody> = (0..test_size)
                .map(|num| CategoryBody {
                    name: format!("test{num}"),
                })
                .collect();
            for category in categories.iter() {
                ids.push(db.category_db().insert(category).unwrap());
            }

            let result = db.category_db().get_all();
            assert!(result.is_ok());
            let in_db = result.unwrap();
            assert_eq!(in_db.len(), ids.len());

            for id in ids {
                assert!(in_db.get(&id).is_some());
                let category = in_db.get(&id).unwrap();
                assert!(categories.contains(category));
            }
        }
    }
}
