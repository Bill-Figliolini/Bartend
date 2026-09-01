use rusqlite::Connection;

use crate::{
    models::{Ingredient, Quantity, RecipeID},
    persistence::DBError,
};

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

//Needs to return ruslite::Error to meet the implementation of query_map
pub(super) fn get(
    connection: &Connection,
    recipe: &RecipeID,
) -> Result<Vec<Ingredient>, rusqlite::Error> {
    let mut stmt = connection
            .prepare(
                "SELECT category_id, quantity, unit FROM ingredients WHERE recipe_id = ?1 ORDER BY ingredient_index;",
            )?;
    let rows = stmt.query_map((*recipe,), |row| {
        let category = row.get(0)?;
        let quantity = Quantity::from_db(row.get(2)?, row.get(1)?);
        Ok(Ingredient { category, quantity })
    })?;
    let rows = rows
        .into_iter()
        .collect::<Result<Vec<Ingredient>, rusqlite::Error>>()?;
    Ok(rows)
}
