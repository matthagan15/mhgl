use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
};

/// The basic trait that "subsets" of nodes, which correspond to basis states
/// of our vector space, need to follow to be represented in hyperedges.
pub trait HgBasis: Sized + PartialEq + Eq + Hash + Clone {
    fn new_empty() -> Self;
    fn cardinality(&self) -> usize;
    fn intersect_with(&mut self, rhs: &Self);
    fn intersection(&self, rhs: &Self) -> Self;
    fn union_with(&mut self, rhs: &Self);
    fn union(&self, rhs: &Self) -> Self;
    fn matches_cardinality(&self, cardinality: usize) -> bool {
        self.cardinality() == cardinality
    }
    fn is_empty_set(&self) -> bool {
        self.matches_cardinality(0)
    }
    // The reason for using Self to represent a node is 
    // easy compatibility with BitNodes
    // TODO: Check if this should be a node?
    fn add_node(&mut self, node: &Self) {
        self.union_with(node);
    }
    fn add_nodes(&mut self, nodes: &HashSet<Self>) {
        for node in nodes.iter() {
            self.union_with(node);
        }
    }
    fn remove_node(&mut self, node: &Self);
}
