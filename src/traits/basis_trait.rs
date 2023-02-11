use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
};

/// For BasisTrait types Add and Mul represent set-theoretic operations. Add corresponds to taking the union of the underlying subset of N and Mul corresponds to taking the intersection of the underlying subset.
pub trait HgBasis:
    Sized + PartialEq + Eq + Hash + Clone
{
    fn cardinality(&self) -> usize;
    fn intersect_with(&mut self, rhs: &Self);
    fn intersection(&self, rhs: &Self);
    fn union_with(&mut self, rhs: &Self);
    fn union(&self, rhs: &Self);
    fn matches_cardinality(&self, cardinality: usize) -> bool {
        self.cardinality() == cardinality
    }
    fn is_empty_set(&self) -> bool {
        self.matches_cardinality(0)
    }
}
