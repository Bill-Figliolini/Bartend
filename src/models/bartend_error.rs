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
