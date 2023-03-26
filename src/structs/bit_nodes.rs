use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
    mem::size_of,
};

use bitvec::prelude as bv;

use serde::{ser::SerializeStruct, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
struct NewBitNodes {
    bv: bv::BitVec,
    pub is_active: bool,
}

impl NewBitNodes {
    pub fn new(num_nodes: usize) -> Self {
        let mut new_bits = bv::BitVec::with_capacity(num_nodes);
        for ix in 0..num_nodes {
            *new_bits.get_mut(ix).unwrap() = false;
        }
        NewBitNodes { bv: new_bits, is_active: false }
    }

    pub fn from(num_nodes: usize, nodes_present: HashSet<usize>) -> Self {
        let mut new_bits = bv::BitVec::with_capacity(num_nodes);
        for ix in 0..num_nodes {
            *new_bits.get_mut(ix).unwrap() = nodes_present.contains(&ix);
        }
        NewBitNodes { bv: new_bits, is_active: true }
    }
    pub fn change_size(mut self, new_num_nodes: usize) {
        
    }
}

impl HgBasis for NewBitNodes {
    fn new_empty() -> Self {
        Self::from(0, HashSet::new())
    }

    fn cardinality(&self) -> usize {
        todo!()
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

use crate::{
    traits::{HgBasis},
    utils::PowerSetBits,
};

/// # HEAVY CONSTRUCTION
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
struct BitFieldBytes {
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


/// ## Under Construction
/// Binary encoding of a subset of nodes for a hypergraph. 
/// Utilizes constant generics so that bitstrings can be stored
/// on the stack as opposed to heap. The key concept is that you
/// arbitrarily order your nodes and assign each node to a single bit
/// in the bit string. Then a given binary string represents a subset,
/// the bits that are 1 indicate that node is present in the subset.
/// This allows for more compact edge storage for smaller graphs. 
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct BitBasis<const K: usize> {
    pub bits: [u8; K],
    // TODO: Change BitNodes API to have this.
    // is_empty: bool,
}

impl<const K: usize> BitBasis<K> {
    /// Gives the zero aka empty element.
    pub fn new() -> BitBasis<K> {
        BitBasis { bits: [0; K] }
    }

    pub fn from(bits: [u8; K]) -> BitBasis<K> {
        BitBasis { bits: bits }
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
    pub fn resize<const M: usize>(self) -> BitBasis<M> {
        let mut new_basis = BitBasis::<M>::new();
        for ix in 0.. (M.min(K)) {
            // Direct index is fine due to min above
            new_basis.bits[ix] = self.bits[ix];
        }
        new_basis
    }
}


impl<const K: usize> Add for BitBasis<K> {
    type Output = BitBasis<K>;

    /// Returns the union of the two neighborhoods.
    fn add(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
        BitBasis::from(new_bits)
    }
}

impl<const K: usize> AddAssign for BitBasis<K> {
    fn add_assign(&mut self, rhs: Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] | rhs.bits[ix];
        }
    }
}

impl<const K: usize> Mul for BitBasis<K> {
    type Output = BitBasis<K>;

    /// Returns the intersection of the two neighborhoods.
    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_bits = [0; K];
        for ix in 0..K {
            new_bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
        BitBasis::from(new_bits)
    }
}

impl<const K: usize> MulAssign for BitBasis<K> {
    fn mul_assign(&mut self, rhs: Self) {
        for ix in 0..K {
            self.bits[ix] = self.bits[ix] & rhs.bits[ix];
        }
    }
}

// impl<const K: usize> HgNode for BitNodes<K> {}

impl<const K: usize> HgBasis for BitBasis<K> {
    fn new_empty() -> Self {
        BitBasis::<K>::new()
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

    fn intersection(&self, rhs: &Self) -> BitBasis<K> {
        let mut ret = BitBasis::new();
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

    fn union(&self, rhs: &Self) -> BitBasis<K> {
        let mut ret = BitBasis::new();
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
        BitBasis { bits: new_bits }
    }

    fn nodes(&self) -> std::collections::HashSet<Self> {
        let pb = PowerSetBits {
            bits: self.bits.clone(),
        };
        pb.get_nodes_set()
            .into_iter()
            .map(|array| BitBasis { bits: array })
            .collect()
    }
}

impl<const K: usize> Serialize for BitBasis<K> {
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

    use super::{BitBasis, BitFieldBytes};

    
    #[test]
    fn test_size_of_structs() {
        let og = BitBasis::<10>::new();
        let bytes = BitFieldBytes {bits: [0_u8; 10].into(), is_active: true};
        println!("og size: {:}", std::mem::size_of_val(&og));
        println!("bytes size: {:}", std::mem::size_of_val(&*bytes.bits));
    }
    

    #[test]
    fn test_bit_nodes_serialization() {
        let bn = BitBasis::from([0, 1, 2]);
        if let Ok(s) = serde_json::to_string(&bn) {
            println!("serde output: {:}", s);
        }
    }

    #[test]
    fn test_resize() {
        let mut og = BitBasis::<10>::new();
        og.bits[0] = 0b1010_0000_u8;
        println!("og: {:?}", og);
        let new: BitBasis<5> = og.resize();
        println!("new: {:?}", new);
    }
}
 