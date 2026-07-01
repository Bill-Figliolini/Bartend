use std::collections::HashMap;

use crate::{
    models::{Category, CategoryBody, CategoryFilter, CategoryID},
    persistence::repositories::CategoryRepository,
};

#[derive(Debug)]
pub struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
}

impl CategoryService {
    #[must_use]
    pub fn get(&self, _filter: CategoryFilter, db: &impl CategoryRepository) -> Vec<Category> {
        match db.get_range(0, 100) {
            Ok(categories) => categories,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn add_category(&self, body: &CategoryBody, db: &impl CategoryRepository) -> CategoryID {
        match db.insert(body) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }
    pub fn delete_category(&self, category: Category, db: &impl CategoryRepository) {
        if let Err(e) = db.delete(category) {
            panic!("{e}")
        }
    }
    pub fn update_category(&self, category: &Category, db: &impl CategoryRepository) {
        if let Err(e) = db.update(category) {
            panic!("{e}")
        }
    }
}
