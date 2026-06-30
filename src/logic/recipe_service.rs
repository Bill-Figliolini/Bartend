use std::collections::HashMap;

use crate::models::{RecipeBody, RecipeID};

#[derive(Debug)]
pub(super) struct RecipeService {
    recipes: HashMap<RecipeID, RecipeBody>,
}
