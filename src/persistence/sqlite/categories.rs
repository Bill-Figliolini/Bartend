use rusqlite::Connection;

use crate::{
    common::{
        category::{Category, CategoryID, CategoryManager},
        item::ItemID,
    },
    persistence::sqlite::DB,
};

pub(super) fn create_category_tables(connection: &Connection) {
    let create_category = "
        CREATE TABLE IF NOT EXISTS category(
            id INTEGER PRIMARY KEY,
            name STRING NOT NULL
        );";
    let category_result = connection.execute(create_category, ());
    if let Err(e) = category_result {
        panic!("Category table creation failed with error: {e}");
    }
    let create_category_graph = "
        CREATE TABLE IF NOT EXISTS category_relations(
            parent_id INTEGER,
            child_id INTEGER,
            UNIQUE (parent_id, child_id)
        );";
    let graph_result = connection.execute(create_category_graph, ());
    if let Err(e) = graph_result {
        panic!("Graph table creation failed with error: {e}");
    }
    let create_category_item_table = "
        CREATE TABLE IF NOT EXISTS category_item_mapping(
            category_id INTEGER,
            item_id INTEGER,
            UNIQUE(category_id, item_id)
        );";
    let mapping_result = connection.execute(create_category_item_table, ());
    if let Err(e) = mapping_result {
        panic!("Graph table creation failed with error: {e}");
    }
}

impl DB {
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
    pub fn update_category(&self, category: Category) {
        todo!()
    }
    pub fn add_category_relation(&self, parent: CategoryID, child: CategoryID) {
        todo!()
    }
    pub fn delete_category_relation(&self, parent: CategoryID, child: CategoryID) {
        todo!()
    }
    pub fn add_item_to_category(&self, category: Category, item: ItemID) {
        todo!()
    }
    pub fn remove_item_from_category(&self, category: Category, item: ItemID) {
        todo!()
    }
    pub fn get_categories(&self) -> CategoryManager {
        todo!()
    }
}
