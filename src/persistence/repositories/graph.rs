use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use crate::{
    models::CategoryID,
    persistence::{
        DBError,
        repositories::{GraphRepository, Repository},
    },
};

pub struct GraphDB<'a> {
    pub connection: &'a Connection,
}

impl<'a> Repository for GraphDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let creation = "CREATE TABLE IF NOT EXISTS graph(
            parent_id INTEGER,
            child_id INTEGER,
            FOREIGN KEY (parent_id) REFERENCES category(id) ON DELETE CASCADE,
            FOREIGN KEY (child_id) REFERENCES category(id) ON DELETE CASCADE,
        UNIQUE (parent_id, child_id));";
        self.connection.execute(creation, ())?;
        Ok(())
    }
}

impl<'a> GraphRepository for GraphDB<'a> {
    fn get(&self) -> Result<HashMap<CategoryID, HashSet<CategoryID>>, DBError> {
        let query = "SELECT parent_id, child_id FROM graph;";
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))?;

        Ok(rows.into_iter().fold(HashMap::new(), |mut acc, row| {
            match row {
                Ok((parent_id, child_id)) => {
                    let parent_entry = acc.entry(parent_id).or_default();
                    parent_entry.insert(child_id);
                    if !acc.contains_key(&child_id) {
                        acc.insert(child_id, HashSet::new());
                    }
                }
                Err(e) => panic!("{}", e),
            }
            acc
        }))
    }

    fn insert(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError> {
        let query = "INSERT INTO graph(parent_id, child_id) VALUES (?1, ?2);";
        self.connection.execute(query, (parent, child))?;
        Ok(())
    }

    fn delete_node(&self, node: CategoryID) -> Result<(), DBError> {
        let query = "DELETE FROM graph WHERE parent_id=?1 OR child_id=?1";
        self.connection.execute(query, (node,))?;
        Ok(())
    }

    fn delete_edge(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError> {
        let query = "DELETE FROM graph WHERE parent_id = ?1 AND child_id = ?2;";
        self.connection.execute(query, (parent, child))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    use crate::{
        models::CategoryBody,
        persistence::{Database, repositories::CategoryRepository},
    };

    fn db_init() -> Database {
        Database::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn creates_without_failure() {
        let db = Database {
            connection: Connection::open_in_memory().unwrap(),
        };

        let result = db.category_db().graph().create_table();

        assert!(result.is_ok());
    }

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

            let result = db.category_db().graph().insert(parent_id, child_id);
            assert!(result.is_ok());

            let in_db: Result<(CategoryID, CategoryID), rusqlite::Error> = db.connection.query_one(
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
                    db.category_db().graph().insert(*last_id, id).unwrap();
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
            let initial_edge_result = db.category_db().graph().get();
            assert!(initial_edge_result.is_ok());
            let inital_edges = initial_edge_result.unwrap();
            for i in 0..length {
                assert!(*&inital_edges.get(&ids[i]).unwrap().contains(&ids[i + 1]));
            }

            let delete_result = db.category_db().graph().delete_edge(ids[1], ids[2]);
            assert!(delete_result.is_ok());

            let after_edges = db.category_db().graph().get().unwrap();
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
            let initial_edge_result = db.category_db().graph().get();
            assert!(initial_edge_result.is_ok());
            let inital_edges = initial_edge_result.unwrap();
            for i in 0..length {
                assert!(*&inital_edges.get(&ids[i]).unwrap().contains(&ids[i + 1]));
            }

            let delete_result = db.category_db().graph().delete_node(ids[2]);
            assert!(delete_result.is_ok());

            let after_edges = db.category_db().graph().get().unwrap();
            for edge in after_edges {
                assert_ne!(edge.0, ids[2]);
                assert!(!edge.1.contains(&ids[2]));
            }
        }
    }
}
