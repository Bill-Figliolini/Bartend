use std::{collections::HashSet, fmt::Display};

use rusqlite::{ToSql, types::FromSql};

use crate::{
    logic::category::graph::{DirectedAcyclicGraph, GraphError},
    persistence::Database,
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
}
impl CategoryManager {
    pub fn new() -> Self {
        let relations = DirectedAcyclicGraph::build_from(&[], &[]).unwrap();
        Self { relations }
    }

    fn read_categories(&mut self) {
        todo!()
        /*let query = "
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
        });*/
    }
    pub fn get_children(&self, id: &CategoryID) -> HashSet<CategoryID> {
        self.relations.get_all_children(id).unwrap_or_default()
    }
    pub fn get_categories(&self) -> Vec<Category> {
        todo!()
    }
    pub fn remove_category(&mut self, id: CategoryID, db: &Database) {
        todo!()
    }
    pub fn add_category(&mut self, name: String, db: &Database) -> CategoryID {
        todo!();
    }
    pub fn add_relation(
        &mut self,
        parent: &CategoryID,
        child: &CategoryID,
        db: &Database,
    ) -> Result<(), GraphError> {
        match self.relations.insert_edge((parent, child)) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
impl Category {
    fn new(id: CategoryID, name: String) -> Self {
        Self { id, name }
    }

    pub fn insert(input: Category) {
        todo!()
    }

    pub fn id(&self) -> CategoryID {
        self.id
    }
    pub fn create() -> String {
        "CREATE TABLE IF NOT EXISTS category(
            id INTEGER PRIMARY KEY,
            name STRING NOT NULL
        );"
        .to_string()
    }
    pub fn update(&self, db: &Database) {
        if let Err(e) = db.connection.execute(
            "
            UPDATE category SET
            name = ?2
            WHERE id = ?1
        ",
            (self.id.0, self.name.clone()),
        ) {
            panic!("Error Updating Category: {e}");
        };
    }
    pub fn delete(self) -> String {
        format!("DELETE * FROM category WHERE id={}", self.id.0)
    }
}

impl CategoryManager {
    pub fn create() -> Vec<String> {
        let mut stmts = Vec::new();
        stmts.push(Category::create());
        stmts.push(DirectedAcyclicGraph::<CategoryID>::create());

        let query = "CREATE TABLE IF NOT EXISTS category_item_mapping(
                category_id INTEGER,
                item_id INTEGER,
                FOREIGN KEY (category_id) REFERENCES category(id),
                FOREIGN KEY (item_id) REFERENCES items(id),
                UNIQUE(category_id, item_id)
            );"
        .to_string();
        stmts.push(query);
        stmts
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

impl Display for CategoryID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
