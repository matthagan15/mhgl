use std::{collections::HashSet, hash::Hash};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::HgBasis;

use super::{generic_vec::GeneroVector, EdgeID, EdgeWeight};

/// Simple enum to denote which direction an edge face
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum EdgeDirection {
    Directed,
    Oriented,
    Undirected,
    Loop,
    Blob,
}

/// # Edge
/// HyperEdges constitute the main objects of a HyperGraph. The fundamental type of edge is
/// a directed, weighted edge which maps an input subset of nodes to an output subsets of nodes
/// with an associated number. However we provide several other edge variants:
/// - Directed: Maps the input basis element to the output basis element with the specified weight.
/// - Undirected: Maps two basis elements to each other with the same weight,
/// so an undirected edge from A = {n_1, n_2, n_3} <-> B = {n_4, n_5} will map A to B and B to A with the same weight.
/// - Oriented: Similar to Undirected but will flip the sign of the edge weight if it is being traversed opposite of the orientation. Ex: A -> B with weight +2.5 but B -> A with weight -2.5.
/// - Loop: Maps a basis element to itself.
/// - Blob: The traditional (in the literature) undirected hyperedge consisting of just a subset of nodes. As far as it's action, we currently think of Blobs as mapping a subset of it's basis element to the complement within the subset. For example, a blob of {a, b, c} would map {a} -> {b, c}, {a, b} -> {c}, {} -> {a, b, c}, etc. 
#[derive(Debug, Clone)]
pub struct GeneroEdge<B: HgBasis> {
    pub id: EdgeID,
    pub weight: EdgeWeight,
    pub in_nodes: B,
    pub out_nodes: B,
    pub direction: EdgeDirection,
}

impl<B: HgBasis> GeneroEdge<B> {
    pub fn new() -> Self {
        GeneroEdge {
            id: Uuid::new_v4(),
            weight: 1.,
            in_nodes: B::new_empty(),
            out_nodes: B::new_empty(),
            direction: EdgeDirection::Directed,
        }
    }

    // TODO: This currently trusts the user way too much, what if we give the same nodes for in and out but specify the direction as Blob? Need to do some basic checks first.
    pub fn from(in_nodes: B, out_nodes: B, weight: EdgeWeight, edge_type: EdgeDirection) -> Self {
        GeneroEdge {
            id: Uuid::new_v4(),
            weight: weight,
            in_nodes: in_nodes,
            out_nodes: out_nodes,
            direction: edge_type,
        }
    }

    pub fn add_input_nodes(&mut self, node: &B) {
        self.in_nodes.add_node(node);
    }

    pub fn remove_input_node(&mut self, node: &B) {
        self.in_nodes.remove_node(node);
    }
    pub fn add_output_nodes(&mut self, node: &B) {
        self.out_nodes.add_node(node);
    }

    pub fn remove_output_node(&mut self, node: &B) {
        self.out_nodes.remove_node(node);
    }

    pub fn remove_node(&mut self, node: &B) {
        self.in_nodes.remove_node(node);
        self.out_nodes.remove_node(node);
    }

    pub fn change_input(&mut self, new_input: B) {
        self.in_nodes = new_input;
    }

    pub fn change_output(&mut self, new_output: B) {
        if self.direction != EdgeDirection::Blob || self.direction != EdgeDirection::Loop {
            self.out_nodes = new_output;
        }
    }

    /// For blobs this returns the size of the blob
    pub fn input_cardinality(&self) -> usize {
        self.in_nodes.cardinality()
    }

    /// For blobs this returns the size of the blob.
    pub fn output_cardinality(&self) -> usize {
        if self.direction == EdgeDirection::Blob || self.direction == EdgeDirection::Loop {
            self.in_nodes.cardinality()
        } else {
            self.out_nodes.cardinality()
        }
    }

    pub fn change_weight(&mut self, new_weight: EdgeWeight) {
        // NaN check is done at graph level, assuming edges should
        // not be publicly accessible.
        self.weight = new_weight
    }

    pub fn flip_to_and_from(&mut self) {
        let tmp = self.in_nodes.clone();
        self.in_nodes = self.out_nodes.clone();
        self.out_nodes = tmp;
    }

    /// Allows you to change edge type. We make the following decisions:
    /// - If you change to a "directed" variant, then we will make sure that if
    /// the edge was previously a loop or blob it maps to the empty set, if the
    /// edge was previously a "directed" variant then we keep input and output
    /// as it was.
    /// - If you change to a blob then we take the union of the input and output
    /// and set that to be the blob.
    /// - If you change to a loop then we simply drop the output.
    pub fn change_direction(&mut self, new_direction: EdgeDirection) {
        match new_direction {
            EdgeDirection::Directed | EdgeDirection::Oriented | EdgeDirection::Undirected => {
                if self.direction == EdgeDirection::Blob || self.direction == EdgeDirection::Loop {
                    self.out_nodes = B::new_empty();
                }
                self.direction = new_direction;
            }
            EdgeDirection::Blob => {
                let u = self.in_nodes.union(&self.out_nodes);
                self.out_nodes = B::new_empty();
                self.in_nodes = u;
                self.direction = new_direction;
            }
            EdgeDirection::Loop => {
                self.out_nodes = B::new_empty();
                self.direction = new_direction
            }
        }
    }

    pub fn clone_input_nodes(&self) -> B {
        self.in_nodes.clone()
    }
    pub fn clone_output_nodes(&self) -> B {
        self.out_nodes.clone()
    }

    pub fn nodes(&self) -> HashSet<B> {
        let tot = self.in_nodes.union(&self.out_nodes);
        tot.nodes()
    }

    // TODO: need to handle undirected, oriented, and blob edges
    pub fn can_map_basis(&self, basis: &B) -> bool {
        match self.direction {
            EdgeDirection::Directed | EdgeDirection::Loop => self.in_nodes == *basis,
            EdgeDirection::Oriented | EdgeDirection::Undirected => {
                self.in_nodes == *basis || self.out_nodes == *basis
            }
            EdgeDirection::Blob => {
                // TODO: add a "contains nodes" function to avoid this stuff
                self.in_nodes.intersection(basis) == *basis
            }
        }
    }
    pub fn matches_output(&self, basis: &B) -> bool {
        match self.direction {
            EdgeDirection::Directed | EdgeDirection::Loop => self.out_nodes == *basis,
            EdgeDirection::Oriented | EdgeDirection::Undirected => {
                self.in_nodes == *basis || self.out_nodes == *basis
            }
            EdgeDirection::Blob => {
                // TODO: add a "contains nodes" function to avoid this stuff
                self.in_nodes.intersection(basis) == *basis
            }
        }
    }

    /// Returns true if the edge is a blob consisting of the given
    /// input basis, false otherwise
    pub fn matches_blob(&self, basis: &B) -> bool {
        self.direction == EdgeDirection::Blob && self.in_nodes == *basis
    }

    /// Returns true if output and input unioned cover the provided basis
    pub fn contains(&self, basis: &B) -> bool {
        let total = self.in_nodes.union(&self.out_nodes);
        total.covers_basis(basis)
    }

    pub fn map(&self, basis: &B) -> GeneroVector<B> {
        if self.can_map_basis(basis) == false {
            return GeneroVector::new();
        }
        let new_vec = GeneroVector::from_basis(basis.clone(), 1.);
        self.map_vector(new_vec)
    }

    pub fn map_vector(&self, mut v: GeneroVector<B>) -> GeneroVector<B> {
        match self.direction {
            EdgeDirection::Directed => {
                let w = v.remove_basis(&self.in_nodes);
                if w != 0. {
                    v.add_basis(self.out_nodes.clone(), w * self.weight);
                }
            }
            EdgeDirection::Oriented => {
                let input_basis_w = v.remove_basis(&self.in_nodes);
                let output_basis_w = v.remove_basis(&self.out_nodes);
                if input_basis_w != 0. {
                    v.add_basis(self.out_nodes.clone(), input_basis_w * self.weight);
                }
                if output_basis_w != 0. {
                    v.add_basis(self.in_nodes.clone(), -1. * output_basis_w * self.weight);
                }
            }
            EdgeDirection::Undirected => {
                let input_basis_w = v.remove_basis(&self.in_nodes);
                let output_basis_w = v.remove_basis(&self.out_nodes);
                if input_basis_w != 0. {
                    v.add_basis(self.out_nodes.clone(), input_basis_w * self.weight);
                }
                if output_basis_w != 0. {
                    v.add_basis(self.in_nodes.clone(), output_basis_w * self.weight);
                }
            }
            EdgeDirection::Loop => {
                if v.basis_to_weight.contains_key(&self.in_nodes) {
                    let old_w = v
                        .basis_to_weight
                        .get_mut(&self.in_nodes)
                        .expect("just checked");
                    *old_w = *old_w * self.weight;
                }
            }
            EdgeDirection::Blob => {
                let mut ret = GeneroVector::new();
                for (b, w) in v.basis_to_weight.drain() {
                    if self.in_nodes.covers_basis(&b) {
                        ret.add_basis(self.in_nodes.complement(&b), w * self.weight);
                    }
                }
                v = ret;
            }
        }
        v
    }
}

impl<B: HgBasis> Hash for GeneroEdge<B> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

mod test {
    use std::collections::HashSet;

    use crate::{structs::{SparseBasis, GeneroEdge, GeneroVector}, EdgeDirection};

    #[test]
    fn test_sparse_map_vec() {
        let _nodes: Vec<u16> = vec![11, 23, 492, 493, 203];
        let b1 = SparseBasis::from(HashSet::from([11_u16, 23, 492, 493]));
        let b2 = SparseBasis::from(HashSet::from([11_u16, 23, 492, 493, 203]));
        let b3 = SparseBasis::<u16>::new();
        let mut e = GeneroEdge::<SparseBasis<u16>>::new();
        e.change_direction(EdgeDirection::Blob);
        e.add_input_nodes(&b2);
        println!("e: {:?}", e);
        let mut v = GeneroVector::<SparseBasis<u16>>::new();
        v.add_basis(b1.clone(), 2.);
        v.add_basis(b3.clone(), 3.);
        println!("input vector: {:#?}", v);
        let out = e.map_vector(v);
        println!("output vector: {:#?}", out);
    }
}
