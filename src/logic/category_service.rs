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
    pub fn item_category(&self, item: ItemID) -> Option<CategoryID> {
        self.item_mapping.get(&item).copied()
    }
    pub fn satisfying_items(&self, category: CategoryID) -> HashSet<ItemID> {
        let mut items = HashSet::new();
        items.extend(self.get_items(category));
        if let Some(categories_to_add) = self.graph.get_all_children(&category) {
            categories_to_add
                .into_iter()
                .fold(&mut items, |acc, category| {
                    acc.extend(self.get_items(category));
                    acc
                });
        }
        items
    }

    pub fn child_categories(
        &self,
        category: CategoryID,
    ) -> Result<HashSet<CategoryID>, BartendError> {
        match self.graph.get_edges(&category) {
            Some(children) => Ok(children),
            None => Err(LogicError::CategoryNotInGraph(category))?,
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

    pub fn get(&self, id: CategoryID) -> Result<&CategoryBody, BartendError> {
        match self.categories.get(&id) {
            //another logic error
            Some(body) => Ok(body),
            None => Err(LogicError::InvalidCategory(id))?,
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

    pub fn get_items(&self, category: CategoryID) -> HashSet<ItemID> {
        self.category_mapping
            .get(&category)
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
        item: ItemID,
        category: CategoryID,
    ) -> Result<(), BartendError> {
        db.map_insert(item, category)?;
        self.category_mapping
            .entry(category)
            .or_default()
            .insert(item);
        self.item_mapping.insert(item, category);
        Ok(())
    }
    pub fn update_item_mapping(
        &mut self,
        db: &impl CategoryRepository,
        item: ItemID,
        category: Option<CategoryID>,
    ) -> Result<(), BartendError> {
        let mut old_category = self.item_category(item);
        match (old_category.take(), category) {
            (None, Some(new)) => {
                self.add_item_mapping(db, item, new)?;
            }
            (Some(old), None) => {
                db.map_delete(item, old)?;
                self.item_mapping.remove(&item);
                match self.category_mapping.entry(old) {
                    Entry::Occupied(mut items) => {
                        items.get_mut().remove(&item);
                    }
                    Entry::Vacant(_) => {
                        Err(LogicError::InvalidCategory(old))?;
                    }
                }
            }
            (Some(old), Some(new)) => {
                db.map_delete(item, old)?;
                match self.category_mapping.entry(old) {
                    Entry::Occupied(mut items) => {
                        items.get_mut().remove(&item);
                    }
                    Entry::Vacant(_) => {
                        Err(LogicError::InvalidCategory(old))?;
                    }
                }
                self.item_mapping.remove(&item);
                self.add_item_mapping(db, item, new)?;
            }
            (None, None) => {}
        }
        Ok(())
    }
    pub fn add_category_relation(
        &mut self,
        db: &impl CategoryRepository,
        parent: CategoryID,
        child: CategoryID,
    ) -> Result<(), BartendError> {
        match self.graph.insert_edge(&parent, &child) {
            Ok(()) => {
                db.insert_relation(parent, child)?;
                Ok(())
            }
            Err(graph_error) => match graph_error {
                super::graph::GraphError::EdgeEndpointNotInGraph => {
                    Err(LogicError::CategoryNotInGraph(parent))?
                }
                super::graph::GraphError::WouldIntroduceCycle => {
                    Err(LogicError::InvalidCategoryRelation { parent, child })?
                }
            },
        }
    }
    pub fn remove_category_relation(
        &mut self,
        db: &impl CategoryRepository,
        parent: CategoryID,
        child: CategoryID,
    ) -> Result<(), BartendError> {
        db.delete_edge(parent, child)?;
        self.graph.remove_edge(&parent, &child);
        Ok(())
    }
    pub fn valid_relations(&self, category: CategoryID) -> HashSet<CategoryID> {
        self.graph
            .get_non_cyclic_additions(&category)
            .unwrap_or_default()
    }
    pub fn category_count(&self) -> usize {
        self.categories.len()
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
        fn get_next(&self) -> CategoryID {
            CategoryID(self.counter + 1)
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
            _item: ItemID,
            _category: CategoryID,
        ) -> Result<(), crate::persistence::DBError> {
            Ok(())
        }

        fn map_delete(
            &self,
            _item: ItemID,
            _category: CategoryID,
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
                let next_id = db.get_next();
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

                _ = service.delete(&db, id);

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

            _ = service.add_item_mapping(db, item, category);

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
        use super::*;

        fn service() -> (TestDB, CategoryService) {
            let db = TestDB::new();
            let service = CategoryService::new(&db).unwrap();
            (db, service)
        }

        #[test]
        fn satisfying_items_include_subcategory_items() {
            let (db, mut service) = service();
            let (spirit, gin) = (CategoryID(0), CategoryID(1));
            service.add_category_relation(&db, spirit, gin).unwrap();
            service.add_item_mapping(&db, ItemID(5), gin).unwrap();
            service.add_item_mapping(&db, ItemID(6), spirit).unwrap();

            assert_eq!(
                service.satisfying_items(spirit),
                HashSet::from([ItemID(5), ItemID(6)])
            );
            assert_eq!(service.satisfying_items(gin), HashSet::from([ItemID(5)]));
            assert!(service.satisfying_items(CategoryID(999)).is_empty());
        }

        #[test]
        fn child_categories_lists_direct_children() {
            let (db, mut service) = service();
            service
                .add_category_relation(&db, CategoryID(0), CategoryID(1))
                .unwrap();
            service
                .add_category_relation(&db, CategoryID(1), CategoryID(2))
                .unwrap();

            assert_eq!(
                service.child_categories(CategoryID(0)).unwrap(),
                HashSet::from([CategoryID(1)])
            );
            assert!(matches!(
                service.child_categories(CategoryID(999)),
                Err(BartendError::Logic(LogicError::CategoryNotInGraph(_)))
            ));
        }

        #[test]
        fn cyclic_relations_are_rejected() {
            let (db, mut service) = service();
            service
                .add_category_relation(&db, CategoryID(0), CategoryID(1))
                .unwrap();

            assert!(matches!(
                service.add_category_relation(&db, CategoryID(1), CategoryID(0)),
                Err(BartendError::Logic(
                    LogicError::InvalidCategoryRelation { .. }
                ))
            ));
        }

        #[test]
        fn relations_to_unknown_categories_are_rejected() {
            let (db, mut service) = service();
            assert!(matches!(
                service.add_category_relation(&db, CategoryID(0), CategoryID(999)),
                Err(BartendError::Logic(LogicError::CategoryNotInGraph(_)))
            ));
        }

        #[test]
        fn removing_a_relation_drops_the_edge() {
            let (db, mut service) = service();
            service
                .add_category_relation(&db, CategoryID(0), CategoryID(1))
                .unwrap();

            service
                .remove_category_relation(&db, CategoryID(0), CategoryID(1))
                .unwrap();

            assert!(service.child_categories(CategoryID(0)).unwrap().is_empty());
        }

        #[test]
        fn valid_relations_exclude_self_ancestors_and_children() {
            let (db, mut service) = service();
            service
                .add_category_relation(&db, CategoryID(0), CategoryID(1))
                .unwrap();
            service
                .add_category_relation(&db, CategoryID(1), CategoryID(2))
                .unwrap();

            let valid = service.valid_relations(CategoryID(1));

            assert_eq!(valid.len(), 27);
            for excluded in [0, 1, 2] {
                assert!(!valid.contains(&CategoryID(excluded)));
            }
            assert!(service.valid_relations(CategoryID(999)).is_empty());
        }
    }
    mod lookup {
        use super::*;

        #[test]
        fn new_groups_stored_item_mappings_by_category() {
            use crate::{
                models::{ItemBody, Quantity, Unit},
                persistence::{Database, repositories::ItemRepository},
            };
            let db = Database::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
            let category = |name: &str| {
                db.category_db()
                    .insert(&CategoryBody {
                        name: name.to_string(),
                    })
                    .unwrap()
            };
            let item = |name: &str| {
                db.item_db()
                    .insert(&ItemBody {
                        name: name.to_string(),
                        quantity: Quantity::new(1.0, Unit::Milliliter),
                    })
                    .unwrap()
            };
            let (gin, vodka) = (category("Gin"), category("Vodka"));
            let (a, b, c) = (item("a"), item("b"), item("c"));
            for (item, category) in [(a, gin), (b, gin), (c, vodka)] {
                db.category_db().map_insert(item, category).unwrap();
            }
            db.category_db().insert_relation(gin, vodka).unwrap();

            let service = CategoryService::new(&db.category_db()).unwrap();

            assert_eq!(service.get_items(gin), HashSet::from([a, b]));
            assert_eq!(service.item_category(c), Some(vodka));
            assert_eq!(
                service.child_categories(gin).unwrap(),
                HashSet::from([vodka])
            );
        }

        #[test]
        fn get_all_returns_every_category() {
            let service = CategoryService::new(&TestDB::new()).unwrap();
            assert_eq!(service.get_all(CategoryFilter {}).len(), 30);
            assert_eq!(service.category_count(), 30);
        }

        #[test]
        fn getting_an_unknown_category_errors() {
            let service = CategoryService::new(&TestDB::new()).unwrap();
            assert_eq!(service.get(CategoryID(3)).unwrap().name, "category 4");
            assert!(matches!(
                service.get(CategoryID(999)),
                Err(BartendError::Logic(LogicError::InvalidCategory(_)))
            ));
        }

        #[test]
        fn pages_are_sorted_by_name() {
            let service = CategoryService::new(&TestDB::new()).unwrap();
            // Names sort as text: "category 1", "category 10", ...
            assert_eq!(service.get_page(0, 2), vec![CategoryID(0), CategoryID(9)]);
            assert_eq!(service.get_page(15, 2), Vec::new());
        }

        #[test]
        fn updating_an_unknown_category_errors() {
            let db = TestDB::new();
            let mut service = CategoryService::new(&db).unwrap();
            let ghost = Category {
                id: CategoryID(999),
                body: CategoryBody::default(),
            };
            assert!(matches!(
                service.update(&db, &ghost),
                Err(BartendError::Logic(LogicError::InvalidCategory(_)))
            ));
        }

        #[test]
        fn deleting_a_category_unmaps_its_items() {
            let db = TestDB::new();
            let mut service = CategoryService::new(&db).unwrap();
            service
                .add_item_mapping(&db, ItemID(1), CategoryID(0))
                .unwrap();

            service.delete(&db, CategoryID(0)).unwrap();

            assert_eq!(service.item_category(ItemID(1)), None);
            assert!(service.get_items(CategoryID(0)).is_empty());
        }
    }
    mod update_item_mapping {
        use super::*;

        fn mapped(item: ItemID, category: Option<CategoryID>) -> (TestDB, CategoryService) {
            let db = TestDB::new();
            let mut service = CategoryService::new(&db).unwrap();
            if let Some(category) = category {
                service.add_item_mapping(&db, item, category).unwrap();
            }
            (db, service)
        }

        #[test]
        fn adds_a_category_to_an_unmapped_item() {
            let (db, mut service) = mapped(ItemID(1), None);
            service
                .update_item_mapping(&db, ItemID(1), Some(CategoryID(2)))
                .unwrap();
            assert_eq!(service.item_category(ItemID(1)), Some(CategoryID(2)));
        }

        #[test]
        fn removes_the_category() {
            let (db, mut service) = mapped(ItemID(1), Some(CategoryID(2)));
            service.update_item_mapping(&db, ItemID(1), None).unwrap();
            assert_eq!(service.item_category(ItemID(1)), None);
            assert!(service.get_items(CategoryID(2)).is_empty());
        }

        #[test]
        fn moves_the_item_between_categories() {
            let (db, mut service) = mapped(ItemID(1), Some(CategoryID(2)));
            service
                .update_item_mapping(&db, ItemID(1), Some(CategoryID(3)))
                .unwrap();
            assert_eq!(service.item_category(ItemID(1)), Some(CategoryID(3)));
            assert!(service.get_items(CategoryID(2)).is_empty());
            assert_eq!(service.get_items(CategoryID(3)), HashSet::from([ItemID(1)]));
        }

        #[test]
        fn leaving_an_unmapped_item_unmapped_does_nothing() {
            let (db, mut service) = mapped(ItemID(1), None);
            service.update_item_mapping(&db, ItemID(1), None).unwrap();
            assert_eq!(service.item_category(ItemID(1)), None);
        }
    }
}
