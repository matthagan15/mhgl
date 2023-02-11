use std::ops::{Add, Index, Mul};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{EdgeID, EdgeWeight};
use crate::structs::nodes::BitNodes;

/// A smaller HyperEdge implementation that uses bits to encode if a node is present or not
/// holdup: isn't this awful for sparse hypergraphs? now we need a bit for any node present,
/// which quickly blows up. however, an edge can now be stored with only n bits (n is number of nodes)
/// so this is efficient for dense hypergraphs.
///
/// ## Standards
/// the first node in the graph is the first bit in the array, so inputs[0][0], the last node is the
/// furthest from here.
pub struct BitEdge<const K: usize> {
    id: EdgeID,
    weight: EdgeWeight,
    inputs: BitNodes<K>,
    outputs: BitNodes<K>,
}

impl<const K: usize> BitEdge<K> {
    pub fn new() -> BitEdge<K> {
        BitEdge {
            id: Uuid::new_v4(),
            weight: 0.0,
            inputs: BitNodes::<K>::new(),
            outputs: BitNodes::<K>::new(),
        }
    }

    /// If you already have the data this tags it
    pub fn from(
        weight: EdgeWeight,
        input_nodes: BitNodes<K>,
        output_nodes: BitNodes<K>,
    ) -> BitEdge<K> {
        BitEdge {
            id: Uuid::new_v4(),
            weight: weight,
            inputs: input_nodes,
            outputs: output_nodes,
        }
    }

    pub fn map_basis(&self, input_basis: BitNodes<K>) -> BitNodes<K> {
        if self.inputs == input_basis {
            self.outputs.clone()
        } else {
            // TODO: THIS IS WRONG! Currently maps a non-matching input to the
            // empty set, which is NOT the same as the zero vector. need to
            // return a bit vector as opposed to a BitNodes.
            BitNodes::<K>::new()
        }
    }

    pub fn matches_input(&self, start_nodes: BitNodes<K>) -> bool {
        self.inputs == start_nodes
    }
    pub fn matches_output(&self, end_nodes: BitNodes<K>) -> bool {
        self.outputs == end_nodes
    }
}
