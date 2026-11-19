use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::{
    logic::{LogicError, graph::DirectedAcyclicGraph, ids_by_name},
    models::{BartendError, Category, CategoryBody, CategoryFilter, CategoryID, ItemID},
    persistence::repositories::CategoryRepository,
};

/// One way down the category tree to a category, for showing where it sits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breadcrumb {
    /// From a top-level category down to the category itself.
    pub path: Vec<CategoryID>,
    /// How many other ways there are down to it, through other parents.
    pub other_paths: usize,
}

/// How many items count as a category, from [`CategoryService::item_counts`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemCounts {
    /// Items filed under the category itself.
    pub direct: usize,
    /// Items filed under the category or any of its subcategories.
    pub total: usize,
}

impl ItemCounts {
    /// Items that count only through a subcategory.
    pub fn via_subcategories(self) -> usize {
        self.total.saturating_sub(self.direct)
    }
}

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

    /// The category's name, or an empty one if it does not exist, such as
    /// for an id shown before a reload caught up with a delete.
    pub fn name_of(&self, id: CategoryID) -> String {
        self.categories
            .get(&id)
            .map_or_default(|body| body.name.clone())
    }

    #[cfg(test)]
    pub fn get_page(&self, page_number: usize, page_size: usize) -> Vec<CategoryID> {
        let page_offset = page_number * page_size;
        self.sorted_ids()
            .into_iter()
            .skip(page_offset)
            .take(page_size)
            .collect()
    }

    /// Every id, ordered by name (case-insensitively), then by id.
    pub fn sorted_ids(&self) -> Vec<CategoryID> {
        self.by_name(self.categories.keys().copied())
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
    /// Every category `item` counts towards: its own, and all of that
    /// category's ancestors. Empty for an item without a category.
    pub fn categories_satisfied_by(&self, item: ItemID) -> HashSet<CategoryID> {
        self.item_category(item)
            .map_or_else(HashSet::new, |category| self.self_and_ancestors(category))
    }

    /// `category` and every category above it: the ingredients something
    /// filed under `category` can fill. Empty for an unknown category.
    pub fn self_and_ancestors(&self, category: CategoryID) -> HashSet<CategoryID> {
        self.graph.get_ancestors(&category).unwrap_or_default()
    }

    /// The direct subcategories of `category`, ordered like
    /// [`sorted_ids`](Self::sorted_ids). Empty for an unknown category.
    pub fn children_by_name(&self, category: CategoryID) -> Vec<CategoryID> {
        self.by_name(self.graph.get_edges(&category).unwrap_or_default())
    }

    /// The categories `category` sits directly under, ordered like
    /// [`sorted_ids`](Self::sorted_ids). Empty for a top-level or unknown
    /// category.
    pub fn parents_by_name(&self, category: CategoryID) -> Vec<CategoryID> {
        self.by_name(
            self.graph
                .get_immediate_ancestors(&category)
                .unwrap_or_default(),
        )
    }

    /// How many items are filed under `category` itself, and how many count
    /// as it once its subcategories' items are included.
    pub fn item_counts(&self, category: CategoryID) -> ItemCounts {
        ItemCounts {
            direct: self.get_items(category).len(),
            total: self.satisfying_items(category).len(),
        }
    }

    /// `ids` ordered by name (case-insensitively), then by id.
    fn by_name(&self, ids: impl IntoIterator<Item = CategoryID>) -> Vec<CategoryID> {
        ids_by_name(ids.into_iter().map(|id| {
            let name = self
                .categories
                .get(&id)
                .map_or("", |body| body.name.as_str());
            (name, id)
        }))
    }

    /// The longest path from a top-level category down to `category`, with
    /// ties going to the path whose names come first alphabetically (from
    /// the top), and a count of the other paths. `None` for an unknown
    /// category.
    pub fn breadcrumb(&self, category: CategoryID) -> Option<Breadcrumb> {
        if !self.graph.contains_vertex(&category) {
            return None;
        }
        let (path, paths) = self.best_path(category, &mut HashMap::new());
        Some(Breadcrumb {
            path,
            other_paths: paths.saturating_sub(1),
        })
    }

    /// The [`breadcrumb`](Self::breadcrumb) path to `category` and the
    /// number of paths there are to it, memoized in `seen` so that shared
    /// ancestors are only walked once.
    fn best_path(
        &self,
        category: CategoryID,
        seen: &mut HashMap<CategoryID, (Vec<CategoryID>, usize)>,
    ) -> (Vec<CategoryID>, usize) {
        if let Some(known) = seen.get(&category) {
            return known.clone();
        }
        let parents = self
            .graph
            .get_immediate_ancestors(&category)
            .unwrap_or_default();
        let mut best: Option<Vec<CategoryID>> = None;
        let mut paths: usize = 0;
        for parent in parents {
            let (path, count) = self.best_path(parent, seen);
            paths = paths.saturating_add(count);
            if best
                .as_ref()
                .is_none_or(|current| self.path_precedes(&path, current))
            {
                best = Some(path);
            }
        }
        let mut path = best.unwrap_or_default();
        path.push(category);
        let result = (path, paths.max(1));
        seen.insert(category, result.clone());
        result
    }

    /// Whether path `a` should be shown over `b`: it is longer, or as long
    /// with names that sort first.
    fn path_precedes(&self, a: &[CategoryID], b: &[CategoryID]) -> bool {
        let key = |path: &[CategoryID]| -> Vec<(String, i64)> {
            path.iter()
                .map(|id| {
                    let name = self.get(*id).map(|body| body.name.to_lowercase());
                    (name.unwrap_or_default(), id.0)
                })
                .collect()
        };
        b.len()
            .cmp(&a.len())
            .then_with(|| key(a).cmp(&key(b)))
            .is_lt()
    }

    pub fn valid_relations(&self, category: CategoryID) -> HashSet<CategoryID> {
        self.graph
            .get_non_cyclic_additions(&category)
            .unwrap_or_default()
    }
    #[cfg(test)]
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
        fn an_item_satisfies_its_category_and_every_ancestor() {
            let (db, mut service) = service();
            let (spirit, botanical, gin, vodka) =
                (CategoryID(0), CategoryID(1), CategoryID(2), CategoryID(3));
            service.add_category_relation(&db, spirit, gin).unwrap();
            service.add_category_relation(&db, botanical, gin).unwrap();
            service.add_category_relation(&db, spirit, vodka).unwrap();
            service.add_item_mapping(&db, ItemID(5), gin).unwrap();

            assert_eq!(
                service.categories_satisfied_by(ItemID(5)),
                HashSet::from([spirit, botanical, gin])
            );
            assert!(service.categories_satisfied_by(ItemID(6)).is_empty());
        }

        /// Adds each `(parent, child)` relation, by id.
        fn relate(db: &TestDB, service: &mut CategoryService, edges: &[(i64, i64)]) {
            for &(parent, child) in edges {
                service
                    .add_category_relation(db, CategoryID(parent), CategoryID(child))
                    .unwrap();
            }
        }

        fn ids(path: &[i64]) -> Vec<CategoryID> {
            path.iter().copied().map(CategoryID).collect()
        }

        #[test]
        fn a_top_level_category_is_its_own_breadcrumb() {
            let (_, service) = service();
            let expected = Breadcrumb {
                path: ids(&[4]),
                other_paths: 0,
            };
            assert_eq!(service.breadcrumb(CategoryID(4)), Some(expected));
            assert_eq!(service.breadcrumb(CategoryID(999)), None);
        }

        #[test]
        fn breadcrumb_follows_the_only_path_down() {
            let (db, mut service) = service();
            relate(&db, &mut service, &[(0, 1), (1, 2)]);
            let expected = Breadcrumb {
                path: ids(&[0, 1, 2]),
                other_paths: 0,
            };
            assert_eq!(service.breadcrumb(CategoryID(2)), Some(expected));
        }

        #[test]
        fn breadcrumb_prefers_the_longest_path_and_counts_the_rest() {
            let (db, mut service) = service();
            // 5 -> 9 directly, and 3 -> 4 -> 9.
            relate(&db, &mut service, &[(5, 9), (3, 4), (4, 9)]);
            let expected = Breadcrumb {
                path: ids(&[3, 4, 9]),
                other_paths: 1,
            };
            assert_eq!(service.breadcrumb(CategoryID(9)), Some(expected));
        }

        #[test]
        fn breadcrumb_ties_go_to_the_alphabetically_first_path() {
            let (db, mut service) = service();
            // Both parents are top-level. Id 4 is "category 5" and id 9 is
            // "category 10", which sorts first.
            relate(&db, &mut service, &[(4, 20), (9, 20)]);
            let expected = Breadcrumb {
                path: ids(&[9, 20]),
                other_paths: 1,
            };
            assert_eq!(service.breadcrumb(CategoryID(20)), Some(expected));
        }

        #[test]
        fn breadcrumb_counts_every_path_through_a_diamond() {
            let (db, mut service) = service();
            // 0 -> {1, 2} -> 3 -> 4, plus 5 -> 4: three paths to 4.
            relate(
                &db,
                &mut service,
                &[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (5, 4)],
            );
            let expected = Breadcrumb {
                path: ids(&[0, 1, 3, 4]),
                other_paths: 2,
            };
            assert_eq!(service.breadcrumb(CategoryID(4)), Some(expected));
        }

        #[test]
        fn children_are_only_the_direct_ones() {
            let (db, mut service) = service();
            service
                .add_category_relation(&db, CategoryID(0), CategoryID(1))
                .unwrap();
            service
                .add_category_relation(&db, CategoryID(1), CategoryID(2))
                .unwrap();

            assert_eq!(service.children_by_name(CategoryID(0)), [CategoryID(1)]);
        }

        #[test]
        fn children_and_parents_are_listed_by_name() {
            let (db, mut service) = service();
            // Ids 0, 1, 4 and 9 are named "category 1", "category 2",
            // "category 5" and "category 10", so by name 0 < 9 < 1 < 4.
            relate(&db, &mut service, &[(0, 1), (0, 9), (0, 4), (1, 4), (9, 4)]);

            assert_eq!(service.children_by_name(CategoryID(0)), ids(&[9, 1, 4]));
            assert_eq!(service.parents_by_name(CategoryID(4)), ids(&[0, 9, 1]));
        }

        #[test]
        fn top_level_leaf_and_unknown_categories_have_no_relatives() {
            let (_, service) = service();
            assert_eq!(service.parents_by_name(CategoryID(0)), ids(&[]));
            assert_eq!(service.children_by_name(CategoryID(0)), ids(&[]));
            assert_eq!(service.parents_by_name(CategoryID(999)), ids(&[]));
            assert_eq!(service.children_by_name(CategoryID(999)), ids(&[]));
        }

        #[test]
        fn a_category_fills_itself_and_every_ancestor() {
            let (db, mut service) = service();
            relate(&db, &mut service, &[(0, 1), (1, 2), (3, 2), (2, 4)]);

            assert_eq!(
                service.self_and_ancestors(CategoryID(2)),
                ids(&[0, 1, 2, 3]).into_iter().collect()
            );
            assert!(service.self_and_ancestors(CategoryID(999)).is_empty());
        }

        #[test]
        fn item_counts_split_direct_items_from_subcategory_items() {
            let (db, mut service) = service();
            let (spirit, gin, old_tom) = (CategoryID(0), CategoryID(1), CategoryID(2));
            relate(&db, &mut service, &[(0, 1), (1, 2)]);
            service.add_item_mapping(&db, ItemID(5), spirit).unwrap();
            service.add_item_mapping(&db, ItemID(6), gin).unwrap();
            service.add_item_mapping(&db, ItemID(7), gin).unwrap();
            service.add_item_mapping(&db, ItemID(8), old_tom).unwrap();

            let counts = service.item_counts(spirit);
            assert_eq!(
                counts,
                ItemCounts {
                    direct: 1,
                    total: 4
                }
            );
            assert_eq!(counts.via_subcategories(), 3);
            assert_eq!(
                service.item_counts(gin),
                ItemCounts {
                    direct: 2,
                    total: 3
                }
            );
            assert_eq!(service.item_counts(CategoryID(999)), ItemCounts::default());
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

            assert_eq!(service.children_by_name(CategoryID(0)), []);
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
                        full_size: None,
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
            assert_eq!(service.children_by_name(gin), [vodka]);
        }

        #[test]
        fn get_all_returns_every_category() {
            let service = CategoryService::new(&TestDB::new()).unwrap();
            assert_eq!(service.get_all(CategoryFilter {}).len(), 30);
            assert_eq!(service.category_count(), 30);
        }

        #[test]
        fn names_are_looked_up_and_unknown_categories_have_none() {
            let service = CategoryService::new(&TestDB::new()).unwrap();
            assert_eq!(service.name_of(CategoryID(3)), "category 4");
            assert_eq!(service.name_of(CategoryID(999)), "");
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
