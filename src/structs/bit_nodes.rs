use std::{
    hash::Hash,
    ops::{Add, Mul, MulAssign, AddAssign},
};

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use uuid::Uuid;

use crate::traits::{HgNode, HgBasis};

#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
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

impl<const K: usize> AddAssign for BitNodes<K> {
    fn add_assign(&mut self, rhs: Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
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

impl<const K: usize> MulAssign for BitNodes<K> {
    fn mul_assign(&mut self, rhs: Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
    }
}

impl<const K: usize> HgNode for BitNodes<K> {}

impl<const K: usize> HgBasis for BitNodes<K> {
    type Node = BitNodes<K>;

    fn from(nodes: std::collections::HashSet<Self::Node>) -> Self {
        let mut ret = BitNodes::new();
        for node in nodes {
            ret += node;
        }
        ret
    }

    fn cardinality(&self) -> usize {
        let mut num_ones = 0;
        for ix in 0..K {
            num_ones += self.bits[ix].count_ones();
        }
        num_ones as usize
    }
}

impl<const K: usize> Serialize for BitNodes<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut s = serializer.serialize_struct("BitNodes", K)?;
        let v = self.bits.to_vec();
        s.serialize_field("bits", &v)?;
        s.end()
    }
}

mod test {
    use serde::Serialize;

    use super::BitNodes;

    #[test]
    fn test_bit_nodes_serialization() {
        let bn = BitNodes::from([0, 1, 2]);
        if let Ok(s) = serde_json::to_string(&bn) {
            println!("serde output: {:}", s);
        }
    }
}
