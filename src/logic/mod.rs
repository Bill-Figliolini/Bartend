mod category_service;
mod graph;
mod item_service;
mod recipe_service;

use std::path::Path;

use crate::{
    models::{CategoryID, ItemID, RecipeID},
    persistence::Database,
};

pub use self::{
    category_service::CategoryService, graph::GraphPatch, item_service::ItemService,
    recipe_service::RecipeService,
};
#[derive(Debug)]
pub enum LogicError {
    InvalidCategoryRelation {
        parent: CategoryID,
        child: CategoryID,
    },
    InvalidCategory(CategoryID),
    InvalidItem(ItemID),
    InvalidRecipe(RecipeID),
    CategoryNotInGraph(CategoryID),
}
