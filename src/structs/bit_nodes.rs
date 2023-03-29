use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
};

use bitvec::prelude::*;

use serde::{ser::SerializeStruct, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
pub struct BitVecBasis {
    bv: BitVec,
    pub is_active: bool,
}

impl BitVecBasis {
    pub fn new(num_nodes: usize) -> Self {
        let mut new_bits = BitVec::with_capacity(num_nodes);
        for ix in 0..num_nodes {
            *new_bits.get_mut(ix).unwrap() = false;
        }
        BitVecBasis { bv: new_bits, is_active: false }
    }

    pub fn from(num_nodes: usize, nodes_present: HashSet<usize>) -> Self {
        let mut new_bits = BitVec::with_capacity(num_nodes);
        for ix in 0..num_nodes {
            *new_bits.get_mut(ix).unwrap() = nodes_present.contains(&ix);
        }
        BitVecBasis { bv: new_bits, is_active: true }
    }

    /// Initializes any new bits to false, removes any bits if the vec
    /// is being strunk
    pub fn change_size(&mut self, new_num_nodes: usize) {
        if new_num_nodes > self.bv.len() {
            self.bv.extend((0..(new_num_nodes - self.bv.len())).map(|_| false ));
        } else {
            for _ in 0..(self.bv.len() - new_num_nodes) {
                self.bv.pop();
            }
        }
    }

    pub fn to_usize_set(&self) -> HashSet<usize> {
        let mut ret = HashSet::new();
        for ix in 0..self.bv.len() {
            if *self.bv.get(ix).unwrap() {
                ret.insert(ix);
            }
        }
        ret
    }
}

impl HgBasis for BitVecBasis {
    fn new_empty() -> Self {
        Self::from(0, HashSet::new())
    }

    fn cardinality(&self) -> usize {
        self.bv.count_ones()
    }

    fn intersect_with(&mut self, rhs: &Self) {
        if self.bv.len() == rhs.bv.len() {
            for ix in 0..self.bv.len() {
                let mut lhs_bit = self.bv.get_mut(ix).unwrap();
                let rhs_bit = rhs.bv.get(ix).unwrap();
                *lhs_bit = *lhs_bit & *rhs_bit;
            }
        }
    }

    fn intersection(&self, rhs: &Self) -> Self {
        if self.bv.len() != rhs.bv.len() {
            BitVecBasis {
                bv: BitVec::new(),
                is_active: false,
            }
        } else {
            let mut intersection_bv = BitVec::with_capacity(self.bv.len());
            for ix in 0..self.bv.len() {
                intersection_bv.push(*self.bv.get(ix).unwrap() & *rhs.bv.get(ix).unwrap());   
            }
            BitVecBasis {
                bv: intersection_bv,
                is_active: true,
            }
        }
    }

    fn union_with(&mut self, rhs: &Self) {
        if self.bv.len() == rhs.bv.len() {
            for ix in 0..rhs.bv.len() {
                let mut lhs = self.bv.get_mut(ix).unwrap();
                *lhs = *lhs | *rhs.bv.get(ix).unwrap();
            }
        }
    }

    fn union(&self, rhs: &Self) -> Self {
        if self.bv.len() != rhs.bv.len() {
            BitVecBasis {
                bv: BitVec::new(),
                is_active: false,
            }
        } else {
            let mut intersection_bv = BitVec::with_capacity(self.bv.len());
            for ix in 0..self.bv.len() {
                intersection_bv.push(*self.bv.get(ix).unwrap() | *rhs.bv.get(ix).unwrap());   
            }
            BitVecBasis {
                bv: intersection_bv,
                is_active: true,
            }
        }
    }

    fn remove_node(&mut self, node: &Self) {
        if self.bv.len() == node.bv.len() && node.bv.count_ones() == 1 {
            *self.bv.get_mut(node.bv.leading_zeros()).unwrap() = false;
        }
    }

    fn complement(&self, rhs: &Self) -> Self {
        if self.bv.len() == rhs.bv.len() && self.is_active && rhs.is_active {
            let mut comp_bv = BitVec::with_capacity(self.bv.len());
            for ix in 0..rhs.bv.len() {
                let lhs_bit = self.bv.get(ix).unwrap();
                let rhs_bit = rhs.bv.get(ix).unwrap();
                comp_bv.push(*lhs_bit ^ *rhs_bit);
            }
            BitVecBasis {
                bv: comp_bv,
                is_active: true,
            }
        } else {
            BitVecBasis {
                bv: BitVec::new(),
                is_active: false,
            }
        }
    }

    fn nodes(&self) -> HashSet<Self> {
        let mut ret = HashSet::new();
        let all_zeros = BitVec::from_iter((0..self.bv.len()).map(|_| false));
        for ix in 0..self.bv.len() {
            if *self.bv.get(ix).unwrap() {
                let mut tmp = all_zeros.clone();
                *tmp.get_mut(ix).unwrap() = true;
                ret.insert(BitVecBasis { bv: tmp, is_active: true });
            }
        }
        ret
    }
}

use crate::{
    traits::{HgBasis},
    utils::PowerSetBits,
};

/// ## Under Construction
/// Binary encoding of a subset of nodes for a hypergraph. 
/// Utilizes constant generics so that bitstrings can be stored
/// on the stack as opposed to heap. The key concept is that you
/// arbitrarily order your nodes and assign each node to a single bit
/// in the bit string. Then a given binary string represents a subset,
/// the bits that are 1 indicate that node is present in the subset.
/// This allows for more compact edge storage for smaller graphs. 
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct ConstGenBitBasis<const K: usize> {
    pub bits: [u8; K],
    // TODO: Change BitNodes API to have this.
    // is_empty: bool,
}

impl<const K: usize> ConstGenBitBasis<K> {
    /// Gives the zero aka empty element.
    pub fn new() -> ConstGenBitBasis<K> {
        ConstGenBitBasis { bits: [0; K] }
    }

    pub fn from(bits: [u8; K]) -> ConstGenBitBasis<K> {
        ConstGenBitBasis { bits: bits }
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

    pub fn to_u32(self) -> HashSet<u32> {
        let mut ret = HashSet::with_capacity(self.cardinality());
        let mut pb = PowerSetBits { bits: self.bits };
        while pb.is_zero() == false {
            ret.insert(pb.leading_zeros());
            pb.flip_kth_one(1);
        }
        ret
    }

    /// When shrinking the current behavior is to literally throw
    /// out the data being cut. Might be nice to give an option to
    /// indicate that data was lost.
    pub fn resize<const M: usize>(self) -> ConstGenBitBasis<M> {
        let mut new_basis = ConstGenBitBasis::<M>::new();
        for ix in 0.. (M.min(K)) {
            // Direct index is fine due to min above
            new_basis.bits[ix] = self.bits[ix];
        }
        new_basis
    }
}


impl<const K: usize> Add for ConstGenBitBasis<K> {
    type Output = ConstGenBitBasis<K>;

    /// Returns the union of the two neighborhoods.
    fn add(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
        ConstGenBitBasis::from(new_bits)
    }
}

impl<const K: usize> AddAssign for ConstGenBitBasis<K> {
    fn add_assign(&mut self, rhs: Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
    }
}

impl<const K: usize> Mul for ConstGenBitBasis<K> {
    type Output = ConstGenBitBasis<K>;

    /// Returns the intersection of the two neighborhoods.
    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
        ConstGenBitBasis::from(new_bits)
    }
}

impl<const K: usize> MulAssign for ConstGenBitBasis<K> {
    fn mul_assign(&mut self, rhs: Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
    }
}

// impl<const K: usize> HgNode for BitNodes<K> {}

impl<const K: usize> HgBasis for ConstGenBitBasis<K> {
    fn new_empty() -> Self {
        ConstGenBitBasis::<K>::new()
    }
    fn cardinality(&self) -> usize {
        let mut num_ones = 0;
        for ix in 0..K {
            num_ones += self.bits[ix].count_ones();
        }
        num_ones as usize
    }

    fn intersect_with(&mut self, rhs: &Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] & rhs.bits[ix]
        }
    }

    fn intersection(&self, rhs: &Self) -> ConstGenBitBasis<K> {
        let mut ret = ConstGenBitBasis::new();
        for ix in 0..K {
            ret.bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
        ret
    }

    fn union_with(&mut self, rhs: &Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
    }

    fn union(&self, rhs: &Self) -> ConstGenBitBasis<K> {
        let mut ret = ConstGenBitBasis::new();
        for ix in 0..K {
            ret.bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
        ret
    }

    fn remove_node(&mut self, node: &Self) {
        if node.cardinality() == 1 {
            for ix in 0..K {
                self.bits[ix] = self.bits[ix] ^ node.bits[ix];
            }
        }
    }

    fn complement(&self, rhs: &Self) -> Self {
        let mut new_bits = [0_u8; K];
        for ix in 0..K {
            // First get all the bits that are only 1 for one of the basis
            new_bits[ix] = self.bits[ix] ^ rhs.bits[ix];
            // Then make sure that one is self.
            new_bits[ix] = new_bits[ix] & self.bits[ix];
        }
        ConstGenBitBasis { bits: new_bits }
    }

    fn nodes(&self) -> std::collections::HashSet<Self> {
        let pb = PowerSetBits {
            bits: self.bits.clone(),
        };
        pb.get_nodes_set()
            .into_iter()
            .map(|array| ConstGenBitBasis { bits: array })
            .collect()
    }
}

impl<const K: usize> Serialize for ConstGenBitBasis<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("BitNodes", K)?;
        let v = self.bits.to_vec();
        s.serialize_field("bits", &v)?;
        s.end()
    }
}

mod test {
    use crate::traits::HgBasis;

    use super::{ConstGenBitBasis};
    
    #[test]
    fn test_bit_nodes_serialization() {
        let bn = ConstGenBitBasis::from([0, 1, 2]);
        if let Ok(s) = serde_json::to_string(&bn) {
            println!("serde output: {:}", s);
        }
    }

    #[test]
    fn test_resize() {
        let mut og = ConstGenBitBasis::<10>::new();
        og.bits[0] = 0b1010_0000_u8;
        println!("og: {:?}", og);
        let new: ConstGenBitBasis<5> = og.resize();
        println!("new: {:?}", new);
    }
}
 
