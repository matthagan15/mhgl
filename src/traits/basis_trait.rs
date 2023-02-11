use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
};

/// For BasisTrait types Add and Mul represent set-theoretic operations. Add corresponds to taking the union of the underlying subset of N and Mul corresponds to taking the intersection of the underlying subset.
pub trait HgBasis:
    Sized + Add + AddAssign + Mul + MulAssign + PartialEq + Eq + Hash + Clone
{
    type Node;

    /// Use of HashSet forces all nodes to be distinct.
    fn from(nodes: HashSet<Self::Node>) -> Self;
    fn cardinality(&self) -> usize;
    fn matches_cardinality(&self, cardinality: usize) -> bool {
        self.cardinality() == cardinality
    }
    fn is_empty_set(&self) -> bool {
        self.matches_cardinality(0)
    }
}
