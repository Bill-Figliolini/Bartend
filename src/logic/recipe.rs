use rusqlite::{ToSql, types::FromSql};

use crate::{
    logic::{category::CategoryID, quantity::Quantity},
    persistence::Database,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RecipeID(pub i64);

#[derive(Debug, Clone)]
pub struct Recipe {
    id: RecipeID,
    body: RecipeBody,
}
#[derive(Debug, Clone)]
pub struct RecipeBody {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
}
#[derive(Debug, Clone)]
pub struct Ingredient {
    category: CategoryID,
    quantity: Quantity,
}

impl Recipe {
    pub fn get_range(db: &Database, offset: usize, quantity: usize) -> Vec<Recipe> {
        let query = format!("SELECT * FROM recipes LIMIT {quantity} OFFSET {offset}");
        let mut stmt = db.connection.prepare(&query).expect("Query must be valid");
        let rows = stmt
            .query_map([], |row| {
                let id = row.get(0).unwrap();
                let name = row.get(1).unwrap();
                let ingredients = Ingredient::get_for_recipe(db, &id);
                Ok(Recipe {
                    id,
                    body: RecipeBody { name, ingredients },
                })
            })
            .unwrap();
        rows.into_iter()
            .fold(Vec::with_capacity(quantity), |mut acc, row| {
                match row {
                    Ok(recipe) => acc.push(recipe),
                    Err(e) => panic!("Retrieving Recipe failled with error: {e}"),
                }
                acc
            })
    }
}
impl Ingredient {
    pub fn create() -> String {
        "CREATE TABLE IF NOT EXISTS recipe_ingredients(
            recipe_id INTEGER,
            index INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            quantity REAL NOT NULL,
            unit INTEGER NOT NULL,
            FOREIGN KEY (recipe_id) REFERENCES recipe(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE RESTRICT,
            UNIQUE(recipe_id, index)
        )"
        .to_string()
    }
    pub fn insert(self, db: &Database, recipe_id: &RecipeID, idx: usize) {
        let query = "INSERT INTO recipe_ingredients(recipe_id, index, category_id, quantity, unit) VALUES (?1, ?2, ?3, ?4)";
        let (quantity, unit) = self.quantity.db_format();

        if let Err(e) = db.connection.execute(
            query,
            (recipe_id, idx as i64, self.category, quantity, unit),
        ) {
            panic!("Ingredient insertion failure with {e}");
        }
    }
    pub fn get_for_recipe(db: &Database, recipe_id: &RecipeID) -> Vec<Ingredient> {
        let mut stmt = db
            .connection
            .prepare(
                "SELECT category_id, quantity, unit FROM ingredients WHERE recipe_id = ?1 ORDER BY idx;",
            )
            .unwrap();
        let rows = stmt
            .query_map((*recipe_id,), |row| {
                let category = row.get(0).unwrap();
                let quantity = Quantity::from_db(row.get(2).unwrap(), row.get(1).unwrap());
                Ok(Ingredient { category, quantity })
            })
            .unwrap();
        rows.into_iter().fold(Vec::new(), |mut acc, ingredient| {
            acc.push(ingredient.unwrap());
            acc
        })
    }
}

impl ToSql for RecipeID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl FromSql for RecipeID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_i64()?;
        Ok(Self(value))
    }
}
