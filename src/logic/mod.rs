mod category_service;
mod graph;
mod item_service;
mod recipe_service;

use std::path::Path;

use crate::{models::CategoryID, persistence::Database};

pub use self::{
    category_service::CategoryService, graph::GraphPatch, item_service::ItemService,
    recipe_service::RecipeService,
};

///Boundary with presentation module.
///Must be able to:
///     Retrive Items, preferably in a collection
///     Accept new Items
#[derive(Debug)]
pub struct BarCollection {
    pub db: Database,
}

impl BarCollection {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let db = match Database::load(path) {
            Ok(db) => db,
            Err(e) => panic!("DB Creation Error: {e}"),
        };
        Self { db }
    }
}
#[derive(Debug)]
pub enum LogicError {
    InvalidCategoryRelation {
        parent: CategoryID,
        child: CategoryID,
    },
    InvalidCategory(CategoryID),
    CategoryNotInGraph(CategoryID),
}
