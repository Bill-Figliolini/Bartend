use std::collections::HashMap;

use crate::{
    logic::LogicError,
    models::{BartendError, Recipe, RecipeBody, RecipeID},
    persistence::repositories::RecipeRepository,
};

#[derive(Debug)]
pub struct RecipeService {
    recipes: HashMap<RecipeID, RecipeBody>,
}

impl RecipeService {
    pub fn new(db: &impl RecipeRepository) -> Result<Self, BartendError> {
        let recipes = db.get_all()?;
        Ok(RecipeService { recipes })
    }

    pub fn get_page(&self, page: usize, page_size: usize) -> Vec<RecipeID> {
        let page_offset = page * page_size;
        self.get_sorted()
            .into_iter()
            .skip(page_offset)
            .take(page_size)
            .collect()
    }

    fn get_sorted(&self) -> Vec<RecipeID> {
        let mut entries: Vec<(String, RecipeID)> = self
            .recipes
            .iter()
            .map(|(id, body)| (body.name.to_lowercase(), *id))
            .collect();
        entries.sort_unstable_by(|(a_name, a_id), (b_name, b_id)| {
            a_name.cmp(b_name).then_with(|| a_id.0.cmp(&b_id.0))
        });
        entries.into_iter().map(|(_, id)| id).collect()
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
    pub fn get(&self, id: RecipeID) -> Result<RecipeBody, BartendError> {
        match self.recipes.get(&id) {
            Some(body) => Ok(body.clone()),
            None => Err(LogicError::InvalidRecipe(id))?,
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
        }
        Ok(())
    }
    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::{
        models::{CategoryBody, Ingredient, Quantity, Unit},
        persistence::{DBError, Database, repositories::CategoryRepository},
    };

    fn test_db() -> Database {
        Database::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn draft(name: &str) -> RecipeBody {
        RecipeBody {
            name: name.to_string(),
            ingredients: Vec::new(),
        }
    }

    fn service_with(db: &Database, names: &[&str]) -> (RecipeService, Vec<RecipeID>) {
        let mut service = RecipeService::new(&db.recipe_db()).unwrap();
        let ids = names
            .iter()
            .map(|name| service.add(&db.recipe_db(), draft(name)).unwrap())
            .collect();
        (service, ids)
    }

    #[test]
    fn new_loads_existing_recipes() {
        let db = test_db();
        let (_, ids) = service_with(&db, &["Negroni", "Martini"]);

        let reloaded = RecipeService::new(&db.recipe_db()).unwrap();

        assert_eq!(reloaded.recipe_count(), 2);
        assert_eq!(reloaded.get(ids[0]).unwrap(), draft("Negroni"));
    }

    #[test]
    fn pages_are_sorted_by_name_ignoring_case() {
        let db = test_db();
        let (service, ids) = service_with(&db, &["negroni", "Martini", "Daiquiri"]);

        assert_eq!(service.get_page(0, 2), vec![ids[2], ids[1]]);
        assert_eq!(service.get_page(1, 2), vec![ids[0]]);
        assert_eq!(service.get_page(2, 2), Vec::new());
    }

    #[test]
    fn equal_names_are_ordered_by_id() {
        let db = test_db();
        let (service, ids) = service_with(&db, &["Sour", "sour"]);
        assert_eq!(service.get_page(0, 10), ids);
    }

    #[test]
    fn get_all_returns_every_recipe() {
        let db = test_db();
        let (service, ids) = service_with(&db, &["Negroni", "Martini"]);

        let mut all: Vec<RecipeID> = service.get_all().iter().map(|r| r.id).collect();
        all.sort_by_key(|id| id.0);

        assert_eq!(all, ids);
    }

    #[test]
    fn getting_an_unknown_recipe_errors() {
        let db = test_db();
        let (service, _) = service_with(&db, &[]);
        assert!(matches!(
            service.get(RecipeID(5)),
            Err(BartendError::Logic(LogicError::InvalidRecipe(RecipeID(5))))
        ));
    }

    #[test]
    fn add_stores_ingredients() {
        let db = test_db();
        let gin = db
            .category_db()
            .insert(&CategoryBody {
                name: "Gin".to_string(),
            })
            .unwrap();
        let body = RecipeBody {
            name: "Gin Shot".to_string(),
            ingredients: vec![Ingredient {
                category: gin,
                quantity: Quantity::new(50.0, Unit::Milliliter),
            }],
        };
        let mut service = RecipeService::new(&db.recipe_db()).unwrap();

        let id = service.add(&db.recipe_db(), body.clone()).unwrap();

        assert_eq!(service.get(id).unwrap(), body);
        assert_eq!(db.recipe_db().get_all().unwrap()[&id], body);
    }

    #[test]
    fn a_rejected_add_is_not_cached() {
        let db = test_db();
        let mut service = RecipeService::new(&db.recipe_db()).unwrap();
        let body = RecipeBody {
            name: "Ghost Shot".to_string(),
            ingredients: vec![Ingredient {
                category: crate::models::CategoryID(99),
                quantity: Quantity::new(50.0, Unit::Milliliter),
            }],
        };

        let result = service.add(&db.recipe_db(), body);

        assert!(matches!(
            result,
            Err(BartendError::Db(DBError::RestrictViolation))
        ));
        assert_eq!(service.recipe_count(), 0);
    }

    #[test]
    fn update_replaces_the_cached_and_stored_recipe() {
        let db = test_db();
        let (mut service, ids) = service_with(&db, &["Negroni"]);
        let recipe = Recipe {
            id: ids[0],
            body: draft("Boulevardier"),
        };

        service.update(&db.recipe_db(), recipe).unwrap();

        assert_eq!(service.get(ids[0]).unwrap(), draft("Boulevardier"));
        assert_eq!(
            db.recipe_db().get_all().unwrap()[&ids[0]],
            draft("Boulevardier")
        );
    }

    #[test]
    fn updating_an_unknown_recipe_errors() {
        let db = test_db();
        let (mut service, _) = service_with(&db, &[]);
        let ghost = Recipe {
            id: RecipeID(5),
            body: draft("Ghost"),
        };
        assert!(matches!(
            service.update(&db.recipe_db(), ghost),
            Err(BartendError::Logic(LogicError::InvalidRecipe(RecipeID(5))))
        ));
    }

    #[test]
    fn delete_removes_the_recipe() {
        let db = test_db();
        let (mut service, ids) = service_with(&db, &["Negroni", "Martini"]);

        service.delete(&db.recipe_db(), ids[0]).unwrap();

        assert_eq!(service.recipe_count(), 1);
        assert!(matches!(
            service.get(ids[0]),
            Err(BartendError::Logic(LogicError::InvalidRecipe(_)))
        ));
        assert!(!db.recipe_db().get_all().unwrap().contains_key(&ids[0]));
    }
}
