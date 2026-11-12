use std::{error::Error, fmt::Display};

use crate::{logic::LogicError, models::ConfigError, persistence::DBError};
#[derive(Debug, Clone)]
pub enum BartendError {
    Logic(LogicError),
    Db(DBError),
    Config(ConfigError),
}

impl From<DBError> for BartendError {
    fn from(value: DBError) -> Self {
        BartendError::Db(value)
    }
}
impl From<LogicError> for BartendError {
    fn from(value: LogicError) -> Self {
        BartendError::Logic(value)
    }
}
impl Display for BartendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BartendError::Logic(logic_error) => write!(f, "{logic_error}"),
            BartendError::Db(dberror) => write!(f, "{dberror}"),
            BartendError::Config(config) => write!(f, "{config}"),
        }
    }
}
impl Display for LogicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LOGIC ERROR: ")?;
        match self {
            LogicError::InvalidCategoryRelation { parent, child } => {
                write!(f, "Category Edge {parent} to {child} would create loop!")
            }
            LogicError::InvalidCategory(category_id) => write!(f, "invalid Category {category_id}"),
            LogicError::InvalidItem(item_id) => write!(f, "invalid Item {item_id}"),
            LogicError::InvalidRecipe(recipe_id) => write!(f, "invalid Recipe {recipe_id}"),
            LogicError::CategoryNotInGraph(category_id) => {
                write!(f, "Category not in graph {category_id}")
            }
        }
    }
}
impl Error for BartendError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CategoryID, ItemID, RecipeID};

    #[test]
    fn logic_errors_describe_the_problem() {
        let cases = [
            (
                LogicError::InvalidCategoryRelation {
                    parent: CategoryID(1),
                    child: CategoryID(2),
                },
                "LOGIC ERROR: Category Edge 1 to 2 would create loop!",
            ),
            (
                LogicError::InvalidCategory(CategoryID(3)),
                "LOGIC ERROR: invalid Category 3",
            ),
            (
                LogicError::InvalidItem(ItemID(4)),
                "LOGIC ERROR: invalid Item 4",
            ),
            (
                LogicError::InvalidRecipe(RecipeID(5)),
                "LOGIC ERROR: invalid Recipe 5",
            ),
            (
                LogicError::CategoryNotInGraph(CategoryID(6)),
                "LOGIC ERROR: Category not in graph 6",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(BartendError::from(error).to_string(), expected);
        }
    }

    #[test]
    fn wrapped_errors_display_their_source() {
        let db = BartendError::from(DBError::InvalidUnit);
        assert_eq!(db.to_string(), DBError::InvalidUnit.to_string());

        let config = BartendError::Config(ConfigError::ReadError);
        assert_eq!(config.to_string(), ConfigError::ReadError.to_string());
    }
}
