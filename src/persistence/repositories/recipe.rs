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
