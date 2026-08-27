use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryID(pub i64);
#[derive(Debug, Clone)]
pub struct Category {
    pub id: CategoryID,
    pub body: CategoryBody,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBody {
    pub name: String,
}
impl Category {}

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

impl Default for CategoryBody {
    fn default() -> Self {
        Self {
            name: "Invalid Category".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CategoryFilter {}
