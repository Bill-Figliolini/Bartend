use std::collections::HashMap;

use crate::{
    logic::graph::DirectedAcyclicGraph,
    models::{Category, CategoryBody, CategoryFilter, CategoryID, ItemID},
    persistence::{
        Database,
        repositories::{CategoryRepository, ItemMappingRepository},
    },
};

#[derive(Debug)]
pub struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
    item_mapping: HashMap<ItemID, CategoryID>,
    graph: DirectedAcyclicGraph<CategoryID>,
}

impl CategoryService {
    //bulk load data at start
    pub fn new(db: &Database) -> Self {
        let categories = match db.category_db().get_all() {
            Ok(map) => map,
            Err(_) => panic!("Error reading category DB"),
        };

        let item_mapping = match db.mapping_db().get_map() {
            Ok(map) => map,
            Err(_) => panic!("Error reading mapping DB"),
        };
        let graph = DirectedAcyclicGraph::new();
        CategoryService {
            categories,
            item_mapping,
            graph,
        }
    }
    pub fn item_category(&self, item: &ItemID) -> Option<CategoryID> {
        self.item_mapping.get(item).copied()
    }
    pub fn item_satisifies_category(&self, item: &ItemID, target_category: &CategoryID) -> bool {
        match (
            self.item_category(item),
            self.graph.get_all_children(target_category),
        ) {
            (Some(item_category), Some(category_set)) => category_set.contains(&item_category),
            (_, _) => false,
        }
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
    pub fn add_item_mapping(
        &mut self,
        db: &impl ItemMappingRepository,
        item: &ItemID,
        category: &CategoryID,
    ) {
        if let Err(e) = db.insert(item, category) {
            panic!("{e}");
        }
        self.item_mapping.insert(*item, *category);
    }
    pub fn update_item_mapping(
        &mut self,
        db: &impl ItemMappingRepository,
        item: &ItemID,
        category: &Option<CategoryID>,
    ) {
        let mut old_category = self.item_category(item);
        match (old_category.take(), category) {
            (None, Some(new)) => {
                self.add_item_mapping(db, item, new);
            }
            (Some(old), None) => {
                self.item_mapping.remove(item);
                if let Err(e) = db.delete(item, &old) {
                    panic!("{e}");
                }
            }
            (Some(old), Some(new)) => {
                if let Err(e) = db.delete(item, &old) {
                    panic!("{e}");
                }
                if let Err(e) = db.insert(item, &old) {
                    panic!("{e}");
                }
                let old_mapping = self.item_mapping.get_mut(item).unwrap();
                *old_mapping = *new;
            }
            (None, None) => {}
        }
    }
}
