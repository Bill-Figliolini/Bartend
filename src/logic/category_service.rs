use std::collections::HashMap;

use crate::models::{CategoryBody, CategoryCommand, CategoryError, CategoryID, Channel};

#[derive(Debug)]
pub(super) struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
}

impl CategoryService {
    pub(super) fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }
}

type CategoryServiceChannel = Channel<CategoryError, CategoryCommand>;
