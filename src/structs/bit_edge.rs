use std::ops::{Add, Mul, Index};

use serde::{Deserialize, Serialize};

use super::EdgeWeight;

#[derive(PartialEq, Eq, Debug)]
pub struct BitNode<const K: usize> {
    bits: [u8; K],
}

impl<const K: usize> BitNode<K> {
    pub fn new() -> BitNode<K> {
        BitNode {bits: [0; K]}
    }

    pub fn from(bits: [u8; K]) -> BitNode<K> {
        BitNode {bits: bits}
    }

    pub fn is_node(&self) -> bool {
        let mut num_ones = 0;
        for ix in 0..K {
            num_ones += self.bits[ix].count_ones();
        }
        num_ones == 1
    }

    pub fn dim(&self) -> usize {
        let mut num_ones = 0;
        for ix in 0..K {
            num_ones += self.bits[ix].count_ones();
        }
        num_ones as usize
    }

    /// Returns the number of bits in the BitNode
    pub fn len(&self) -> usize {
        self.bits.len() * 8
    }
}

impl<const K: usize> Add for BitNode<K> {
    type Output = BitNode<K>;

    /// Basically returns the union of the two neighborhoods.
    fn add(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
        BitNode::from(new_bits)
    }
}

impl<const K: usize> Mul for BitNode<K> {
    type Output = BitNode<K>;

    /// Returns the intersection of the two neighborhoods.
    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
        BitNode::from(new_bits)
    }
}



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
    pub fn from(input_nodes: BitNode<K1>, output_nodes: BitNode<K2>, weight: EdgeWeight) -> BitEdge<K1, K2> {
        BitEdge { weight: weight, inputs: input_nodes, outputs: output_nodes }
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