use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::AddAssign;
use uuid::Uuid;

/// # Nodes
/// This library uses the notation "node" as opposed to vertex to align with my research. The
/// rationale behind this is that "V" is preferred to represent a vector space as opposed to
/// some other letter, which makes using "N" for the node set of a hypergraph a bit easier.
///
/// The fundamental concept of the nodes in the graph are objects that one is interested in
/// modeling that are uniquely identifiable and one is interested in behavior of groups of
/// these objects. To align with the 3 different hypergraph implementations there are currently
/// 3 ways nodes are represented (respectively) 1. 128 bit Uuids using the uuid crate 2. A single
/// "1" in a binary number (if N nodes are being used in a graph we use N bits to represent the graph,
/// so each node gets its own "spot") 3. As a specific index to the adjacency/walk matrix used for the densest hypergraphs
/// (indices may also represent subsets of nodes, so all nodes are indices not all indices are nodes).
/// There is currently no traits or other generalizations to make this easier to work with. We do
/// not yet support adding labels to nodes either, and that must be done by hand by the end user.
pub trait HgNode:
    Debug + Eq + PartialEq + Hash + PartialOrd + Ord + Clone + Copy + Serialize
{
    fn max_number() -> Self;
    fn zero() -> Self;
    fn plus_one(&mut self);
}

impl HgNode for Uuid {
    fn max_number() -> Self {
        Uuid::from_u128(u128::MAX)
    }

    fn zero() -> Self {
        Uuid::nil()
    }

    fn plus_one(&mut self) {
        *self = Uuid::from_u128(self.as_u128() + 1)
    }
}
impl HgNode for u128 {
    fn max_number() -> Self {
        u128::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u64 {
    fn max_number() -> Self {
        u64::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u32 {
    fn max_number() -> Self {
        u32::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u16 {
    fn max_number() -> Self {
        u16::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u8 {
    fn max_number() -> Self {
        u8::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
