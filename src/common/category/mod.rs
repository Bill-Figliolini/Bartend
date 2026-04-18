use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use rusqlite::{ToSql, params, types::FromSql};

use crate::{
    common::category::graph::{DirectedAcyclicGraph, GraphError},
    persistence::{DB, DBCreate, DBUnit},
};

mod graph;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryID(pub i64);
#[derive(Debug, Clone)]
pub struct Category {
    id: CategoryID,
    name: String,
}
#[derive(Debug)]
pub struct CategoryManager {
    relations: DirectedAcyclicGraph<CategoryID>,
    names: HashMap<CategoryID, String>,
}
impl CategoryManager {
    pub fn new() -> Self {
        Self {
            relations: DirectedAcyclicGraph::build_from(&[], &[]).unwrap(),
            names: HashMap::new(),
        }
    }
    fn read_categories(&mut self, db: &DB) {
        let query = "
                SELECT * FROM category WHERE id=?1
            ";
        let mut stmt = db
            .connection
            .prepare(query)
            .expect("Query must be valid SQL");
        let rows = stmt
            .query_map([], |row| {
                let id = row.get(0).unwrap();
                let name = row.get(1).unwrap();
                Ok(Category { id, name })
            })
            .unwrap();
        rows.fold(Vec::new(), |mut acc, category| {
            match category {
                Ok(category) => acc.push(category),
                Err(e) => panic!("Error Reading Categories: {e}"),
            }
            acc
        });
    }
    pub fn get_children(&self, id: &CategoryID) -> HashSet<CategoryID> {
        self.relations.get_all_children(id).unwrap_or_default()
    }
    pub fn get_categories(&self) -> Vec<Category> {
        let ids = self.relations.get_vertices();
        let categories = ids
            .into_iter()
            .map(|id| Category::new(id, self.names.get(&id).unwrap().clone()))
            .collect();
        categories
    }
    pub fn remove_category(&mut self, db: &DB, id: CategoryID) {
        self.names.remove(&id);
        self.relations.remove(id); //Perhaps relations should return a full list of additions?
        //needs more. a full commit of the new relations db as well.
        db.delete_category(id);
    }
    pub fn add_category(&mut self, db: &DB, name: String) {
        let id = db.add_category(name.clone());
        self.names.insert(id, name);
        self.relations.insert_vertex(id);
    }
    pub fn add_relation(
        &mut self,
        db: &DB,
        parent: &CategoryID,
        child: &CategoryID,
    ) -> Result<(), GraphError> {
        match self.relations.insert_edge((parent, child)) {
            Ok(()) => {
                //db.add_category_relation(*parent, *child);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
impl Category {
    fn new(id: CategoryID, name: String) -> Self {
        Self { id, name }
    }
    pub fn id(&self) -> CategoryID {
        self.id
    }
    pub fn test_cat() -> Self {
        Category {
            id: CategoryID(1),
            name: "test".to_string(),
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl ToSql for CategoryID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for CategoryID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_i64()?;
        Ok(CategoryID(value))
    }
}

impl DBCreate for Category {
    fn create(db: &DB) {
        let query = "
            CREATE TABLE IF NOT EXISTS category(
                id INTEGER PRIMARY KEY,
                name STRING NOT NULL
            );";
        if let Err(e) = db.connection.execute(query, ()) {
            panic!("Category table creation failed with error: {e}");
        };
    }
}
impl DBUnit for Category {
    fn update(self, db: &DB) {
        let query = "
            UPDATE category SET
            name = ?2
            WHERE id = ?1
        ";
        if let Err(e) = db.connection.execute(query, (self.id, self.name.clone())) {
            panic!("Update category failed with error: {e}");
        }
    }

    fn delete(self, db: &DB) {
        let query = "
            DELETE * FROM category WHERE id=?1
        ";
        if let Err(e) = db.connection.execute(query, (self.id,)) {
            panic!("Error deleting category: {e}");
        }
    }
}

impl DBCreate for CategoryManager {
    fn create(db: &DB) {
        Category::create(db);
        DirectedAcyclicGraph::<CategoryID>::create(db);

        let create_category_item_table = "
            CREATE TABLE IF NOT EXISTS category_item_mapping(
                category_id INTEGER,
                item_id INTEGER,
                UNIQUE(category_id, item_id)
            );
        ";
        if let Err(e) = db.connection.execute(create_category_item_table, ()) {
            panic!("Graph table creation failed with error: {e}");
        }
    }
}
