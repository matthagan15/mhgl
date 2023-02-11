use std::{
    hash::Hash,
    ops::{Add, Mul},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BitNodes<const K: usize> {
    pub bits: [u8; K],
}

impl<const K: usize> BitNodes<K> {
    /// Gives the zero aka empty element.
    pub fn new() -> BitNodes<K> {
        BitNodes { bits: [0; K] }
    }

    pub fn from(bits: [u8; K]) -> BitNodes<K> {
        BitNodes { bits: bits }
    }

    pub fn is_node(&self) -> bool {
        let mut num_ones = 0;
        for ix in 0..K {
            num_ones += self.bits[ix].count_ones();
        }
        num_ones == 1
    }

    pub fn cardinality(&self) -> usize {
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

impl<const K: usize> Add for BitNodes<K> {
    type Output = BitNodes<K>;

    /// Returns the union of the two neighborhoods.
    fn add(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
        BitNodes::from(new_bits)
    }
}

impl<const K: usize> Mul for BitNodes<K> {
    type Output = BitNodes<K>;

    /// Returns the intersection of the two neighborhoods.
    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
        BitNodes::from(new_bits)
    }
}
