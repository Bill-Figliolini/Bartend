use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::{
    logic::{LogicError, graph::DirectedAcyclicGraph},
    models::{BartendError, Category, CategoryBody, CategoryFilter, CategoryID, ItemID},
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
    pub fn new(db: &impl CategoryRepository) -> Result<Self, BartendError> {
        let categories = db.get_all()?;

        let item_mapping = db.get_map()?;

        let category_mapping = item_mapping.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<CategoryID, HashSet<ItemID>>, (item, category)| {
                if let Some(set) = acc.get_mut(category) {
                    set.insert(*item);
                } else {
                    let mut new_set = HashSet::new();
                    new_set.insert(*item);
                    acc.insert(*category, new_set);
                }
                acc
            },
        );
        let category_relations = db.get_graph()?;
        let mut graph = DirectedAcyclicGraph::load(category_relations);
        for category in categories.keys() {
            graph.insert_vertex(*category);
        }
        Ok(CategoryService {
            categories,
            category_mapping,
            item_mapping,
            graph,
        })
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

    pub fn child_categories(
        &self,
        category: &CategoryID,
    ) -> Result<HashSet<CategoryID>, BartendError> {
        match self.graph.get_edges(category) {
            Some(children) => Ok(children),
            None => Err(LogicError::CategoryNotInGraph(*category))?,
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

    pub fn get(&self, id: &CategoryID) -> Result<&CategoryBody, BartendError> {
        match self.categories.get(id) {
            //another logic error
            Some(body) => Ok(body),
            None => Err(LogicError::InvalidCategory(*id))?,
        }
    }

    pub fn get_page(&self, page_number: usize, page_size: usize) -> Vec<CategoryID> {
        let page_offset = page_number * page_size;
        self.get_sorted()
            .into_iter()
            .skip(page_offset)
            .take(page_size)
            .collect()
    }

    fn get_sorted(&self) -> Vec<CategoryID> {
        let mut entries: Vec<(String, CategoryID)> = self
            .categories
            .iter()
            .map(|(id, body)| (body.name.to_lowercase(), *id))
            .collect();
        entries.sort_unstable_by(|(a_name, a_id), (b_name, b_id)| {
            a_name.cmp(b_name).then_with(|| a_id.0.cmp(&b_id.0))
        });
        entries.into_iter().map(|(_, id)| id).collect()
    }

    pub fn get_items(&self, category: &CategoryID) -> HashSet<ItemID> {
        self.category_mapping
            .get(category)
            .unwrap_or(&HashSet::new())
            .iter()
            .copied()
            .collect()
    }

    //Writes, cache invalidating
    pub fn insert(
        &mut self,
        db: &impl CategoryRepository,
        body: &CategoryBody,
    ) -> Result<CategoryID, BartendError> {
        let id = db.insert(body)?;
        self.categories.insert(id, body.clone());
        self.graph.insert_vertex(id);
        Ok(id)
    }
    pub fn delete(
        &mut self,
        db: &impl CategoryRepository,
        category: CategoryID,
    ) -> Result<(), BartendError> {
        let patch = self.graph.get_removal_patch(&category);
        db.delete(&patch)?;
        self.categories.remove(&category);
        self.graph.remove(patch);
        self.category_mapping.remove(&category);
        self.item_mapping.retain(|_, value| value != &category);
        Ok(())
    }
    pub fn update(
        &mut self,
        db: &impl CategoryRepository,
        category: &Category,
    ) -> Result<(), BartendError> {
        db.update(category)?;
        if let Some(cached_copy) = self.categories.get_mut(&category.id) {
            *cached_copy = category.body.clone();
            Ok(())
        } else {
            Err(LogicError::InvalidCategory(category.id))?
        }
    }
    pub fn add_item_mapping(
        &mut self,
        db: &impl CategoryRepository,
        item: &ItemID,
        category: &CategoryID,
    ) -> Result<(), BartendError> {
        db.map_insert(item, category)?;
        self.category_mapping
            .entry(*category)
            .or_default()
            .insert(*item);
        self.item_mapping.insert(*item, *category);
        Ok(())
    }
    pub fn update_item_mapping(
        &mut self,
        db: &impl CategoryRepository,
        item: &ItemID,
        category: &Option<CategoryID>,
    ) -> Result<(), BartendError> {
        let mut old_category = self.item_category(item);
        match (old_category.take(), category) {
            (None, Some(new)) => {
                self.add_item_mapping(db, item, new)?;
            }
            (Some(old), None) => {
                db.map_delete(item, &old)?;
                self.item_mapping.remove(item);
                match self.category_mapping.entry(old) {
                    Entry::Occupied(mut items) => {
                        items.get_mut().remove(item);
                    }
                    Entry::Vacant(_) => {
                        Err(LogicError::InvalidCategory(old))?;
                    }
                }
            }
            (Some(old), Some(new)) => {
                db.map_delete(item, &old)?;
                match self.category_mapping.entry(old) {
                    Entry::Occupied(mut items) => {
                        items.get_mut().remove(item);
                    }
                    Entry::Vacant(_) => {
                        Err(LogicError::InvalidCategory(old))?;
                    }
                }
                self.item_mapping.remove(item);
                self.add_item_mapping(db, item, new)?;
            }
            (None, None) => {}
        }
        Ok(())
    }
    pub fn add_category_relation(
        &mut self,
        db: &impl CategoryRepository,
        parent: &CategoryID,
        child: &CategoryID,
    ) -> Result<(), BartendError> {
        match self.graph.insert_edge(parent, child) {
            Ok(()) => {
                db.insert_relation(*parent, *child)?;
                Ok(())
            }
            Err(graph_error) => match graph_error {
                super::graph::GraphError::EdgeEndpointNotInGraph => {
                    Err(LogicError::CategoryNotInGraph(*parent))?
                }
                super::graph::GraphError::WouldIntroduceCycle => {
                    Err(LogicError::InvalidCategoryRelation {
                        parent: *parent,
                        child: *child,
                    })?
                }
            },
        }
    }
    pub fn remove_category_relation(
        &mut self,
        db: &impl CategoryRepository,
        parent: &CategoryID,
        child: &CategoryID,
    ) -> Result<(), BartendError> {
        db.delete_edge(*parent, *child)?;
        self.graph.remove_edge(parent, child);
        Ok(())
    }
    pub fn valid_relations(&self, category: &CategoryID) -> HashSet<CategoryID> {
        self.graph
            .get_non_cyclic_additions(category)
            .unwrap_or_default()
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

        fn delete(
            &self,
            _item: &GraphPatch<CategoryID>,
        ) -> Result<(), crate::persistence::DBError> {
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
        let category_service = CategoryService::new(&db).unwrap();

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
                let mut service = CategoryService::new(&db).unwrap();
                let stub_body = CategoryBody {
                    name: "test".to_string(),
                };
                let next_id = db._get_next();
                assert!(!service.graph.contains_vertex(&next_id));

                let id = service.insert(&db, &stub_body).expect("Should be valid");

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
                let mut service = CategoryService::new(&db).unwrap();
                let stub_body = CategoryBody {
                    name: "test".to_string(),
                };
                let id = service.insert(&db, &stub_body).expect("should be valid");
                assert!(service.categories.contains_key(&id));
                assert!(service.graph.contains_vertex(&id));

                _ = service.delete(&db, id.clone());

                assert!(!service.categories.contains_key(&id));
                assert!(!service.graph.contains_vertex(&id));
            }
        }
        mod item_update {
            use super::*;
            #[test]
            fn updates_in_table() {
                let db = TestDB::new();
                let mut service = CategoryService::new(&db).unwrap();
                let initial_body = CategoryBody {
                    name: "test".to_string(),
                };
                let updated_body = CategoryBody {
                    name: "next".to_string(),
                };
                let id = service.insert(&db, &initial_body).expect("valid for test");
                assert!(service.categories.contains_key(&id));
                assert_eq!(service.categories.get(&id).unwrap().name, initial_body.name);

                _ = service.update(
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

        #[test]
        fn addition_updates_both_mappings() {
            let db = &TestDB::new();
            let mut service = CategoryService::new(db).unwrap();
            let item = ItemID(0);
            let category = CategoryID(0);

            _ = service.add_item_mapping(db, &item, &category);

            assert!(
                service
                    .category_mapping
                    .get(&category)
                    .unwrap()
                    .contains(&item)
            );
            assert_eq!(service.item_mapping.get(&item).unwrap(), &category);
        }
    }
    mod category_resolution {
        //graph specific behavior
    }
}
