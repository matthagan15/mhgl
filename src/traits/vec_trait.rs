use crate::structs::EdgeWeight;
use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Add, AddAssign, Mul, MulAssign},
};

use super::HgBasis;

pub trait HgVector:
    PartialEq + Eq + Clone + Add + AddAssign + Mul<EdgeWeight> + MulAssign<EdgeWeight>
{
    type Basis: HgBasis;
    fn zero() -> Self;
    fn is_zero(&self) -> bool;
    fn basis(&self) -> &HashMap<Self::Basis, EdgeWeight>;
    fn dot(&self, other: &Self) -> EdgeWeight {
        let self_basis = self.basis();
        let other_basis = other.basis();
        let mut tot = 0.0;
        for (b, w) in self_basis.iter() {
            if let Some(other_w) = other_basis.get(b) {
                tot += w * other_w;
            }
        }
        tot
    }
    fn from(basis_weight_pairs: Vec<(Self::Basis, EdgeWeight)>) -> Self;
    fn project(&mut self, cardinality: usize);
    fn l_norm(&self, l: i32) -> f64;
}
