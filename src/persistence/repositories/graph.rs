use rusqlite::Connection;

use crate::persistence::repositories::{GraphRepository, Repository};

pub struct GraphDB<'a> {
    pub connection: &'a Connection,
}

impl<'a> Repository for GraphDB<'a> {
    fn create_table(&self) -> Result<(), crate::persistence::DBError> {
        let creation = "CREATE TABLE IF NOT EXISTS category(
            parent_id INTEGER,
            child_id INTEGER,
            FOREIGN KEY (parent_id) REFERENCES category(id) ON DELETE CASCADE,
            FOREIGN KEY (child_id) REFERENCES category(id) ON DELETE CASCADE,
        UNIQUE (parent_id, child_id));";
        self.connection.execute(creation, ())?;
        Ok(())
    }
}

impl<'a> GraphRepository for GraphDB<'a> {}
