use crate::{
    models::recipe::{Recipe, RecipeBody, RecipeID},
    persistence::{
        DBError,
        repositories::{
            IngredientDB, IngredientRepository, RecipeDB, RecipeRepository, Repository,
        },
    },
};

impl<'a> RecipeDB<'a> {
    fn ingredient(&'a self) -> IngredientDB<'a> {
        IngredientDB {
            connection: self.connection,
        }
    }
}

impl<'a> Repository for RecipeDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS recipes(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                )";
        self.connection.execute(query, ())?;
        self.ingredient().create_table()?;
        Ok(())
    }
}
impl<'a> RecipeRepository for RecipeDB<'a> {
    fn insert(&self, body: &RecipeBody) -> Result<RecipeID, DBError> {
        let query = "INSERT INTO recipes(name) VALUES (?1)";
        self.connection.execute(query, (&body.name,))?;
        let recipe_id = RecipeID(self.connection.last_insert_rowid());
        let ingredient_db = self.ingredient();
        for (idx, ingredient) in body.ingredients.iter().enumerate() {
            ingredient_db.insert(&recipe_id, &idx, ingredient)?;
        }
        Ok(recipe_id)
    }

    fn update(&self, item: &Recipe) -> Result<(), DBError> {
        let query = "UPDATE recipes SET name=?2 WHERE id=?1";
        self.connection.execute(query, (item.id, &item.body.name))?;
        let ingredients_db = self.ingredient();
        ingredients_db.delete(&item.id)?;
        for (idx, ingredient) in item.body.ingredients.iter().enumerate() {
            ingredients_db.insert(&item.id, &idx, ingredient)?;
        }
        Ok(())
    }

    fn delete(&self, item: Recipe) -> Result<(), DBError> {
        let query = "DELETE FROM recipes WHERE id=?1";
        self.connection.execute(query, (item.id,))?;
        Ok(())
    }

    fn get_range(&self, offset: usize, limit: usize) -> Result<Vec<Recipe>, DBError> {
        let query = format!("SELECT * FROM recipes LIMIT {limit} OFFSET {offset}");
        let mut stmt = self
            .connection
            .prepare(&query)
            .expect("Query must be valid");
        let rows = stmt
            .query_map([], |row| {
                let id = row.get(0)?;
                let name = row.get(1)?;
                let ingredients = self.ingredient().get(&id)?;
                Ok(Recipe {
                    id,
                    body: RecipeBody { name, ingredients },
                })
            })
            .unwrap();
        Ok(rows
            .into_iter()
            .fold(Vec::with_capacity(limit), |mut acc, row| {
                match row {
                    Ok(recipe) => acc.push(recipe),
                    Err(e) => panic!("Retrieving Recipe failled with error: {e}"),
                }
                acc
            }))
    }
}
