use crate::{
    logic::{
        quantity::Quantity,
        recipe::{Ingredient, RecipeID},
    },
    persistence::{
        DBError,
        repositories::{IngredientDB, IngredientRepository, Repository},
    },
};

impl<'a> Repository for IngredientDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS ingredients(
            recipe_id INTEGER,
            index INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            quantity REAL NOT NULL,
            unit INTEGER NOT NULL,
            FOREIGN KEY (recipe_id) REFERENCES recipe(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE RESTRICT,
            UNIQUE(recipe_id, index)
        )";
        self.connection.execute(query, ())?;
        Ok(())
    }
}
impl<'a> IngredientRepository for IngredientDB<'a> {
    fn insert(
        &self,
        recipe: &RecipeID,
        index: &usize,
        ingredient: &Ingredient,
    ) -> Result<(), DBError> {
        let query = "INSERT INTO ingredients(recipe_id, index, category_id, quantity, unit) VALUES (?1, ?2, ?3, ?4)";
        let (quantity, unit) = ingredient.quantity.db_format();

        self.connection.execute(
            query,
            (recipe, *index as i64, ingredient.category, quantity, unit),
        )?;
        Ok(())
    }

    fn delete(&self, recipe: &RecipeID) -> Result<(), DBError> {
        let query = "DELETE FROM ingredients WHERE recipe_id = ?1";
        self.connection.execute(query, (recipe,))?;
        Ok(())
    }

    fn get(&self, recipe: &RecipeID) -> Result<Vec<Ingredient>, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare(
                "SELECT category_id, quantity, unit FROM ingredients WHERE recipe_id = ?1 ORDER BY idx;",
            )
            .unwrap();
        let rows = stmt
            .query_map((*recipe,), |row| {
                let category = row.get(0)?;
                let quantity = Quantity::from_db(row.get(2)?, row.get(1)?);
                Ok(Ingredient { category, quantity })
            })
            .unwrap();
        Ok(rows.into_iter().fold(Vec::new(), |mut acc, ingredient| {
            acc.push(ingredient.unwrap());
            acc
        }))
    }
}
