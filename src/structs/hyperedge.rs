use core::hash::Hash;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::Index,
};
use uuid::Uuid;

use super::hypergraph::Neighborhood;
use crate::structs::node_vec::HgVector;

pub type EdgeWeight = f64;
type NodeID = Uuid;
type EdgeID = Uuid;
static EDGE_WEIGHT_DEFAULT: EdgeWeight = 1.0;

// TODO: add logging

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum EdgeDirection {
    Directed,
    Oriented,
    Undirected,
    Loop,
    Blob,
}

/// # HyperEdge
/// Basic unit to map a node vector to another node vector in a hypergraph. The absolute most basic
/// type of hyperedge maps a single set of nodes to another set of nodes (potentially the same) with
/// some weight. Currently nodes have a set of potential input nodes and a set of output nodes
/// the variants are isolated into the direction parameter, currently
/// the direction is an enum of:
/// - Directed
/// - Oriented
/// - Undirected
/// - Loop
/// - Blob
/// Some potential directions that could be added:
/// - SuperBlob: connects any two disjoint subsets of nodes within the blob. ex: SuperBlob ({a,b,c}) could map
/// {a} -> {b} and {a} -> {b,c}. d
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HyperEdge {
    pub id: EdgeID,
    pub weight: EdgeWeight,
    pub in_nodes: HashSet<NodeID>,
    pub out_nodes: HashSet<NodeID>,
    pub direction: EdgeDirection,
}

impl HyperEdge {
    pub fn new() -> HyperEdge {
        HyperEdge {
            id: Uuid::new_v4(),
            weight: EDGE_WEIGHT_DEFAULT,
            in_nodes: HashSet::new(),
            out_nodes: HashSet::new(),
            direction: EdgeDirection::Directed,
        }
    }

    /// Create an edge from the provided in_nodes and out_nodes and edge type. Only preferential behavior is
    /// loops and blobs are created from the union of the two provided sets. The reasoning behind this for loop
    /// is that the output nodes should be empty or equal to the input nodes. For a blob node the idea is that
    /// it maps from all subsets of a blob to it's complement in the blob, so we want to take the union as the
    /// "blob" that it can map.
    ///
    /// ### blob
    /// blobs probably shouldn't exist, can view a "traditional" blob as a map from empty set to the blob?
    pub fn from(
        in_nodes: HashSet<NodeID>,
        out_nodes: HashSet<NodeID>,
        edge_type: EdgeDirection,
    ) -> HyperEdge {
        match edge_type {
            EdgeDirection::Directed => HyperEdge {
                id: Uuid::new_v4(),
                weight: EDGE_WEIGHT_DEFAULT,
                in_nodes: in_nodes,
                out_nodes: out_nodes,
                direction: edge_type,
            },
            EdgeDirection::Undirected => HyperEdge {
                id: Uuid::new_v4(),
                weight: EDGE_WEIGHT_DEFAULT,
                in_nodes: in_nodes.clone(),
                out_nodes: out_nodes.clone(),
                direction: edge_type,
            },
            EdgeDirection::Oriented => HyperEdge {
                id: Uuid::new_v4(),
                weight: EDGE_WEIGHT_DEFAULT,
                in_nodes: in_nodes.clone(),
                out_nodes: out_nodes.clone(),
                direction: edge_type,
            },
            EdgeDirection::Loop => {
                let u: HashSet<NodeID> = in_nodes.union(&out_nodes).map(|x| x.clone()).collect();
                HyperEdge {
                    id: Uuid::new_v4(),
                    weight: EDGE_WEIGHT_DEFAULT,
                    in_nodes: u,
                    out_nodes: HashSet::new(),
                    direction: edge_type,
                }
            }
            EdgeDirection::Blob => {
                let u: HashSet<NodeID> = in_nodes.union(&out_nodes).map(|x| x.clone()).collect();
                HyperEdge {
                    id: Uuid::new_v4(),
                    weight: EDGE_WEIGHT_DEFAULT,
                    in_nodes: u,
                    out_nodes: HashSet::new(),
                    direction: edge_type,
                }
            }
        }
    }

    pub fn add_input_node(&mut self, node: NodeID) {
        self.in_nodes.insert(node);
    }

    pub fn input_dim(&self) -> usize {
        self.in_nodes.len()
    }
    pub fn output_dim(&self) -> usize {
        self.out_nodes.len()
    }
    pub fn add_output_node(&mut self, node: NodeID) {
        self.out_nodes.insert(node);
    }

    pub fn remove_input_node(&mut self, node: &NodeID) {
        self.in_nodes.remove(node);
    }
    pub fn remove_output_node(&mut self, node: &NodeID) {
        self.out_nodes.remove(node);
    }

    pub fn flip_to_and_from(&mut self) {
        let tmp_from = self.in_nodes.clone();
        self.in_nodes = self.out_nodes.clone();
        self.out_nodes = tmp_from;
    }
    pub fn clone_in_nodes(&self) -> HashSet<NodeID> {
        self.in_nodes.clone()
    }
    pub fn clone_id(&self) -> EdgeID {
        self.id.clone()
    }

    pub fn map_basis(&self, b: &HashSet<NodeID>) -> HgVector {
        let mut ret = HgVector::new();
        match self.direction {
            EdgeDirection::Directed => {
                if self.matches_input(b) {
                    let tmp = HgVector::from_basis(self.out_nodes.clone(), self.weight);
                    ret.add(&tmp);
                }
            }
            EdgeDirection::Undirected => {
                if self.matches_input(b) {
                    let tmp = HgVector::from_basis(self.out_nodes.clone(), self.weight);
                    ret.add(&tmp);
                } else if self.matches_output(b) {
                    let tmp = HgVector::from_basis(self.in_nodes.clone(), self.weight);
                    ret.add(&tmp);
                }
            }
            EdgeDirection::Oriented => {
                if self.matches_input(b) {
                    let tmp = HgVector::from_basis(self.out_nodes.clone(), self.weight);
                    ret.add(&tmp);
                } else if self.matches_output(b) {
                    let tmp = HgVector::from_basis(self.in_nodes.clone(), -1. * self.weight);
                    ret.add(&tmp);
                }
            }
            EdgeDirection::Loop => {
                if self.matches_input(b) {
                    let tmp = HgVector::from_basis(self.in_nodes.clone(), self.weight);
                    ret.add(&tmp);
                }
            }
            EdgeDirection::Blob => {
                if b.is_subset(&self.in_nodes) {
                    let mut destination = HashSet::with_capacity(self.in_nodes.len() - b.len());
                    for id in self.in_nodes.iter() {
                        if b.contains(id) == false {
                            destination.insert(id.clone());
                        }
                    }
                    let tmp = HgVector::from_basis(destination, self.weight);
                    ret.add(&tmp);
                }
            }
        }
        ret
    }

    /// Map an input vector to an output vector.
    pub fn map_vec(&self, input_vec: &HgVector) -> HgVector {
        let mut ret = HgVector::new();
        for (basis, weight) in input_vec.basis() {
            let mut y = self.map_basis(&basis.into_iter().collect());
            y.multiply_scalar(weight);
            ret.add(&y);
        }
        ret
    }

    /// True if the union of in_nodes and out_nodes covers the provided set
    pub fn covers_set(&self, node_set: &HashSet<NodeID>) -> bool {
        let h: HashSet<_> = self
            .in_nodes
            .union(&self.out_nodes)
            .map(|x| x.clone())
            .collect();
        h.is_superset(node_set)
    }
    /// Returns a set with all nodes present in the edge (the union of input and output)
    pub fn total_nodes(&self) -> HashSet<NodeID> {
        self.in_nodes
            .union(&self.out_nodes)
            .map(|x| x.clone())
            .collect()
    }
    /// True if this hyperedge is contained in the provided set, False otherwise.
    pub fn is_covered_by(&self, cover: &HashSet<NodeID>) -> bool {
        let h: HashSet<_> = self
            .in_nodes
            .union(&self.out_nodes)
            .map(|x| x.clone())
            .collect();
        h.is_subset(cover)
    }
    /// True if provided set is same as the in_nodes of this edge.
    pub fn matches_input(&self, node_set: &HashSet<NodeID>) -> bool {
        self.in_nodes.is_subset(node_set) && self.in_nodes.is_superset(node_set)
    }

    /// True if provided set is same sa the out_nodes of this edge
    pub fn matches_output(&self, node_set: &HashSet<NodeID>) -> bool {
        self.out_nodes.is_subset(node_set) && self.out_nodes.is_superset(node_set)
    }
}

impl ToString for HyperEdge {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Hash for HyperEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

mod tests {
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    #[test]
    fn test_complement() {
        // let mut he: HyperEdge<(), i32> = HyperEdge::new(());
        // let node_vec = nodes!(1,2,3,4);
        // he.add_neighbors(node_vec)
    }

    #[test]
    fn test_boundary() {}
}
