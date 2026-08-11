use std::collections::HashMap;

use crate::{
    models::{Recipe, RecipeBody, RecipeID},
    persistence::repositories::RecipeRepository,
};

#[derive(Debug)]
pub struct RecipeService {
    recipes: HashMap<RecipeID, RecipeBody>,
    page_size: usize,
}

impl RecipeService {
    pub fn new(db: &impl RecipeRepository) -> Self {
        let recipes = match db.get_all() {
            Ok(recipes) => recipes,
            Err(e) => panic!("{e}"),
        };
        let page_size = 15;
        RecipeService { recipes, page_size }
    }

    pub fn get_page(&self, page: usize) -> Vec<RecipeID> {
        let page_offset = page * self.page_size;
        self.recipes
            .keys()
            .copied()
            .skip(page_offset)
            .take(self.page_size)
            .collect()
    }
    pub fn get_all(&self) -> Vec<Recipe> {
        self.recipes.iter().fold(
            Vec::with_capacity(self.recipes.len()),
            |mut acc, (id, body)| {
                acc.push(Recipe {
                    id: *id,
                    body: body.clone(),
                });
                acc
            },
        )
    }
    pub fn get(&self, id: &RecipeID) -> RecipeBody {
        match self.recipes.get(id) {
            Some(body) => body.clone(),
            None => panic!("Invalid Recipe ID"),
        }
    }
    pub fn add(&mut self, db: &mut impl RecipeRepository, body: RecipeBody) -> RecipeID {
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
    pub fn update(&mut self, db: &mut impl RecipeRepository, recipe: Recipe) {
        if let Err(e) = db.update(&recipe) {
            panic!("{e}");
        }
        match self.recipes.get_mut(&recipe.id) {
            Some(slot) => *slot = recipe.body,
            None => panic!("Invalid Recipe ID in circulaton"),
        };
    }
}
