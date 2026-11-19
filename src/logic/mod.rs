mod category_service;
mod graph;
mod item_service;
mod recipe_service;
mod stock;
mod usage;

use crate::models::{CategoryID, ItemID, Quantity, RecipeID};

pub use self::{
    category_service::{CategoryService, ItemCounts},
    graph::GraphPatch,
    item_service::ItemService,
    recipe_service::RecipeService,
    stock::{IngredientStatus, StockLevel, can_make, ingredient_statuses, totals_by_key},
    usage::{RecipeUse, recipes_for_category, recipes_using},
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
    /// An item's full size is not a positive amount of the same kind of
    /// quantity as its stock.
    InvalidFullSize,
}

/// The ids of `entries`, ordered by name (case-insensitively), then by id so
/// that same-named entries keep a stable order.
fn ids_by_name<'a, Id: Ord>(entries: impl IntoIterator<Item = (&'a str, Id)>) -> Vec<Id> {
    let mut entries: Vec<(String, Id)> = entries
        .into_iter()
        .map(|(name, id)| (name.to_lowercase(), id))
        .collect();
    entries.sort_unstable();
    entries.into_iter().map(|(_, id)| id).collect()
}
