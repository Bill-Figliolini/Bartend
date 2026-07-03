use std::collections::HashMap;

use crate::{
    models::{Category, CategoryBody, CategoryFilter, CategoryID, ItemID},
    persistence::{Database, repositories::CategoryRepository},
};

#[derive(Debug)]
pub struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
    item_mapping: HashMap<ItemID, CategoryID>,
}

impl CategoryService {
    //bulk load data at start
    pub fn new(db: &Database) -> Self {
        let categories = HashMap::new();
        let item_mapping = HashMap::new();
        CategoryService {
            categories,
            item_mapping,
        }
    }

    pub fn item_category(&self, item: &ItemID) -> Option<&CategoryID> {
        self.item_mapping.get(item)
    }
    #[must_use]
    pub fn get_all(&self, _filter: CategoryFilter) -> Vec<Category> {
        self.categories
            .iter()
            .map(|(id, body)| Category {
                id: id.clone(),
                body: body.clone(),
            })
            .collect()
    }

    pub fn get(&self, id: &CategoryID) -> &CategoryBody {
        match self.categories.get(id) {
            Some(body) => body,
            None => panic!("Invalid CategoryID in cirulation!"),
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
                "Categories attempted to update nonexistent category {}",
                category
            );
        }
    }
}
