use std::ops::{Add, Index, Mul};

use serde::{Deserialize, Serialize};

use super::EdgeWeight;
use crate::structs::nodes::BitNode;

/// A smaller HyperEdge implementation that uses bits to encode if a node is present or not
/// holdup: isn't this awful for sparse hypergraphs? now we need a bit for any node present,
/// which quickly blows up. however, an edge can now be stored with only n bits (n is number of nodes)
/// so this is efficient for dense hypergraphs.
///
/// ## Standards
/// the first node in the graph is the first bit in the array, so inputs[0][0], the last node is the
/// furthest from here.
pub struct BitEdge<const K1: usize, const K2: usize> {
    weight: EdgeWeight,
    inputs: BitNode<K1>,
    outputs: BitNode<K2>,
}

impl<const K1: usize, const K2: usize> BitEdge<K1, K2> {
    pub fn new() -> BitEdge<K1, K2> {
        BitEdge {
            weight: 0.0,
            inputs: BitNode::<K1>::new(),
            outputs: BitNode::<K2>::new(),
        }
    }

    /// Currently drops input_nodes if it has more than 1 bit flipped.
    pub fn from(
        input_nodes: BitNode<K1>,
        output_nodes: BitNode<K2>,
        weight: EdgeWeight,
    ) -> BitEdge<K1, K2> {
        BitEdge {
            weight: weight,
            inputs: input_nodes,
            outputs: output_nodes,
        }
    }

    pub fn matches_input<const k: usize>(&self, start_nodes: BitNode<k>) -> bool {
        let mut ret = true;
        if k == K1 {
            for ix in 0..k {
                if self.inputs.bits[k] != start_nodes.bits[k] {
                    ret = false;
                }
            }
        }
        ret
    }
}
