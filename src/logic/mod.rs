mod category_service;
mod graph;
mod item_service;
mod recipe_service;
mod stock;

use crate::models::{CategoryID, ItemID, Quantity, RecipeID};

pub use self::{
    category_service::CategoryService,
    graph::GraphPatch,
    item_service::ItemService,
    recipe_service::RecipeService,
    stock::{can_make, totals_by_key},
};
#[derive(Debug, Clone)]
pub enum LogicError {
    InvalidCategoryRelation {
        parent: CategoryID,
        child: CategoryID,
    },
    InvalidCategory(CategoryID),
    InvalidItem(ItemID),
    InvalidRecipe(RecipeID),
    CategoryNotInGraph(CategoryID),
    /// Using the item would take more than it has in stock.
    InsufficientStock {
        item: ItemID,
        needed: Quantity,
        available: Quantity,
    },
    /// The item was used in a different kind of quantity (e.g. by mass)
    /// than it is stocked in (e.g. by volume).
    IncompatibleUnits(ItemID),
}
