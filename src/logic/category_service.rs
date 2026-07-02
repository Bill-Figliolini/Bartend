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
    pub fn new() -> Self {
        CategoryService {
            categories: HashMap::new(),
        }
    }

    #[must_use]
    pub fn get(&self, _filter: CategoryFilter, db: &impl CategoryRepository) -> Vec<Category> {
        match db.get_range(0, 100) {
            Ok(categories) => categories,
            Err(e) => panic!("{e}"),
        }
    }

    //Writes, cache invalidating
    pub fn insert(&mut self, body: &CategoryBody, db: &impl CategoryRepository) -> CategoryID {
        match db.insert(body) {
            Ok(id) => {
                self.categories.insert(id, body.clone());
                id
            }
            Err(e) => panic!("{e}"),
        }
    }
    pub fn delete(&mut self, category: Category, db: &impl CategoryRepository) {
        self.categories.remove(&category.id);
        if let Err(e) = db.delete(category) {
            panic!("{e}")
        }
    }
    pub fn update(&mut self, category: &Category, db: &impl CategoryRepository) {
        if let Some(cached_copy) = self.categories.get_mut(&category.id) {
            *cached_copy = category.body.clone();
            if let Err(e) = db.update(category) {
                panic!("{e}")
            }
        } else {
            panic!(
                "Categories attempted to update uncached category {}",
                category
            );
        }
    }
}
