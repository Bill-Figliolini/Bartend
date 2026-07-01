use std::fmt::Display;

use crate::models::Channel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryID(pub i64);
#[derive(Debug, Clone)]
pub struct Category {
    pub id: CategoryID,
    pub body: CategoryBody,
}
#[derive(Debug, Clone)]
pub struct CategoryBody {
    pub name: String,
}
impl Category {}

pub enum CategoryCommand {}

pub enum CategoryError {}

type CategoryUserChannel = Channel<CategoryCommand, CategoryError>;

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body.name)
    }
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for CategoryID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
