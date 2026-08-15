use std::collections::{HashMap, HashSet};

use crate::{
    logic::{LogicError, graph::DirectedAcyclicGraph},
    models::{Category, CategoryBody, CategoryFilter, CategoryID, ItemID},
    persistence::repositories::CategoryRepository,
};

#[derive(Debug)]
pub struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
    category_mapping: HashMap<CategoryID, HashSet<ItemID>>,
    item_mapping: HashMap<ItemID, CategoryID>,
    graph: DirectedAcyclicGraph<CategoryID>,
}

impl CategoryService {
    //bulk load data at start
    pub fn new(db: &impl CategoryRepository) -> Self {
        let categories = match db.get_all() {
            Ok(map) => map,
            Err(_) => panic!("Error reading category DB"),
        };

        let item_mapping = match db.get_map() {
            Ok(map) => map,
            Err(_) => panic!("Error reading mapping DB"),
        };

        let category_mapping = item_mapping.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<CategoryID, HashSet<ItemID>>, (item, category)| {
                match acc.get_mut(category) {
                    Some(set) => {
                        set.insert(*item);
                    }
                    None => {
                        let mut new_set = HashSet::new();
                        new_set.insert(*item);
                        acc.insert(*category, new_set);
                    }
                }
                acc
            },
        );
        let category_relations = db.get_graph().unwrap();
        let graph = DirectedAcyclicGraph::load(category_relations);
        CategoryService {
            categories,
            item_mapping,
            category_mapping,
            graph,
        }
    }
    pub fn item_category(&self, item: &ItemID) -> Option<CategoryID> {
        self.item_mapping.get(item).copied()
    }
    pub fn satisfying_items(&self, category: &CategoryID) -> HashSet<ItemID> {
        let mut items = HashSet::new();
        items.extend(self.get_items(category));
        if let Some(categories_to_add) = self.graph.get_all_children(category) {
            categories_to_add
                .into_iter()
                .fold(&mut items, |acc, category| {
                    acc.extend(self.get_items(&category));
                    acc
                });
        }
        items
    }

    pub fn child_categories(&self, category: &CategoryID) -> HashSet<CategoryID> {
        match self.graph.get_edges(category) {
            Some(children) => children,
            None => panic!(
                "Something has gone wrong, attempted to get category: {:?}\n current state of the graph: {:?}\nstate of categories: {:?}",
                *category, self.graph, self.categories
            ),
        }
    }

    #[must_use]
    pub fn get_all(&self, _filter: CategoryFilter) -> Vec<Category> {
        self.categories
            .iter()
            .map(|(id, body)| Category {
                id: *id,
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

    pub fn get_page(&self, page_number: usize, page_size: usize) -> Vec<CategoryID> {
        let page_offset = page_number * page_size;
        self.categories
            .keys()
            .copied()
            .skip(page_offset)
            .take(page_size)
            .collect()
    }

    pub fn get_items(&self, category: &CategoryID) -> HashSet<ItemID> {
        self.category_mapping
            .get(category)
            .unwrap_or(&HashSet::new())
            .iter()
            .cloned()
            .collect()
    }

    //Writes, cache invalidating
    pub fn insert(&mut self, db: &impl CategoryRepository, body: &CategoryBody) -> CategoryID {
        match db.insert(body) {
            Ok(id) => {
                self.categories.insert(id, body.clone());
                self.graph.insert_vertex(id);
                id
            }
            Err(e) => panic!("{e}"),
        }
    }
    //TODO: Need to fix for potiental error cases on db fail
    pub fn delete(&mut self, db: &impl CategoryRepository, category: CategoryID) {
        let patch = self.graph.get_removal_patch(&category);
        if let Err(e) = db.delete(category) {
            panic!("{e}")
        }
        if let Err(e) = db.delete_node(&patch) {
            panic!("{e}")
        }
        self.categories.remove(&category);
        self.graph.remove(patch);
    }
    pub fn update(&mut self, db: &impl CategoryRepository, category: &Category) {
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
        db: &impl CategoryRepository,
        item: &ItemID,
        category: &CategoryID,
    ) {
        if let Err(e) = db.map_insert(item, category) {
            panic!("{e}");
        }
        self.item_mapping.insert(*item, *category);
    }
    pub fn update_item_mapping(
        &mut self,
        db: &impl CategoryRepository,
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
                if let Err(e) = db.map_delete(item, &old) {
                    panic!("{e}");
                }
            }
            (Some(old), Some(new)) => {
                if let Err(e) = db.map_delete(item, &old) {
                    panic!("{e}");
                }
                if let Err(e) = db.map_insert(item, &new) {
                    panic!("{e}");
                }
                let old_mapping = self.item_mapping.get_mut(item).unwrap();
                *old_mapping = *new;
            }
            (None, None) => {}
        }
    }
    pub fn add_category_relation(
        &mut self,
        db: &impl CategoryRepository,
        parent: &CategoryID,
        child: &CategoryID,
    ) -> Result<(), LogicError> {
        if let Err(_) = self.graph.insert_edge(parent, child) {
            Err(LogicError::InvalidCategoryRelation {
                parent: *parent,
                child: *child,
            })
        } else {
            if let Err(e) = db.insert_relation(*parent, *child) {
                panic!("{e}")
            };
            Ok(())
        }
    }
    pub fn remove_category_relation(
        &mut self,
        db: &impl CategoryRepository,
        parent: &CategoryID,
        child: &CategoryID,
    ) {
        self.graph.remove_edge(parent, child);

        if let Err(e) = db.delete_edge(*parent, *child) {
            panic!("{e}")
        };
    }
    pub fn valid_relations(&self, category: &CategoryID) -> HashSet<CategoryID> {
        match self.graph.get_non_cyclic_additions(category) {
            Some(candidates) => candidates,
            None => HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logic::GraphPatch;

    use super::*;

    struct TestDB {
        counter: i64,
    }
    impl TestDB {
        fn new() -> Self {
            Self { counter: 30 }
        }
        fn _get_next(&self) -> CategoryID {
            CategoryID(self.counter + 1)
        }
        fn _update(&mut self) {
            self.counter += 1;
        }
    }
    impl CategoryRepository for TestDB {
        fn insert(&self, _body: &CategoryBody) -> Result<CategoryID, crate::persistence::DBError> {
            let next = CategoryID(self.counter);
            Ok(next)
        }

        fn update(&self, _item: &Category) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn delete(&self, _item: CategoryID) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn get_all(
            &self,
        ) -> Result<HashMap<CategoryID, CategoryBody>, crate::persistence::DBError> {
            let mut result = HashMap::new();
            for i in 0..30 {
                result.insert(
                    CategoryID(i),
                    CategoryBody {
                        name: format!("category {}", i + 1),
                    },
                );
            }
            Ok(result)
        }
        fn get_graph(
            &self,
        ) -> Result<HashMap<CategoryID, HashSet<CategoryID>>, crate::persistence::DBError> {
            Ok(HashMap::new())
        }

        fn insert_relation(
            &self,
            _parent: CategoryID,
            _child: CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn delete_node(
            &self,
            _patch: &GraphPatch<CategoryID>,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn delete_edge(
            &self,
            _parent: CategoryID,
            _child: CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }
        fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, crate::persistence::DBError> {
            let map = HashMap::new();
            Ok(map)
        }
        fn map_insert(
            &self,
            _item: &ItemID,
            _category: &CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn map_delete(
            &self,
            _item: &ItemID,
            _category: &CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }
    }
    #[test]
    fn category_service_loads_in_all_data() {
        let db = TestDB::new();
        let category_service = CategoryService::new(&db);

        assert_eq!(category_service.categories.len(), 30);
    }
    mod category_mangement {
        //For synchronization behaviors
        use super::*;
        mod item_addition {
            use super::*;
            #[test]
            fn initializes_id_in_table_and_graph() {
                let db = TestDB::new();
                let mut service = CategoryService::new(&db);
                let stub_body = CategoryBody {
                    name: "test".to_string(),
                };
                let next_id = db._get_next();
                assert!(!service.graph.contains_vertex(&next_id));

                let id = service.insert(&db, &stub_body);

                assert!(service.categories.contains_key(&id));
                assert!(service.graph.contains_vertex(&id));
                assert_eq!(service.categories.get(&id).unwrap().name, stub_body.name);
            }
        }
        mod item_removal {
            use super::*;
            #[test]
            fn removes_id_from_table_and_graph() {
                let db = TestDB::new();
                let mut service = CategoryService::new(&db);
                let stub_body = CategoryBody {
                    name: "test".to_string(),
                };
                let id = service.insert(&db, &stub_body);
                assert!(service.categories.contains_key(&id));
                assert!(service.graph.contains_vertex(&id));

                service.delete(&db, id.clone());

                assert!(!service.categories.contains_key(&id));
                assert!(!service.graph.contains_vertex(&id));
            }
        }
        mod item_update {
            use super::*;
            #[test]
            fn updates_in_table() {
                let db = TestDB::new();
                let mut service = CategoryService::new(&db);
                let initial_body = CategoryBody {
                    name: "test".to_string(),
                };
                let updated_body = CategoryBody {
                    name: "next".to_string(),
                };
                let id = service.insert(&db, &initial_body);
                assert!(service.categories.contains_key(&id));
                assert_eq!(service.categories.get(&id).unwrap().name, initial_body.name);

                service.update(
                    &db,
                    &Category {
                        id,
                        body: updated_body.clone(),
                    },
                );

                assert_eq!(service.categories.get(&id).unwrap().name, updated_body.name);
            }
        }
    }
    mod item_mapping {
        //mapping specific behavior
        use super::*;
    }
    mod category_resolution {
        //graph specific behavior
        use super::*;
    }
}
