use crate::logic::{category::CategoryID, quantity::Quantity};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RecipeID(pub i64);

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: RecipeID,
    pub body: RecipeBody,
}
#[derive(Debug, Clone)]
pub struct RecipeBody {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
}
#[derive(Debug, Clone)]
pub struct Ingredient {
    pub category: CategoryID,
    pub quantity: Quantity,
}

impl Recipe {}
impl Ingredient {}
