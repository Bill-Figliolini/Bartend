use rusqlite::Connection;
use std::collections::HashMap;

use crate::{
    models::{Recipe, RecipeBody, RecipeID},
    persistence::{
        DBError,
        repositories::{RecipeDB, RecipeRepository, ingredients},
    },
};

impl<'a> RecipeDB<'a> {
    fn from_db(connection: &Connection, row: &rusqlite::Row) -> Result<Recipe, rusqlite::Error> {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let ingredients = ingredients::get(connection, &id)?;
        Ok(Recipe {
            id,
            body: RecipeBody { name, ingredients },
        })
    }
}

impl<'a> RecipeDB<'a> {
    pub(in crate::persistence) fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS recipes(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                )";
        self.connection.execute(query, ())?;
        let query = ingredients::schema();
        self.connection.execute(query, ())?;
        Ok(())
    }
}

impl<'a> RecipeRepository for RecipeDB<'a> {
    fn insert(&self, body: &RecipeBody) -> Result<RecipeID, DBError> {
        let query = "INSERT INTO recipes(name) VALUES (?1)";
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(query, (&body.name,))?;

        let recipe = RecipeID(transaction.last_insert_rowid());

        for (index, ingredient) in body.ingredients.iter().enumerate() {
            ingredients::insert(&transaction, &recipe, &index, ingredient)?;
        }
        transaction.commit()?;
        Ok(recipe)
    }

    fn update(&self, recipe: &Recipe) -> Result<(), DBError> {
        let query = "UPDATE recipes SET name=?2 WHERE id=?1";
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(query, (recipe.id, &recipe.body.name))?;

        ingredients::delete(&transaction, &recipe.id)?;

        for (index, ingredient) in recipe.body.ingredients.iter().enumerate() {
            ingredients::insert(&transaction, &recipe.id, &index, ingredient)?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn delete(&self, item: RecipeID) -> Result<(), DBError> {
        let query = "DELETE FROM recipes WHERE id=?1";
        self.connection.execute(query, (item,))?;
        Ok(())
    }

    fn get_all(&self) -> Result<HashMap<RecipeID, RecipeBody>, DBError> {
        let query = "SELECT * FROM recipes";
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map([], |row| RecipeDB::from_db(self.connection, row))?;
        let rows: Vec<Recipe> = rows
            .into_iter()
            .collect::<Result<Vec<Recipe>, rusqlite::Error>>()?;
        let recipes = rows.into_iter().map(|row| (row.id, row.body)).collect();
        Ok(recipes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random_range;
    use rusqlite::Connection;

    use crate::{
        models::{CategoryBody, CategoryID, CountName::Dash, Ingredient, Quantity},
        persistence::{Database, repositories::CategoryRepository},
    };

    fn db_init() -> Database {
        Database::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn create_categories(quantity: usize) -> Vec<CategoryBody> {
        (0..quantity)
            .map(|i| CategoryBody {
                name: format!("Category {i}"),
            })
            .collect()
    }

    fn create_recipe(categories: &Vec<CategoryID>, name: String) -> RecipeBody {
        let ingredients = categories
            .iter()
            .map(|id| {
                let quantity = match rand::random_range(0..3) {
                    0 => Quantity::Count {
                        quantity: random_range(0.0..=750.0),
                        name: Dash,
                    },
                    1 => Quantity::Mass {
                        quantity: random_range(0.0..=750.0),
                    },
                    2 => Quantity::Volume {
                        quantity: random_range(0.0..=750.0),
                    },
                    _ => unreachable!(),
                };
                Ingredient {
                    category: *id,
                    quantity,
                }
            })
            .collect();
        RecipeBody { name, ingredients }
    }

    fn verify_recipe(in_db: &Recipe, input: &RecipeBody) {
        assert_eq!(in_db.body.name, input.name);
        for (ingredient_in_db, ingredient_input) in
            in_db.body.ingredients.iter().zip(input.ingredients.iter())
        {
            assert_eq!(ingredient_in_db, ingredient_input)
        }
    }

    #[test]
    fn table_created_successfully() {
        let database = Database {
            connection: Connection::open_in_memory().unwrap(),
        };
        let table_names = vec!["recipes", "ingredients"];

        let result = database.recipe_db().create_table();

        assert!(result.is_ok());
        for table in table_names {
            assert!(database.connection.table_exists(None, table).unwrap())
        }
    }
    mod insert {

        use super::*;

        #[test]
        fn inserts_row_into_db() {
            let db = db_init();
            let category_bodies = create_categories(5);
            let category_ids = category_bodies
                .iter()
                .map(|body| db.category_db().insert(body).unwrap())
                .collect();
            let recipe = create_recipe(&category_ids, "Recipe".to_string());

            let result = db.recipe_db().insert(&recipe);

            assert!(result.is_ok());
            let recipe_id = result.unwrap();
            let in_db = db
                .connection
                .query_one("SELECT * FROM recipes WHERE id=?1", (recipe_id,), |row| {
                    RecipeDB::from_db(&db.connection, row)
                })
                .unwrap();
            verify_recipe(&in_db, &recipe);
        }
    }
    mod delete {
        use rusqlite::OptionalExtension;

        use super::*;
        #[test]
        fn removes_from_db() {
            let db = db_init();
            let category_bodies = create_categories(5);
            let category_ids = category_bodies
                .iter()
                .map(|body| db.category_db().insert(body).unwrap())
                .collect();
            let recipe = create_recipe(&category_ids, "Recipe".to_string());
            let id = db.recipe_db().insert(&recipe).unwrap();

            let result = db.recipe_db().delete(id);

            assert!(result.is_ok());
            let in_db = db
                .connection
                .query_one("SELECT * FROM recipes WHERE id=?1", (id,), |row| {
                    RecipeDB::from_db(&db.connection, row)
                })
                .optional()
                .unwrap();

            assert!(in_db.is_none());
        }
    }
    mod update {
        use super::*;

        #[test]
        fn updates_in_db() {
            let db = db_init();
            let category_bodies = create_categories(5);
            let category_ids = category_bodies
                .iter()
                .map(|body| db.category_db().insert(body).unwrap())
                .collect();
            let recipe = create_recipe(&category_ids, "Recipe".to_string());
            let updated_bodies = create_categories(10);
            let updated_category_ids = updated_bodies
                .iter()
                .map(|body| db.category_db().insert(body).unwrap())
                .collect();
            let updated_recipe = create_recipe(&updated_category_ids, "THE BIG ONE".to_string());
            let id = db.recipe_db().insert(&recipe).unwrap();

            let result = db.recipe_db().update(&Recipe {
                id,
                body: updated_recipe.clone(),
            });

            assert!(result.is_ok());
            let in_db = db
                .connection
                .query_one("SELECT * FROM recipes WHERE id=?1", (id,), |row| {
                    RecipeDB::from_db(&db.connection, row)
                })
                .unwrap();
            verify_recipe(&in_db, &updated_recipe);
        }
    }
    mod get_all {

        use super::*;
        #[test]
        fn gets_all_in_db() {
            let db = db_init();
            let num_recipes = 100;
            let mut recipes = Vec::with_capacity(num_recipes);
            for i in 0..=num_recipes {
                let ingredient_count: usize = rand::random_range(0..=100);
                let category_bodies = create_categories(ingredient_count);
                let category_ids = category_bodies
                    .iter()
                    .map(|body| db.category_db().insert(body).unwrap())
                    .collect();
                let recipe = create_recipe(&category_ids, format!("Recipe {i}"));
                let id = db.recipe_db().insert(&recipe).unwrap();
                recipes.push(Recipe { id, body: recipe });
            }

            let result = db.recipe_db().get_all();
            let mut in_db = result.unwrap();

            for recipe in recipes {
                assert!(in_db.contains_key(&recipe.id));
                assert_eq!(in_db.get(&recipe.id).unwrap(), &recipe.body);
                in_db.remove(&recipe.id);
            }
            assert!(in_db.is_empty())
        }
    }
    mod foreign_key_constraints {
        use super::*;

        #[test]
        fn ingredient_using_category_blocks_deletion() {}
    }
}
