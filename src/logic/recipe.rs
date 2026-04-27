use rusqlite::{ToSql, types::FromSql};

use crate::logic::{category::CategoryID, quantity::Quantity};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RecipeID(i64);

#[derive(Debug)]
struct Recipe {
    id: RecipeID,
    name: String,
    ingredients: Vec<(CategoryID, Quantity)>,
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
