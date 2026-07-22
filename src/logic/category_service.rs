use std::collections::{HashMap, HashSet};

use crate::{
    logic::{LogicError, graph::DirectedAcyclicGraph},
    models::{Category, CategoryBody, CategoryFilter, CategoryID, ItemID},
    persistence::repositories::{CategoryRepository, GraphRepository, ItemMappingRepository},
};

#[derive(Debug)]
pub struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
    category_mapping: HashMap<CategoryID, HashSet<ItemID>>,
    item_mapping: HashMap<ItemID, CategoryID>,
    graph: DirectedAcyclicGraph<CategoryID>,

    page_size: usize,
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
        let category_relations = db.graph().get().unwrap();
        let graph = DirectedAcyclicGraph::load(category_relations);
        CategoryService {
            categories,
            item_mapping,
            category_mapping,
            graph,
            page_size: 15,
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

    pub fn get_page(&self, page_number: usize) -> Vec<CategoryID> {
        let page_offset = page_number * self.page_size;
        self.categories
            .keys()
            .copied()
            .skip(page_offset)
            .take(self.page_size)
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
    pub fn delete(&mut self, db: &impl CategoryRepository, category: CategoryID) {
        self.categories.remove(&category);
        self.graph.remove(category);
        if let Err(e) = db.delete(category) {
            panic!("{e}")
        }
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
        let db = db.mapping();
        if let Err(e) = db.insert(item, category) {
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
        let map_db = db.mapping();
        let mut old_category = self.item_category(item);
        match (old_category.take(), category) {
            (None, Some(new)) => {
                self.add_item_mapping(db, item, new);
            }
            (Some(old), None) => {
                self.item_mapping.remove(item);
                if let Err(e) = map_db.delete(item, &old) {
                    panic!("{e}");
                }
            }
            (Some(old), Some(new)) => {
                if let Err(e) = map_db.delete(item, &old) {
                    panic!("{e}");
                }
                if let Err(e) = map_db.insert(item, &old) {
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
        if let Err(_) = self.graph.insert_edge((parent, child)) {
            Err(LogicError::InvalidCategoryRelation {
                parent: *parent,
                child: *child,
            })
        } else {
            if let Err(e) = db.graph().insert(*parent, *child) {
                panic!("{e}")
            };
            Ok(())
        }
    }
    pub fn list_valid_relations(&self, category: &CategoryID) -> HashSet<CategoryID> {
        match self.graph.get_non_parents(category) {
            Some(candidates) => candidates,
            None => HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::repositories::{GraphRepository, Repository};

    use super::*;

    struct TestDB {
        counter: i64,
    }
    struct TestMapDB {}
    struct TestGraphDB {}
    impl TestDB {
        fn new() -> Self {
            Self { counter: 30 }
        }
        fn mapping_db(&self) -> TestMapDB {
            TestMapDB {}
        }
        fn graph_db(&self) -> TestGraphDB {
            TestGraphDB {}
        }
    }
    impl Repository for TestDB {
        fn create_table(&self) -> Result<(), crate::persistence::DBError> {
            Ok(())
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
        fn mapping(&self) -> impl ItemMappingRepository {
            self.mapping_db()
        }
        fn graph(&self) -> impl GraphRepository {
            self.graph_db()
        }

        fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, crate::persistence::DBError> {
            self.mapping().get_map()
        }
    }
    impl Repository for TestMapDB {
        fn create_table(&self) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }
    }
    impl ItemMappingRepository for TestMapDB {
        fn get_map(&self) -> Result<HashMap<ItemID, CategoryID>, crate::persistence::DBError> {
            let map = HashMap::new();
            Ok(map)
        }

        fn insert(
            &self,
            _item: &ItemID,
            _category: &CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn delete(
            &self,
            _item: &ItemID,
            _category: &CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }
    }
    impl Repository for TestGraphDB {
        fn create_table(&self) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }
    }
    impl GraphRepository for TestGraphDB {
        fn get(
            &self,
        ) -> Result<HashMap<CategoryID, HashSet<CategoryID>>, crate::persistence::DBError> {
            Ok(HashMap::new())
        }

        fn insert(
            &self,
            _parent: CategoryID,
            _child: CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn delete_node(&self, _node: CategoryID) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn delete_edge(
            &self,
            _parent: CategoryID,
            _child: CategoryID,
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
}
