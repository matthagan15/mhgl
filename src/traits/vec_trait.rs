use crate::structs::EdgeWeight;
use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
};

pub trait HgVector:
    PartialEq + Eq + Clone + Add + AddAssign + Mul<EdgeWeight> + MulAssign<EdgeWeight>
{
    type Basis: Hash;
    fn zero() -> Self;
    fn is_zero(&self) -> bool;
    fn dot(&self, other: &Self) -> EdgeWeight;
    fn basis(&self) -> HashMap<Self::Basis, EdgeWeight>;
    fn from(basis_weight_pairs: Vec<(Self::Basis, EdgeWeight)>) -> Self;
    fn project(&mut self, cardinality: usize);
}
