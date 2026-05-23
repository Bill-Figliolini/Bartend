use crate::{
    logic::{category::CategoryID, quantity::Quantity},
    persistence::Database,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RecipeID(pub i64);

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: RecipeID,
    pub body: RecipeBody,
}
#[derive(Debug, Clone)]
pub struct RecipeBody {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
}
#[derive(Debug, Clone)]
pub struct Ingredient {
    pub category: CategoryID,
    pub quantity: Quantity,
}

impl Recipe {}
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
