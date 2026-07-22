//! mod graph;
//! Data structure for understanding and maintaining the relationships between categories.
//!

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub(super) struct DirectedAcyclicGraph<T: Copy + Eq + Hash> {
    graph: HashMap<T, HashSet<T>>,
}
#[derive(Debug)]
pub enum GraphError {
    EdgeEndpointNotInGraph,
    WouldIntroduceCycle,
}

impl<T: Copy + Eq + Hash> DirectedAcyclicGraph<T> {
    pub fn new() -> Self {
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
    fn contains_vertex(&self, vertex: &T) -> bool {
        self.graph.contains_key(vertex)
    }
    pub fn insert_edge(&mut self, parent: &T, child: &T) -> Result<(), GraphError> {
        if !(self.contains_vertex(parent) && self.contains_vertex(child)) {
            return Err(GraphError::EdgeEndpointNotInGraph);
        }
        if self.is_parent_of(parent, child) {
            return Err(GraphError::WouldIntroduceCycle);
        }
        self.graph.get_mut(parent).unwrap().insert(*child);
        Ok(())
    }
    fn get_edges(&self, vertex: &T) -> &HashSet<T> {
        self.graph.get(vertex).unwrap()
    }
    pub fn remove(&mut self, vertex: T) {
        let Some(child_verticies) = self.graph.remove(&vertex) else {
            return;
        };
        for (_, adj_set) in self.graph.iter_mut() {
            if adj_set.remove(&vertex) {
                adj_set.extend(child_verticies.clone());
            }
        }
    }
    pub fn is_parent_of(&self, parent_vertex: &T, child_vertex: &T) -> bool {
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
        let mut children: HashSet<T> = self.get_edges(vertex).clone();
        for child_vertex in &children {
            stack.push(*child_vertex);
        }
        while let Some(current_vertex) = stack.pop() {
            for child_vertex in self.get_edges(&current_vertex) {
                if children.insert(*child_vertex) {
                    stack.push(*child_vertex);
                }
            }
        }
        Some(children)
    }
    pub fn get_non_cyclic(&self, search_vertex: &T) -> Option<HashSet<T>> {
        if !self.contains_vertex(search_vertex) {
            return None;
        }
        //This feels like an awful idea, will investigate for better
        let non_cyclic: HashSet<T> =
            self.graph
                .keys()
                .fold(HashSet::new(), |mut acc, graph_vertex: &T| {
                    if !self.is_parent_of(graph_vertex, search_vertex)
                        && !self
                            .graph
                            .get(graph_vertex)
                            .unwrap()
                            .contains(search_vertex)
                    {
                        acc.insert(*graph_vertex);
                    }
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
            let graph = (1..=3).fold(HashMap::new(), |mut acc, i| {
                let set = (1..i - 1).fold(HashSet::new(), |mut set_acc, x| {
                    set_acc.insert(x);
                    set_acc
                });
                acc.insert(i, set);
                acc
            });
            DirectedAcyclicGraph::load(graph)
        }
        #[test]
        fn removing_results_in_lower_members_being_moved() {
            let mut graph = get_graph();
            eprintln!("{:?}", graph);

            graph.remove(2);

            assert!(!graph.contains_vertex(&2));
            assert_eq!(*graph.get_edges(&3), HashSet::from([1]));
        }
        #[test]
        fn cycles_not_allowed_at_insertion() {
            let mut graph = get_graph();

            let insert_result = graph.insert_edge(&3, &1);

            assert!(insert_result.is_err())
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
            let mut graph = DirectedAcyclicGraph::new();
            for i in 1..=10 {
                graph.insert_vertex(i);
                for divisor in (1..i).filter(|x| i % x == 0) {
                    graph.insert_edge(&i, &divisor).unwrap();
                }
            }
            graph
        }
        #[test]
        fn returns_all_valid_connections() {
            let graph = get_graph();
            eprintln!("{:?}", graph.get_non_cyclic(&10));
            assert!(false);
        }
    }
}
