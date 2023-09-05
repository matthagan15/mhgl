use std::{collections::HashSet, hash::Hash};

use serde::{Serialize, Deserialize};

/// The basic trait that "subsets" of nodes, which correspond to basis states
/// of our vector space, need to follow to be represented in hyperedges.
pub trait HgBasis: PartialEq + Eq + Hash + Clone + Serialize {
    fn new_empty() -> Self;
    fn len(&self) -> usize;
    fn intersect_with(&mut self, rhs: &Self);
    fn intersection(&self, rhs: &Self) -> Self;
    fn union_with(&mut self, rhs: &Self);
    fn union(&self, rhs: &Self) -> Self;
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
    fn complement(&self, rhs: &Self) -> Self;
    fn covers_basis(&self, basis: &Self) -> bool {
        let intersection = self.intersection(basis);
        intersection == *basis
    }
    fn nodes(&self) -> HashSet<Self>;
}
