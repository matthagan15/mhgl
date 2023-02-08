use std::{
    hash::Hash,
    ops::{Add, Mul},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct SparseNode(Uuid);

pub trait NodeID: Eq + PartialEq + Hash + PartialOrd + Ord + Clone + Copy + Serialize {}

impl NodeID for Uuid {}
impl NodeID for u64 {}
impl NodeID for u32 {}
impl NodeID for u16 {}
impl NodeID for u8 {}

#[derive(PartialEq, Eq, Debug)]
pub struct BitNode<const K: usize> {
    pub bits: [u8; K],
}

impl<const K: usize> BitNode<K> {
    pub fn new() -> BitNode<K> {
        BitNode { bits: [0; K] }
    }

    pub fn from(bits: [u8; K]) -> BitNode<K> {
        BitNode { bits: bits }
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
