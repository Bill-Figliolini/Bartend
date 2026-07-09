use std::collections::HashMap;

use crate::{
    models::{Recipe, RecipeBody, RecipeID},
    persistence::repositories::RecipeRepository,
};

#[derive(Debug)]
pub(super) struct RecipeService {
    recipes: HashMap<RecipeID, RecipeBody>,
}

impl RecipeService {
    pub fn new(db: &impl RecipeRepository) -> Self {
        let recipes = HashMap::new();
        RecipeService { recipes }
    }
    pub fn get(&self, id: &RecipeID) -> RecipeBody {
        match self.recipes.get(id) {
            Some(body) => body.clone(),
            None => panic!("Invalid Recipe ID"),
        }
    }
    pub fn add(&mut self, db: &impl RecipeRepository, body: RecipeBody) -> RecipeID {
        let id = match db.insert(&body) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        };
        self.recipes.insert(id, body);
        id
    }
    pub fn delete(&mut self, db: &impl RecipeRepository, recipe: RecipeID) {
        if let Err(e) = db.delete(recipe) {
            panic!("{e}");
        }
        self.recipes.remove(&recipe);
    }
    pub fn update_recipe(&mut self, db: &impl RecipeRepository, recipe: Recipe) {
        if let Err(e) = db.update(&recipe) {
            panic!("{e}");
        }
        match self.recipes.get_mut(&recipe.id) {
            Some(slot) => *slot = recipe.body,
            None => panic!("Invalid Recipe ID in circulaton"),
        };
    }
}
