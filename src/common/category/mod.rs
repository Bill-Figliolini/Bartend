use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::{
    common::category::graph::{DirectedAcyclicGraph, GraphError},
    persistence::{DB, DBStore},
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

impl DBStore for Category {
    fn create(db: &DB) {
        todo!()
    }

    fn read(db: &DB) -> Self {
        todo!()
    }

    fn input(db: &DB, input: impl IntoIterator) -> Self {
        todo!()
    }

    fn update(&self, db: &DB) {
        todo!()
    }

    fn delete(self, db: &DB) {
        todo!()
    }
}

impl DBStore for CategoryManager {
    fn create(db: &DB) {
        todo!()
    }

    fn read(db: &DB) -> Self {
        todo!()
    }

    fn input(db: &DB, input: impl IntoIterator) -> Self {
        todo!()
    }

    fn update(&self, db: &DB) {
        todo!()
    }

    fn delete(self, db: &DB) {
        todo!()
    }
}
