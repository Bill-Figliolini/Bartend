use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::common::category::graph::{DirectedAcyclicGraph, GraphError};

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
        todo!()
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
    pub fn remove_category(&mut self, id: CategoryID) {
        self.names.remove(&id);
        self.relations.remove(id);
        todo!("Add hook into Persistance here")
    }
    pub fn add_category(&mut self, name: String) {
        let id = todo!("Add hook into Persistance here");
        self.names.insert(id, name);
        self.relations.insert_vertex(id);
    }
    pub fn add_relation(
        &mut self,
        parent: &CategoryID,
        child: &CategoryID,
    ) -> Result<(), GraphError> {
        match self.relations.insert_edge((parent, child)) {
            Ok(()) => {
                todo!("Hook into persistance here");
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
