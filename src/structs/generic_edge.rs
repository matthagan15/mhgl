use std::{collections::HashSet, hash::Hash};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::HgBasis;

use super::{generic_vec::GeneroVector, EdgeID, EdgeWeight};

/// Simple enum to denote which direction an edge faces
#[derive(Debug, Deserialize, Serialize, Hash, Clone, Copy, PartialEq)]
pub enum EdgeDirection {
    /// A set A that maps to a set B
    Directed,

    /// Set of nodes that map to another set and vice versa
    Symmetric,
    /// Set of nodes that map to themselves
    Loop,
    /// A set of nodes
    Undirected,

    Simplicial,
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
/// - Undirected: The traditional (in the literature) undirected hyperedge consisting of just a subset of nodes. As far as it's action, we currently think of Undirecteds as mapping a subset of it's basis element to the complement within the subset. For example, a Undirected of {a, b, c} would map {a} -> {b, c}, {a, b} -> {c}, {} -> {a, b, c}, etc.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct GeneroEdge<B: HgBasis> {
    pub in_nodes: B,
    pub out_nodes: B,
    pub direction: EdgeDirection,
}

impl<B: HgBasis> From<B> for GeneroEdge<B> {
    fn from(value: B) -> Self {
        Self {
            in_nodes: value,
            out_nodes: B::new_empty(),
            direction: EdgeDirection::Undirected,
        }
    }
}

impl<B: HgBasis> From<(B, B)> for GeneroEdge<B> {
    fn from(value: (B, B)) -> Self {
        Self {
            in_nodes: value.0,
            out_nodes: value.1,
            direction: EdgeDirection::Directed,
        }
    }
}

impl<B: HgBasis> GeneroEdge<B> {
    /// Creates a new Directed edge with weight 1. from
    /// the empty set to the empty set
    pub fn new() -> Self {
        GeneroEdge {
            in_nodes: B::new_empty(),
            out_nodes: B::new_empty(),
            direction: EdgeDirection::Directed,
        }
    }

    /// Creates an edge from the specified parts.
    pub fn from(in_nodes: B, out_nodes: B, weight: EdgeWeight, edge_type: EdgeDirection) -> Self {
        // TODO: This currently trusts the user way too much, what if we give the same nodes for in and out but specify the direction as Undirected? Need to do some basic checks first.
        GeneroEdge {
            in_nodes,
            out_nodes,
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
        if self.direction != EdgeDirection::Undirected || self.direction != EdgeDirection::Loop {
            self.out_nodes = new_output;
        }
    }

    /// For Undirecteds this returns the size of the Undirected
    pub fn input_cardinality(&self) -> usize {
        self.in_nodes.len()
    }

    /// For Undirecteds this returns the size of the Undirected.
    pub fn output_cardinality(&self) -> usize {
        if self.direction == EdgeDirection::Undirected || self.direction == EdgeDirection::Loop {
            self.in_nodes.len()
        } else {
            self.out_nodes.len()
        }
    }

    pub fn change_in_and_out(&mut self) {
        let tmp = self.in_nodes.clone();
        self.in_nodes = self.out_nodes.clone();
        self.out_nodes = tmp;
    }

    /// Allows you to change edge type. We make the following decisions:
    /// - If you change to a "directed" variant, then we will make sure that if
    /// the edge was previously a loop or Undirected it maps to the empty set, if the
    /// edge was previously Directed, Oriented, or Symmetric then we keep input and output
    /// as it was.
    /// - If you change to an Undirected edge then we take the union of the input and output
    /// and set that to be the Undirected.
    /// - If you change to a loop then we simply drop the output.
    pub fn change_direction(&mut self, new_direction: EdgeDirection) {
        match new_direction {
            EdgeDirection::Directed | EdgeDirection::Symmetric => {
                if self.direction == EdgeDirection::Undirected
                    || self.direction == EdgeDirection::Loop
                {
                    self.out_nodes = B::new_empty();
                }
                self.direction = new_direction;
            }
            EdgeDirection::Undirected | EdgeDirection::Simplicial => {
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

    pub fn node_vec(&self) -> Vec<B> {
        let tot = self.in_nodes.union(&self.out_nodes);
        tot.nodes().into_iter().collect()
    }

    /// Returns true if the provided basis matches an input of the
    /// edge (Undirected, Oriented, and Symmetric edges have more than
    /// one possible input)
    pub fn can_map_basis(&self, basis: &B) -> bool {
        match self.direction {
            EdgeDirection::Directed | EdgeDirection::Loop => self.in_nodes == *basis,
            EdgeDirection::Symmetric => {
                self.in_nodes == *basis || self.out_nodes == *basis
            }
            EdgeDirection::Undirected | EdgeDirection::Simplicial => self.in_nodes.intersection(basis) == *basis,
        }
    }
    /// Returns true if the provided basis is an output of this edge.
    pub fn matches_output(&self, basis: &B) -> bool {
        match self.direction {
            EdgeDirection::Directed | EdgeDirection::Loop => self.out_nodes == *basis,
            EdgeDirection::Symmetric => {
                self.in_nodes == *basis || self.out_nodes == *basis
            }
            EdgeDirection::Undirected | EdgeDirection::Simplicial => self.in_nodes.intersection(basis) == *basis,
        }
    }

    /// Returns true if this edge maps the input set to the output set
    pub fn is_correctly_mapped(&self, input: &B, output: &B) -> bool {
        match self.direction {
            EdgeDirection::Directed => self.in_nodes == *input && self.out_nodes == *output,
            EdgeDirection::Loop => self.in_nodes == *input && *output == *input,
            EdgeDirection::Symmetric => {
                let og_dir = self.in_nodes == *input && self.out_nodes == *output;
                let opposite_dir = self.in_nodes == *output && self.out_nodes == *input;
                og_dir || opposite_dir
            }
            EdgeDirection::Undirected | EdgeDirection::Simplicial => self.in_nodes.complement(input) == *output,
        }
    }

    /// Returns true if the edge is a Undirected consisting of the given
    /// input basis, false otherwise
    pub fn matches_undirected(&self, basis: &B) -> bool {
        self.direction == EdgeDirection::Undirected && self.in_nodes == *basis
    }

    /// Returns true if the provided set is contained in the union
    /// of the input and output of the edge. For undirected edges this
    /// means the given basis is a subset of `self`.
    pub fn contains(&self, basis: &B) -> bool {
        let total = self.in_nodes.union(&self.out_nodes);
        total.covers_basis(basis)
    }

    /// Maps the input set to it's output set with it's weight
    pub fn map(&self, input: &B) -> Option<(B, EdgeWeight)> {
        let v = GeneroVector::from_basis(input.clone(), 1.);
        let out = self.map_vector(v);
        let mut tups = out.to_tuples();
        if tups.len() == 1 {
            Some(tups.pop().unwrap())
        } else {
            None
        }
    }

    /// Maps an input set to a vector of the output set
    /// specified by the edge. Returns 0 vector if it cannot
    /// be mapped.
    pub fn map_to_vector(&self, basis: &B) -> GeneroVector<B> {
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
                    v.add_basis(self.out_nodes.clone(), w);
                }
            }
            EdgeDirection::Symmetric => {
                let input_basis_w = v.remove_basis(&self.in_nodes);
                let output_basis_w = v.remove_basis(&self.out_nodes);
                if input_basis_w != 0. {
                    v.add_basis(self.out_nodes.clone(), input_basis_w);
                }
                if output_basis_w != 0. {
                    v.add_basis(self.in_nodes.clone(), output_basis_w);
                }
            }
            EdgeDirection::Loop => {}
            EdgeDirection::Undirected | EdgeDirection::Simplicial => {
                let mut ret = GeneroVector::new();
                for (b, w) in v.basis_to_weight.drain() {
                    if self.in_nodes.covers_basis(&b) {
                        ret.add_basis(self.in_nodes.complement(&b), w);
                    }
                }
                v = ret;
            }
        }
        v
    }
}

mod test {
    use std::collections::HashSet;

    use crate::{structs::{GeneroEdge, GeneroVector}, EdgeDirection, SparseNodeSet};

    #[test]
    fn test_sparse_map_vec() {
        let _nodes: Vec<u16> = vec![11, 23, 492, 493, 203];
        let b1 = SparseNodeSet::from_slice(&[11_u16, 23, 492, 493]);
        let b2 = SparseNodeSet::from_slice(&[11_u16, 23, 492, 493, 203]);
        let b3 = SparseNodeSet::<u16>::new();
        let mut e = GeneroEdge::<SparseNodeSet<u16>>::new();
        e.change_direction(EdgeDirection::Undirected);
        e.add_input_nodes(&b2);
        println!("e: {:?}", e);
        let mut v = GeneroVector::<SparseNodeSet<u16>>::new();
        v.add_basis(b1.clone(), 2.);
        v.add_basis(b3.clone(), 3.);
        println!("input vector: {:#?}", v);
        let out = e.map_vector(v);
        println!("output vector: {:#?}", out);
    }
    
    fn basic_edge() -> GeneroEdge<SparseNodeSet<u32>> {
        let b = SparseNodeSet::from_slice(&[1, 2, 3_u32]);
        let e = GeneroEdge::from(b, SparseNodeSet::new(), 1., EdgeDirection::Undirected);
        e
    }

    #[test]
    fn test_serialization() {
        let e = basic_edge();
        let s = serde_json::to_string(&e).expect("could not serialize edge");
        dbg!(s);
    }
}
