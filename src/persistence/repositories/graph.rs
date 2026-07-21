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
                    if acc.contains_key(&parent_id) {
                        let bucket = acc.get_mut(&parent_id).expect("confirmed to exist");
                        bucket.insert(child_id);
                    } else {
                        let mut new_set = HashSet::new();
                        new_set.insert(child_id);
                        acc.insert(parent_id, new_set);
                    }
                }
                Err(e) => panic!("{}", e),
            }
            acc
        }))
    }

    fn insert(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError> {
        let query = "INSERT INTO graph(parent_id, child_id VALUES (?1, ?2);";
        self.connection.execute(query, (parent, child))?;
        Ok(())
    }

    fn delete_node(&self, node: CategoryID) -> Result<(), DBError> {
        let query = "DELETE FROM graph WHERE parent_id=?1;";
        self.connection.execute(query, (node,))?;
        Ok(())
    }

    fn delete_edge(&self, parent: CategoryID, child: CategoryID) -> Result<(), DBError> {
        let query = "DELETE FROM graph WHERE parent_id = ?1 AND child_id = ?2;";
        self.connection.execute(query, (parent, child))?;
        Ok(())
    }
}
