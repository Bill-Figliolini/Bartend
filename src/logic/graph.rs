//! mod graph;
//! Data structure for understanding and maintaining the relationships between categories.
//!

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub(super) struct DirectedAcyclicGraph<T: Copy + Eq + Hash + PartialEq> {
    graph: HashMap<T, HashSet<T>>,
}
#[derive(Debug)]
pub struct GraphPatch<T: Copy + Eq + Hash + PartialEq> {
    pub to_remove: T,
    pub parents: Option<HashSet<T>>,
    pub children: Option<HashSet<T>>,
}

#[derive(Debug)]
pub enum GraphError {
    EdgeEndpointNotInGraph,
    WouldIntroduceCycle,
}

impl<T: std::fmt::Debug + Copy + Eq + Hash + PartialEq> DirectedAcyclicGraph<T> {
    pub fn _new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }
    pub fn load(graph: HashMap<T, HashSet<T>>) -> Self {
        Self { graph }
    }
    pub fn insert_vertex(&mut self, vertex: T) {
        self.graph.entry(vertex).or_insert_with(|| HashSet::new());
    }
    pub fn contains_vertex(&self, vertex: &T) -> bool {
        self.graph.contains_key(vertex)
    }
    pub fn insert_edge(&mut self, parent: &T, child: &T) -> Result<(), GraphError> {
        if !(self.contains_vertex(parent) && self.contains_vertex(child)) {
            return Err(GraphError::EdgeEndpointNotInGraph);
        }
        if self.is_parent_of(child, parent) {
            return Err(GraphError::WouldIntroduceCycle);
        }
        self.graph.get_mut(parent).unwrap().insert(*child);
        Ok(())
    }
    pub fn get_edges(&self, vertex: &T) -> Option<HashSet<T>> {
        self.graph.get(vertex).map(|set| set.clone())
    }
    //TODO: for DB changes, will need to split change detection and actual removal
    pub fn remove(&mut self, patch: GraphPatch<T>) {
        self.graph.remove(&patch.to_remove);
        if let Some(parents) = patch.parents {
            for parent in parents.iter() {
                self.graph.entry(*parent).and_modify(|children| {
                    children.remove(&patch.to_remove);
                });
            }
            if let Some(added_children) = patch.children {
                for parent in parents {
                    self.graph.entry(parent).and_modify(|children| {
                        children.extend(added_children.clone());
                    });
                }
            }
        }
    }
    pub fn get_removal_patch(&self, vertex: &T) -> GraphPatch<T> {
        let patch = GraphPatch {
            to_remove: *vertex,
            parents: self.get_immediate_ancestors(vertex),
            children: self.get_edges(vertex),
        };
        eprintln!("{patch:?}");
        patch
    }
    pub fn remove_edge(&mut self, parent: &T, child: &T) {
        let Some(mut child_set) = self.get_edges(parent) else {
            return;
        };
        child_set.remove(child);
        let slot = self.graph.get_mut(parent).expect("Already got set");
        *slot = child_set;
    }

    pub fn is_parent_of(&self, parent_vertex: &T, child_vertex: &T) -> bool {
        if parent_vertex == child_vertex {
            return true;
        }
        let current_set = self.graph.get(parent_vertex);
        let next_set = match current_set {
            Some(next_set) => next_set,
            None => return false,
        };
        if next_set.contains(child_vertex) {
            return true;
        }
        for vertex in next_set {
            if self.is_parent_of(vertex, child_vertex) {
                return true;
            }
        }
        false
    }
    pub fn get_all_children(&self, vertex: &T) -> Option<HashSet<T>> {
        if !self.contains_vertex(vertex) {
            return None;
        }
        let mut stack: Vec<T> = Vec::new();
        let mut children: HashSet<T> = self.get_edges(vertex).unwrap();
        for child_vertex in &children {
            stack.push(*child_vertex);
        }
        while let Some(current_vertex) = stack.pop() {
            for child_vertex in self.get_edges(&current_vertex).unwrap() {
                if children.insert(child_vertex) {
                    stack.push(child_vertex);
                }
            }
        }
        Some(children)
    }
    pub fn get_ancestors(&self, child: &T) -> Option<HashSet<T>> {
        if !self.contains_vertex(child) {
            return None;
        }
        let mut set_of_ancestors = HashSet::new();
        let mut to_process = Vec::from([*child]);
        while let Some(current_ancestor) = to_process.pop() {
            if let Some(next_ancestors) = self.get_immediate_ancestors(&current_ancestor) {
                let unvisited_next: HashSet<T> = next_ancestors
                    .into_iter()
                    .filter(|ancestor| !set_of_ancestors.contains(ancestor))
                    .collect();
                to_process.extend(unvisited_next);
            }
            set_of_ancestors.insert(current_ancestor);
        }

        Some(set_of_ancestors)
    }
    pub fn get_immediate_ancestors(&self, child: &T) -> Option<HashSet<T>> {
        if !self.contains_vertex(child) {
            return None;
        }
        let mut ancestors = HashSet::new();
        for (node, edges) in self.graph.iter() {
            if edges.contains(child) {
                ancestors.insert(*node);
            }
        }

        Some(ancestors)
    }
    pub fn get_non_cyclic_additions(&self, search_vertex: &T) -> Option<HashSet<T>> {
        if !self.contains_vertex(search_vertex) {
            return None;
        }
        let parents_of_search = self
            .get_ancestors(search_vertex)
            .expect("Search node should be in graph");
        let children_of_search = self
            .get_edges(search_vertex)
            .expect("Search node should be in graph");
        let non_cyclic = self
            .graph
            .keys()
            .filter(|vertex| {
                **vertex != *search_vertex
                    && !parents_of_search.contains(*vertex)
                    && !children_of_search.contains(*vertex)
            })
            .fold(HashSet::new(), |mut acc, graph_vertex: &T| {
                acc.insert(*graph_vertex);
                acc
            });

        Some(non_cyclic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod basic_behavior {
        use super::*;
        fn get_graph() -> DirectedAcyclicGraph<u32> {
            let mut graph = DirectedAcyclicGraph::_new();
            graph.insert_vertex(1);
            graph.insert_vertex(2);
            graph.insert_vertex(3);
            graph.insert_vertex(4);
            graph.insert_vertex(5);
            graph.insert_edge(&1, &2).unwrap();
            graph.insert_edge(&2, &3).unwrap();
            graph.insert_edge(&1, &4).unwrap();
            graph.insert_edge(&4, &5).unwrap();

            graph
        }
        #[test]
        fn cycles_not_allowed_at_insertion() {
            let mut graph = get_graph();

            let insert_result = graph.insert_edge(&3, &1);

            assert!(insert_result.is_err())
        }
        #[test]
        fn self_loops_not_allowed() {
            let mut graph = get_graph();

            let insert_result = graph.insert_edge(&1, &1);

            assert!(insert_result.is_err())
        }
        #[test]
        fn remove_edge_splits_tree() {
            let mut graph = get_graph();

            graph.remove_edge(&2, &3);

            assert!(!graph.graph.get(&1).unwrap().contains(&3));
            assert!(!graph.graph.get(&2).unwrap().contains(&3));
        }
        #[test]
        fn get_ancestors_returns_none_if_not_in_graph() {
            let graph = get_graph();

            let result = graph.get_ancestors(&25);

            assert!(result.is_none())
        }
    }
    mod removal_relation_maintenance {
        use super::*;

        fn get_graph() -> DirectedAcyclicGraph<u32> {
            let mut graph = DirectedAcyclicGraph::_new();
            graph.insert_vertex(1);
            graph.insert_vertex(2);
            graph.insert_vertex(3);
            graph.insert_vertex(4);
            graph.insert_vertex(5);
            graph.insert_edge(&1, &2).unwrap();
            graph.insert_edge(&2, &3).unwrap();
            graph.insert_edge(&1, &4).unwrap();
            graph.insert_edge(&4, &5).unwrap();

            graph
        }
        #[test]
        fn removing_results_in_lower_members_being_moved() {
            let mut graph = get_graph();

            let patch = graph.get_removal_patch(&2);
            graph.remove(patch);

            assert!(!graph.contains_vertex(&2));
            assert_eq!(graph.get_edges(&1).unwrap(), HashSet::from([3, 4]));
        }
        #[test]
        fn removing_topmost_results_separate_trees() {
            let mut graph = get_graph();

            let patch = graph.get_removal_patch(&1);
            graph.remove(patch);

            assert!(!graph.contains_vertex(&1));
            assert_eq!(graph.get_all_children(&2).unwrap(), HashSet::from([3]));
            assert_eq!(graph.get_all_children(&4).unwrap(), HashSet::from([5]));
        }
        #[test]
        fn removing_bottomost_results_no_other_changes() {
            let mut graph = get_graph();

            let patch = graph.get_removal_patch(&5);
            graph.remove(patch);

            assert!(!graph.contains_vertex(&5));
            assert_eq!(
                graph.get_all_children(&1).unwrap(),
                HashSet::from([2, 3, 4])
            );
        }
    }
    mod get_all_children {
        use super::*;
        fn get_graph() -> DirectedAcyclicGraph<i32> {
            let graph = (1..=5).fold(HashMap::new(), |mut acc, i| {
                let mut new_set = HashSet::new();
                if i > 1 {
                    new_set.insert(i - 1);
                }
                acc.insert(i, new_set);
                acc
            });

            DirectedAcyclicGraph::load(graph)
        }
        #[test]
        fn results_in_indirect_child_nodes_returned() {
            let graph = get_graph();

            let implication = graph.get_all_children(&4);

            let expected_implication = Some(HashSet::from([1, 2, 3]));
            assert_eq!(implication, expected_implication);
        }
        #[test]
        fn results_in_none_if_not_in_graph() {
            let graph = get_graph();

            let implication = graph.get_all_children(&7);

            let expected_implication = None;
            assert_eq!(implication, expected_implication);
        }
    }
    mod get_non_cyclic {
        use super::*;
        fn get_graph() -> DirectedAcyclicGraph<u32> {
            let mut graph = DirectedAcyclicGraph::_new();
            for i in 1..=5 {
                graph.insert_vertex(i);
            }
            graph.insert_edge(&1, &2).unwrap();
            graph.insert_edge(&2, &4).unwrap();
            graph.insert_edge(&4, &3).unwrap();
            graph
        }
        #[test]
        fn returns_all_valid_connections() {
            let graph = get_graph();
            let mut actual_results = Vec::new();
            for i in 1..=5 {
                actual_results.push(graph.get_non_cyclic_additions(&i).unwrap());
            }
            let expected_results = vec![
                HashSet::from([3, 4, 5]),
                HashSet::from([3, 5]),
                HashSet::from([5]),
                HashSet::from([5]),
                HashSet::from([1, 2, 3, 4]),
            ];
            assert_eq!(actual_results, expected_results);
        }
        #[test]
        fn returns_none_if_not_in_graph() {
            let graph = get_graph();

            let result = graph.get_non_cyclic_additions(&123);

            assert!(result.is_none())
        }
    }
}
