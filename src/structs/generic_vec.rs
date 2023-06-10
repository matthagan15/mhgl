use std::{
    collections::{HashMap, HashSet},
    ops::{Add, AddAssign, Mul, MulAssign},
};

use rand::*;
use serde::Serialize;

use crate::traits::HgBasis;

use super::EdgeWeight;

/// A representation of a vector living in the power set vector space. Aka something a
/// hypergraph can act on in a linear fashion.
/// Essentially a collection of basis states (subsets of the overall node set)
/// and coefficients for each. Internally a map is stored from cardinality to
/// basis elements for easier lookup/projection.
#[derive(Clone, Debug, Serialize)]
pub struct GeneroVector<B: HgBasis> {
    pub basis_to_weight: HashMap<B, EdgeWeight>,
    cardinality_to_basis_set: HashMap<usize, HashSet<B>>,
}

impl<B: HgBasis> GeneroVector<B> {
    pub fn new() -> GeneroVector<B> {
        GeneroVector {
            basis_to_weight: HashMap::new(),
            cardinality_to_basis_set: HashMap::new(),
        }
    }

    pub fn new_from(input: Vec<(B, EdgeWeight)>) -> GeneroVector<B> {
        let mut basis_map = HashMap::with_capacity(input.len());
        let mut card_map = HashMap::new();
        for (basis, weight) in input.into_iter() {
            let cur_weight = basis_map.entry(basis.clone()).or_insert(0.);
            *cur_weight += weight;
            let card_basis_set = card_map
                .entry(basis.cardinality())
                .or_insert(HashSet::new());
            card_basis_set.insert(basis);
        }
        GeneroVector {
            basis_to_weight: basis_map,
            cardinality_to_basis_set: card_map,
        }
    }

    pub fn to_tuples(self) -> Vec<(B, EdgeWeight)> {
        self.basis().into_iter().collect()
    }

    pub fn basis(&self) -> Vec<(B, EdgeWeight)> {
        self.basis_to_weight.clone().into_iter().collect()
    }

    pub fn from_basis(b: B, w: EdgeWeight) -> GeneroVector<B> {
        GeneroVector {
            basis_to_weight: HashMap::from([(b.clone(), w)]),
            cardinality_to_basis_set: HashMap::from([(b.cardinality(), HashSet::from([b]))]),
        }
    }

    /// Samples a random basis element of a given cardinality in the vector.
    pub fn sample_basis_with_cardinality(&self, card: usize) -> Option<B> {
        if self.cardinality_to_basis_set.contains_key(&card) == false {
            None
        } else {
            let mut rng = thread_rng();
            let tot = self
                .cardinality_to_basis_set
                .get(&card)
                .unwrap()
                .iter()
                .fold(0.0, |acc, y| {
                    acc + match self.basis_to_weight.get(y) {
                        Some(w) => *w,
                        None => 0.0,
                    }
                });
            for basis in self.cardinality_to_basis_set.get(&card).unwrap().iter() {
                let p = match self.basis_to_weight.get(basis) {
                    Some(w) => *w,
                    None => 0.0,
                };
                if rng.gen_bool(p / tot) {
                    return Some(basis.clone());
                }
            }
            None
        }
    }

    /// an empty EdgeVec is homogeneous vacuously
    /// Homogeneous in this setting means that it consists only of neighborhoods of the
    /// same size, the path stays at a constant dimension.
    // fn is_homogeneous(&self) -> bool {
    //     if self.basis_to_weight.len() == 0 {
    //         return true;
    //     }
    //     let mut first_nonempty_k = None;
    //     for (k, h) in self.cardinality_to_basis_set.iter() {
    //         if first_nonempty_k.is_none() && h.len() > 0 {
    //             first_nonempty_k = Some(k);
    //         } else if first_nonempty_k.is_some() && h.len() > 0 {
    //             return false;
    //         }
    //     }
    //     true
    // }

    pub fn cardinality(&self) -> HashMap<usize, EdgeWeight> {
        let mut ret = HashMap::new();
        let mut tot = 0.0;
        for (b, v) in self.basis_to_weight.iter() {
            tot += v.abs();
            let card_val = ret.entry(b.cardinality()).or_insert(0.0);
            *card_val = *card_val + v.abs();
        }
        for (_k, v) in ret.iter_mut() {
            *v = *v / tot;
        }
        ret
    }

    /// Takes inner product, self is mapped to dual (aka weights of self get conjugated)
    pub fn dot(&self, other: &Self) -> EdgeWeight {
        let mut tot = 0.;
        if self.basis_to_weight.len() > other.basis_to_weight.len() {
            for (basis, w1) in other.basis_to_weight.iter() {
                if let Some(w2) = self.basis_to_weight.get(basis) {
                    tot += w1 * w2;
                }
            }
        } else {
            for (basis, w1) in self.basis_to_weight.iter() {
                if let Some(w2) = other.basis_to_weight.get(basis) {
                    tot += w1 * w2;
                }
            }
        }
        tot
    }
    pub fn apply_projection(&mut self, dim: usize) {
        let cards: HashSet<usize> = self.cardinality_to_basis_set.keys().cloned().collect();
        let mut basis_for_removal = HashSet::new();
        for card in cards {
            if card != dim {
                if let Some(basis_set) = self.cardinality_to_basis_set.remove(&card) {
                    for basis in basis_set {
                        basis_for_removal.insert(basis);
                    }
                }
            }
        }
        for basis in basis_for_removal {
            self.basis_to_weight.remove(&basis);
        }
    }

    /// Remove the basis element from the vector, returning the associated
    /// coefficient or 0. if it was not present.
    pub fn remove_basis(&mut self, basis: &B) -> EdgeWeight {
        if let Some(w) = self.basis_to_weight.remove(basis) {
            if let Some(card_set) = self.cardinality_to_basis_set.get_mut(&basis.cardinality()) {
                card_set.remove(basis);
            }
            w
        } else {
            0.
        }
    }
    pub fn add_basis(&mut self, basis: B, weight: EdgeWeight) {
        let old_weight = self.basis_to_weight.entry(basis.clone()).or_insert(0.);
        *old_weight = *old_weight + weight;
        let card_set = self
            .cardinality_to_basis_set
            .entry(basis.cardinality())
            .or_default();
        card_set.insert(basis);
    }
}

impl<B: HgBasis> Add for GeneroVector<B> {
    type Output = GeneroVector<B>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut total = HashMap::new();
        for (b, w) in self.basis_to_weight.iter() {
            total.insert(b.clone(), *w);
        }
        for (b, w) in rhs.basis_to_weight {
            let weight = total.entry(b).or_insert(0.0);
            *weight = *weight + w;
        }
        GeneroVector::new_from(total.into_iter().collect())
    }
}

impl<'a, B: HgBasis> AddAssign<&'a GeneroVector<B>> for GeneroVector<B> {
    fn add_assign(&mut self, rhs: &'a Self) {
        for (b, w) in rhs.basis_to_weight.iter() {
            if self.basis_to_weight.contains_key(b) {
                let x = self.basis_to_weight.get_mut(b).expect("just checked");
                *x = *x + *w;
            } else {
                self.basis_to_weight.insert(b.clone(), *w);
                let hs = self
                    .cardinality_to_basis_set
                    .entry(b.cardinality())
                    .or_default();
                hs.insert(b.clone());
            }
        }
    }
}

impl<B: HgBasis> Mul<EdgeWeight> for GeneroVector<B> {
    type Output = GeneroVector<B>;

    fn mul(self, rhs: EdgeWeight) -> Self::Output {
        let mut ret = self;
        for (_b, w) in ret.basis_to_weight.iter_mut() {
            *w = *w * rhs;
        }
        ret
    }
}

impl<B: HgBasis> MulAssign<EdgeWeight> for GeneroVector<B> {
    fn mul_assign(&mut self, rhs: EdgeWeight) {
        for (_b, w) in self.basis_to_weight.iter_mut() {
            *w = *w * rhs;
        }
    }
}
