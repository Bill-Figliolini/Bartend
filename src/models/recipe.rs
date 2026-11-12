use std::fmt::Display;

use crate::models::{category::CategoryID, quantity::Quantity};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RecipeID(pub i64);

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: RecipeID,
    pub body: RecipeBody,
}
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeBody {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    pub category: CategoryID,
    pub quantity: Quantity,
}

impl Recipe {}
impl Ingredient {}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body.name)
    }
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for RecipeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for RecipeBody {
    fn default() -> Self {
        Self {
            name: "Invalid Recipe".to_string(),
            ingredients: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipes_display_their_name_and_compare_by_id() {
        let negroni = Recipe {
            id: RecipeID(1),
            body: RecipeBody {
                name: "Negroni".to_string(),
                ingredients: Vec::new(),
            },
        };
        let placeholder = Recipe {
            id: RecipeID(1),
            body: RecipeBody::default(),
        };
        assert_eq!(negroni.to_string(), "Negroni");
        assert_eq!(negroni.id.to_string(), "1");
        assert_eq!(negroni, placeholder);
        assert_eq!(placeholder.body.name, "Invalid Recipe");
    }
}
