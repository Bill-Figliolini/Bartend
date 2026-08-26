use std::{error::Error, fmt::Display};

use crate::{logic::LogicError, persistence::DBError};
#[derive(Debug)]
pub enum BartendError {
    LogicError(LogicError),
    DBError(DBError),
}

impl From<DBError> for BartendError {
    fn from(value: DBError) -> Self {
        BartendError::DBError(value)
    }
}
impl From<LogicError> for BartendError {
    fn from(value: LogicError) -> Self {
        BartendError::LogicError(value)
    }
}
impl Display for BartendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BartendError::LogicError(logic_error) => write!(f, "{logic_error}"),
            BartendError::DBError(dberror) => write!(f, "{dberror}"),
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
