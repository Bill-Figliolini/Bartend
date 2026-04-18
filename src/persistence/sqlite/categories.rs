use crate::{
    common::category::{CategoryID, CategoryManager},
    persistence::{DBCreate, sqlite::Database},
};

pub(super) fn create_category_tables(db: &Database) {
    CategoryManager::create(db);
}

impl Database {
    pub fn add_category(&self, name: String) -> CategoryID {
        let query = "INSERT INTO category(name) VALUES (?1)";
        match self.connection.execute(query, (name,)) {
            Ok(_) => CategoryID(self.connection.last_insert_rowid()),
            Err(e) => panic!("Error inserting into category: {e}"),
        }
    }
    pub fn delete_category(&self, id: CategoryID) {
        let query = "DELETE FROM category WHERE id=?1";
        if let Err(e) = self.connection.execute(query, (id.0,)) {
            panic!("Error deleting category: {e}");
        }
    }
}
