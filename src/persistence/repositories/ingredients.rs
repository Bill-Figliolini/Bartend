use rusqlite::Connection;

use crate::{
    models::{Ingredient, Quantity, RecipeID},
    persistence::DBError,
};

pub(super) fn schema() -> &'static str {
    "CREATE TABLE IF NOT EXISTS ingredients(
                recipe_id INTEGER,
                ingredient_index INTEGER,
                category_id INTEGER,
                quantity REAL NOT NULL,
                unit INTEGER NOT NULL,
                FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
                FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE RESTRICT,
                UNIQUE(recipe_id, ingredient_index)
            )"
}

pub(super) fn insert(
    transaction: &Connection,
    recipe: &RecipeID,
    index: &usize,
    ingredient: &Ingredient,
) -> Result<(), DBError> {
    let query = "INSERT INTO ingredients(recipe_id, ingredient_index, category_id, quantity, unit) VALUES (?1, ?2, ?3, ?4, ?5)";
    let (quantity, unit) = ingredient.quantity.db_format();
    transaction.execute(
        query,
        (recipe, *index as i64, ingredient.category, quantity, unit),
    )?;
    Ok(())
}

pub(super) fn delete(connection: &Connection, recipe: &RecipeID) -> Result<(), DBError> {
    let query = "DELETE FROM ingredients WHERE recipe_id = ?1";
    connection.execute(query, (recipe,))?;
    Ok(())
}

pub(super) fn get(
    connection: &Connection,
    recipe: &RecipeID,
) -> Result<Vec<Ingredient>, rusqlite::Error> {
    let mut stmt = connection
            .prepare(
                "SELECT category_id, quantity, unit FROM ingredients WHERE recipe_id = ?1 ORDER BY ingredient_index;",
            )?;
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
