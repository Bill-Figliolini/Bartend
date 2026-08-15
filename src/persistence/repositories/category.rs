use std::collections::{HashMap, HashSet};

use rusqlite::{Connection, Row};

use crate::{
    logic::GraphPatch,
    models::{Category, CategoryBody, CategoryID, ItemID},
    persistence::{
        DBError,
        repositories::{CategoryDB, CategoryRepository},
    },
};

impl<'a> CategoryDB<'a> {
    fn from_db(row: &Row) -> Result<Category, rusqlite::Error> {
        let id = row.get(0)?;
        let name = row.get(1)?;
        Ok(Category {
            id,
            body: CategoryBody { name },
        })
    }
    fn internal_relation_insert(
        connection: &Connection,
        parent: &CategoryID,
        child: &CategoryID,
    ) -> Result<(), DBError> {
        let query = "INSERT INTO graph(parent_id, child_id) VALUES (?1, ?2);";
        connection.execute(query, (parent, child))?;
        Ok(())
    }
}

impl<'a> CategoryDB<'a> {
    pub(in crate::persistence) fn create_table(&self) -> Result<(), DBError> {
        let category_schema = "CREATE TABLE IF NOT EXISTS category(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );";
        self.connection.execute(category_schema, ())?;
        let graph_schema = "CREATE TABLE IF NOT EXISTS graph(
                    parent_id INTEGER,
                    child_id INTEGER,
                    FOREIGN KEY (parent_id) REFERENCES category(id) ON DELETE CASCADE,
                    FOREIGN KEY (child_id) REFERENCES category(id) ON DELETE CASCADE,
                UNIQUE (parent_id, child_id));";
        self.connection.execute(graph_schema, ())?;
        let map_schema = "CREATE TABLE IF NOT EXISTS category_item(
                    category_id INTEGER,
                    item_id INTEGER,
                    FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE CASCADE,
                    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
                    UNIQUE(category_id, item_id)
                )";
        self.connection.execute(map_schema, ())?;
        Ok(())
    }
}
impl<'a> CategoryRepository for CategoryDB<'a> {
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
        let table: Result<HashMap<CategoryID, CategoryBody>, rusqlite::Error> = rows
            .into_iter()
            .map(|result| result.map(|category| (category.id, category.body)))
            .collect();
        Ok(table?)
    }
    fn get_graph(&self) -> Result<HashMap<CategoryID, HashSet<CategoryID>>, DBError> {
        let query = "SELECT parent_id, child_id FROM graph;";
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))?;

        Ok(rows.into_iter().fold(HashMap::new(), |mut acc, row| {
            match row {
                Ok((parent_id, child_id)) => {
                    let parent_entry = acc.entry(parent_id).or_default();
                    parent_entry.insert(child_id);
                    acc.entry(child_id).or_insert_with(|| HashSet::new());
                }
                Err(e) => panic!("{}", e),
            }
            acc
        }))
    }

    fn insert_relation(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError> {
        CategoryDB::internal_relation_insert(self.connection, &parent, &child)
    }

    fn delete_node(&self, patch: &GraphPatch<CategoryID>) -> Result<(), DBError> {
        let query = "DELETE FROM graph WHERE parent_id=?1 OR child_id=?1";
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(query, (patch.to_remove,))?;
        if let Some(ref children) = patch.children
            && let Some(ref parents) = patch.parents
        {
            for parent in parents {
                for child in children {
                    CategoryDB::internal_relation_insert(&transaction, parent, child)?
                }
            }
        }
        transaction.commit()?;
        Ok(())
    }

    fn delete_edge(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError> {
        let query = "DELETE FROM graph WHERE parent_id = ?1 AND child_id = ?2;";
        self.connection.execute(query, (parent, child))?;
        Ok(())
    }

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
    fn map_insert(&self, item_id: &ItemID, category_id: &CategoryID) -> Result<(), DBError> {
        self.connection.execute(
            "INSERT INTO category_item(category_id, item_id) VALUES (?1, ?2)",
            (category_id, item_id),
        )?;
        Ok(())
    }
    fn map_delete(&self, item: &ItemID, category: &CategoryID) -> Result<(), DBError> {
        self.connection.execute(
            "DELETE FROM category_item WHERE item_id = ?1 AND category_id = ?2",
            (item, category),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::persistence::Database;

    fn db_init() -> Database {
        Database::new(Connection::open_in_memory().unwrap()).unwrap()
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
    mod category {
        use super::*;
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
    mod graph {
        use super::*;

        mod insert {
            use super::*;

            #[test]
            fn inserts_row_into_db() {
                let db = db_init();
                let parent_id = db
                    .category_db()
                    .insert(&CategoryBody {
                        name: "test".to_string(),
                    })
                    .unwrap();
                let child_id = db
                    .category_db()
                    .insert(&CategoryBody {
                        name: "Test 2".to_string(),
                    })
                    .unwrap();

                let result = db.category_db().insert_relation(parent_id, child_id);
                assert!(result.is_ok());

                let in_db: Result<(CategoryID, CategoryID), rusqlite::Error> =
                    db.connection.query_one(
                        "SELECT * FROM graph WHERE parent_id=?1 AND child_id=?2",
                        (parent_id, child_id),
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    );
                assert!(in_db.is_ok());
                let in_db = in_db.unwrap();
                assert_eq!(in_db.0, parent_id);
                assert_eq!(in_db.1, child_id);
            }
        }

        mod delete {
            use super::*;
            fn line_graph_builder(db: &Database, length: usize) -> Vec<CategoryID> {
                let mut ids = Vec::with_capacity(length);
                for i in 0..=length {
                    let category = CategoryBody {
                        name: i.to_string(),
                    };
                    let id = db.category_db().insert(&category).unwrap();
                    if let Some(last_id) = ids.last() {
                        db.category_db().insert_relation(*last_id, id).unwrap();
                    }
                    ids.push(id);
                }
                ids
            }

            #[test]
            fn edge_delete_only_effects_edge() {
                //Set up
                let db = db_init();
                let length = 4;
                let ids = line_graph_builder(&db, length);
                //Initial state
                let initial_edge_result = db.category_db().get_graph();
                assert!(initial_edge_result.is_ok());
                let inital_edges = initial_edge_result.unwrap();
                for i in 0..length {
                    assert!(*&inital_edges.get(&ids[i]).unwrap().contains(&ids[i + 1]));
                }

                let delete_result = db.category_db().delete_edge(ids[1], ids[2]);
                assert!(delete_result.is_ok());

                let after_edges = db.category_db().get_graph().unwrap();
                for id in ids.iter() {
                    assert!(after_edges.keys().any(|key| key == id));
                }
                assert!(after_edges.get(&ids[0]).unwrap().contains(&ids[1]));
                assert!(!after_edges.get(&ids[1]).unwrap().contains(&ids[2]));
                assert!(after_edges.get(&ids[2]).unwrap().contains(&ids[3]));
            }

            #[test]
            fn node_delete_removes_all_trace() {
                //Set up
                let db = db_init();
                let length = 5;
                let ids = line_graph_builder(&db, length);
                let initial_edge_result = db.category_db().get_graph();
                assert!(initial_edge_result.is_ok());
                let inital_edges = initial_edge_result.unwrap();
                for i in 0..length {
                    assert!(*&inital_edges.get(&ids[i]).unwrap().contains(&ids[i + 1]));
                }
                let patch = GraphPatch {
                    to_remove: ids[2],
                    parents: Some(HashSet::from([ids[1]])),
                    children: Some(HashSet::from([ids[3]])),
                };

                let delete_result = db.category_db().delete_node(&patch);
                assert!(delete_result.is_ok());

                let after_edges = db.category_db().get_graph().unwrap();
                for edge in after_edges {
                    assert_ne!(edge.0, ids[2]);
                    assert!(!edge.1.contains(&ids[2]));
                }
            }
        }
    }

    mod map {
        use super::*;
        use crate::{
            models::{ItemBody, Quantity},
            persistence::repositories::ItemRepository,
        };
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

                let result = db.category_db().map_insert(&item_id, &category_id);

                assert!(result.is_ok());

                let in_db_result = db.category_db().get_map();
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

                let result = db.category_db().map_insert(&item_id, &category_id);

                assert!(result.is_ok());

                db.item_db().delete(item_id).unwrap();

                let in_db = db.category_db().get_map().unwrap();

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

                let result = db.category_db().map_insert(&item_id, &category_id);

                assert!(result.is_ok());

                db.category_db().delete(category_id).unwrap();

                let in_db = db.category_db().get_map().unwrap();

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

                let result = db.category_db().map_insert(&item_id, &category_id);
                assert!(result.is_ok());

                let result = db.category_db().map_delete(&item_id, &category_id);
                assert!(result.is_ok());

                let in_db = db.category_db().get_map().unwrap();

                assert!(in_db.get(&item_id).is_none())
            }
        }
    }
}
