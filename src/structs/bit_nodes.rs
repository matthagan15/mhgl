use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
    mem::size_of,
};

use serde::{ser::SerializeStruct, Serialize};


use crate::{
    traits::{HgBasis},
    utils::PowerSetBits,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct BitFieldBytes {
    bits: Vec<u8>,
    pub is_active: bool,
}

impl BitFieldBytes {

    /// Provides a new BitField initialized to the empty set
    /// and is initially inactive.
    pub fn new(num_nodes: usize) -> Self {
        let num_bytes = num_nodes / 8 + 1;
        let mut v = Vec::with_capacity(num_bytes);
        v.resize(num_bytes, 0_u8);
        BitFieldBytes { bits: v, is_active:  false }
    }
}

impl HgBasis for BitFieldBytes {
    fn new_empty() -> Self {
        let mut bfb = BitFieldBytes::new(0);
        bfb.is_active = true;
        bfb
    }

    fn cardinality(&self) -> usize {
        let mut tot = 0;
        for ix in 0..self.bits.len() {
            tot += self.bits[ix].count_ones();
        }
        tot as usize
    }

    fn intersect_with(&mut self, rhs: &Self) {
        todo!()
    }

    fn intersection(&self, rhs: &Self) -> Self {
        todo!()
    }

    fn union_with(&mut self, rhs: &Self) {
        todo!()
    }

    fn union(&self, rhs: &Self) -> Self {
        todo!()
    }

    fn remove_node(&mut self, node: &Self) {
        todo!()
    }

    fn complement(&self, rhs: &Self) -> Self {
        todo!()
    }

    fn nodes(&self) -> HashSet<Self> {
        todo!()
    }
}



#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct BitNodes<const K: usize> {
    pub bits: [u8; K],
    // TODO: Change BitNodes API to have this.
    // is_empty: bool,
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

    pub fn to_u32(self) -> HashSet<u32> {
        let mut ret = HashSet::with_capacity(self.cardinality());
        let mut pb = PowerSetBits { bits: self.bits };
        while pb.is_zero() == false {
            ret.insert(pb.leading_zeros());
            pb.flip_kth_one(1);
        }
        ret
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

// impl<const K: usize> HgNode for BitNodes<K> {}

impl<const K: usize> HgBasis for BitNodes<K> {
    fn new_empty() -> Self {
        BitNodes::<K>::new()
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

    fn intersection(&self, rhs: &Self) -> BitNodes<K> {
        let mut ret = BitNodes::new();
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

    fn union(&self, rhs: &Self) -> BitNodes<K> {
        let mut ret = BitNodes::new();
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
        BitNodes { bits: new_bits }
    }

    fn nodes(&self) -> std::collections::HashSet<Self> {
        let pb = PowerSetBits {
            bits: self.bits.clone(),
        };
        pb.get_nodes_set()
            .into_iter()
            .map(|array| BitNodes { bits: array })
            .collect()
    }
}

impl<const K: usize> Serialize for BitNodes<K> {
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
    use super::{BitNodes, BitFieldBytes};

    
    #[test]
    fn test_size_of_structs() {
        let og = BitNodes::<10>::new();
        let bytes = BitFieldBytes {bits: [0_u8; 10].into(), is_active: true};
        println!("og size: {:}", std::mem::size_of_val(&og));
        println!("bytes size: {:}", std::mem::size_of_val(&*bytes.bits));
    }
    

    #[test]
    fn test_bit_nodes_serialization() {
        let bn = BitNodes::from([0, 1, 2]);
        if let Ok(s) = serde_json::to_string(&bn) {
            println!("serde output: {:}", s);
        }
    }
}
