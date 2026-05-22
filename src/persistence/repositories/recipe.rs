use crate::{
    logic::recipe::{Recipe, RecipeBody, RecipeID},
    persistence::{
        DBError,
        repositories::{RecipeDB, RecipeRepository, Repository},
    },
};

impl<'a> Repository for RecipeDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS recipes(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                )";
        self.connection.execute(query, ())?;
        Ok(())
    }
}
impl<'a> RecipeRepository for RecipeDB<'a> {
    fn insert(&self, body: &RecipeBody) -> Result<RecipeID, DBError> {
        let query = "INSERT INTO recipes(name) VALUES (?1)";
        self.connection.execute(query, (&body.name,))?;
        let recipe_id = RecipeID(self.connection.last_insert_rowid());
        for (idx, ingredient) in body.ingredients.iter().enumerate() {
            todo!()
        }
        Ok(recipe_id)
    }

    fn update(&self, item: &Recipe) -> Result<(), DBError> {
        todo!()
    }

    fn delete(&self, item: Recipe) -> Result<(), DBError> {
        todo!()
    }

    fn get_range(&self, offset: usize, limit: usize) -> Result<Vec<Recipe>, DBError> {
        todo!()
    }
}
