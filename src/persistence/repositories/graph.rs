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
}
