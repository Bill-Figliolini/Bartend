//! mod graph;
//! Data structure for understanding and maintaining the relationships between categories.
//!

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};
#[derive(Debug)]
struct Graph<T: Copy + Eq + Hash> {
    matrix: HashMap<T, HashSet<T>>,
}
#[derive(Debug)]
enum GraphError {
    EdgeEndpointNotInGraph,
    WouldIntroduceCycle,
}

impl<T: Copy + Eq + Hash> Graph<T> {
    pub fn new() -> Self {
        Self {
            matrix: HashMap::new(),
        }
    }
    pub fn build_from(vertices: &[T], edges: &[(T, T)]) -> Result<Self, GraphError> {
        let mut graph = Self::new();
        for vertex in vertices {
            graph.insert_vertex(*vertex);
        }
        for edge in edges {
            graph.insert_edge(edge)?;
        }
        Ok(graph)
    }
    pub fn insert_vertex(&mut self, vertex: T) {
        if !self.matrix.contains_key(&vertex) {
            self.matrix.insert(vertex, HashSet::new());
        }
    }
    fn contains_vertex(&self, vertex: &T) -> bool {
        self.matrix.contains_key(vertex)
    }
    pub fn insert_edge(&mut self, edge: &(T, T)) -> Result<(), GraphError> {
        if !(self.contains_vertex(&edge.0) && self.contains_vertex(&edge.1)) {
            return Err(GraphError::EdgeEndpointNotInGraph);
        }
        self.matrix.get_mut(&edge.0).unwrap().insert(edge.1);
        Ok(())
    }
    fn get_edges(&self, vertex: &T) -> &HashSet<T> {
        self.matrix.get(vertex).unwrap()
    }
    fn does_x_imply_y(&self, vertex_x: T, vertex_y: T) -> bool {
        self.matrix.get(&vertex_x).unwrap().contains(&vertex_y)
    }
    pub fn remove(&mut self, vertex: T) {
        if !self.contains_vertex(&vertex) {
            return;
        }
    }
    pub fn implies(&self, vertex: &T) -> HashSet<T> {
        let mut implications = HashSet::new();
        let mut stack = Vec::new();
        if !self.contains_vertex(vertex) {
            return implications;
        }
        for child_vertex in self.get_edges(vertex) {
            stack.push(*child_vertex);
            implications.insert(*child_vertex);
        }
        while let Some(current_vertex) = stack.pop() {
            for child_vertex in self.get_edges(&current_vertex) {
                stack.push(*child_vertex);
                implications.insert(*child_vertex);
            }
        }
        implications
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removing_results_in_lower_members_being_moved() {
        let mut graph = Graph::build_from(&[1, 2, 3], &[(1, 2), (2, 3)]).unwrap();

        graph.remove(2);

        assert!(!graph.contains_vertex(&2));
        assert_eq!(*graph.get_edges(&1), HashSet::from([3]));
    }
    #[test]
    fn implication_results_in_indirect_child_nodes_returned() {
        let graph = Graph::build_from(&[1, 2, 3, 4, 5], &[(1, 2), (2, 3), (3, 4), (4, 5)]).unwrap();

        let implication = graph.implies(&2);

        let expected_implication = HashSet::from([3, 4, 5]);
        assert_eq!(implication, expected_implication);
    }
}
