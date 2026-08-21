use std::collections::HashMap;

use crate::{
    logic::LogicError,
    models::{BartendError, Recipe, RecipeBody, RecipeID},
    persistence::repositories::RecipeRepository,
};

#[derive(Debug)]
pub struct RecipeService {
    recipes: HashMap<RecipeID, RecipeBody>,
    page_size: usize,
}

impl RecipeService {
    pub fn new(db: &impl RecipeRepository) -> Result<Self, BartendError> {
        let recipes = db.get_all()?;
        let page_size = 15;
        Ok(RecipeService { recipes, page_size })
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
    pub fn get(&self, id: &RecipeID) -> Result<RecipeBody, BartendError> {
        match self.recipes.get(id) {
            Some(body) => Ok(body.clone()),
            None => Err(LogicError::InvalidRecipe(*id))?,
        }
    }
    pub fn add(
        &mut self,
        db: &impl RecipeRepository,
        body: RecipeBody,
    ) -> Result<RecipeID, BartendError> {
        let id = db.insert(&body)?;
        self.recipes.insert(id, body);
        Ok(id)
    }
    pub fn delete(
        &mut self,
        db: &impl RecipeRepository,
        recipe: RecipeID,
    ) -> Result<(), BartendError> {
        db.delete(recipe)?;
        self.recipes.remove(&recipe);
        Ok(())
    }
    pub fn update(
        &mut self,
        db: &impl RecipeRepository,
        recipe: Recipe,
    ) -> Result<(), BartendError> {
        db.update(&recipe)?;
        match self.recipes.get_mut(&recipe.id) {
            Some(slot) => *slot = recipe.body,
            None => Err(LogicError::InvalidRecipe(recipe.id))?,
        };
        Ok(())
    }
}
