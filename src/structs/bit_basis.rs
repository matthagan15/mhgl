use std::{
    collections::HashSet,
    hash::Hash,
};

use bitvec::prelude::*;

use serde::{Deserialize, Serialize};

/// Representation of a subset of nodes using a binary
/// encoding. Use `flip_node` to set a specific node as present
/// or not present in the basis. `query_node` returns if
/// the node is present in the subset or not. Can be resized.
#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, PartialOrd, Eq)]
pub struct BitBasis {
    bv: BitVec,
    is_active: bool,
}

impl BitBasis {
    /// Returns a new empty set basis that is active.
    pub fn new(num_nodes: usize) -> Self {
        let mut new_bits = BitVec::with_capacity(num_nodes);
        for _ix in 0..num_nodes {
            new_bits.push(false);
        }
        BitBasis {
            bv: new_bits,
            is_active: true,
        }
    }

    pub fn from(num_nodes: usize, nodes_present: HashSet<usize>) -> Self {
        let mut new_bits = BitVec::from_iter((0..num_nodes).map(|_| false));
        for ix in 0..num_nodes {
            *new_bits.get_mut(ix).unwrap() = nodes_present.contains(&ix);
        }
        BitBasis {
            bv: new_bits,
            is_active: true,
        }
    }

    /// Initializes any new bits to false, removes any bits if the vec
    /// is being strunk
    pub fn change_size(&mut self, new_num_nodes: usize) {
        if new_num_nodes > self.bv.len() {
            self.bv
                .extend((0..(new_num_nodes - self.bv.len())).map(|_| false));
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

    /// Flip a node from being present to not-present and vice
    /// versa. Does nothing if index is out of bounds.
    pub fn flip_node(&mut self, node_index: usize) {
        if let Some(mut x) = self.bv.get_mut(node_index) {
            *x = *x ^ true;
        }
    }

    /// Returns true if the node at the given index is in the subset.
    /// Panics if index is out of bounds.
    pub fn query_node(&self, node_index: usize) -> bool {
        *self
            .bv
            .get(node_index)
            .expect("BitBasis query_node index out of bounds")
    }

    fn make_active(&mut self) {
        self.is_active = true;
    }
    fn make_inactive(&mut self) {
        self.is_active = false;
    }
}

// TODO: Handle situations with inactive inputs/self.
impl HgBasis for BitBasis {
    fn new_empty() -> Self {
        Self::from(0, HashSet::new())
    }

    fn len(&self) -> usize {
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
            BitBasis {
                bv: BitVec::new(),
                is_active: false,
            }
        } else {
            let mut intersection_bv = BitVec::with_capacity(self.bv.len());
            for ix in 0..self.bv.len() {
                intersection_bv.push(*self.bv.get(ix).unwrap() & *rhs.bv.get(ix).unwrap());
            }
            BitBasis {
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
            BitBasis {
                bv: BitVec::new(),
                is_active: false,
            }
        } else {
            let mut intersection_bv = BitVec::with_capacity(self.bv.len());
            for ix in 0..self.bv.len() {
                intersection_bv.push(*self.bv.get(ix).unwrap() | *rhs.bv.get(ix).unwrap());
            }
            BitBasis {
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
            BitBasis {
                bv: comp_bv,
                is_active: true,
            }
        } else {
            BitBasis {
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
                ret.insert(BitBasis {
                    bv: tmp,
                    is_active: true,
                });
            }
        }
        ret
    }
}

use crate::{traits::HgBasis};

mod test {
    

    

    #[test]
    fn test_bit_basis() {
        let mut b = BitBasis::new(5);
        b.flip_node(0);
        b.flip_node(4);
        dbg!(b);
    }
}
