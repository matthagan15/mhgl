use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::structs::{sparse_edge::SparseEdge, EdgeWeight, NodeID};
use crate::traits::*;
use std::{
    collections::{HashMap, HashSet},
    ops::{Add, AddAssign, Mul, MulAssign},
};

/// This is basic dot product that tells us if two basis vectors are orthonormal. This is the
/// most straightforward way, if they are the same. Fails if duplicates in either vector is detected.
fn are_basis_elems_equal<N: HgNode>(a: &Vec<N>, b: &Vec<N>) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let dim = a.len();
    let mut a_set = HashSet::new();
    let mut b_set = HashSet::new();
    for ix in 0..dim {
        // prepare sets and check for duplicates. If duplicates detected, return false.
        if a_set.insert(a[ix].clone()) == false {
            return false;
        }
        if b_set.insert(b[ix].clone()) == false {
            return false;
        }
    }
    a_set.is_subset(&b_set) && b_set.is_subset(&a_set)
}

/// for future use with complex coefficients, etc.
fn dot_conj(w: EdgeWeight) -> EdgeWeight {
    w
}

/// Defaults to euclidean distance, should probably add in a parameter to determine
/// which metric to use.
pub fn distance<N: HgNode>(x: &SparseVector<N>, y: &SparseVector<N>) -> f64 {
    let mut tmp = y.clone();
    tmp *= -1.;
    tmp += x.clone();
    let mut tot = 0.0_f64;
    for (_, w) in tmp.basis_to_weight {
        tot += w.powi(2);
    }
    tot.sqrt()
}

/// A representation of a vector living in the power set module. Aka something a
/// hypergraph can act on in a linear fashion.
/// The basis elements are all possible sets of NodeIDs, the basic dot product
/// is defined as 1 if the two sets are equal, 0 otherwise. Note the empty set
/// exists.
/// # Invariant/Internals
/// Although a basis vector is a set, it's easiest to work with basis vectors when the
/// data storage is a HashMap from basis to coefficient. Unfortunately you cannot hash
/// a hashset due to the randomness, so we use SORTED vectors as the basis elements.
#[derive(Clone, Debug, Serialize)]
pub struct SparseVector<N: HgNode> {
    pub basis_to_weight: HashMap<Vec<N>, EdgeWeight>,
    cardinality_to_basis_set: HashMap<usize, HashSet<Vec<N>>>,
}

impl<N: HgNode> SparseVector<N> {
    pub fn new() -> SparseVector<N> {
        SparseVector {
            basis_to_weight: HashMap::new(),
            cardinality_to_basis_set: HashMap::new(),
        }
    }

    pub fn new_from(input: Vec<(HashSet<N>, EdgeWeight)>) -> SparseVector<N> {
        let mut map = HashMap::new();
        for (node_set, weight) in input.into_iter() {
            let mut node_vec: Vec<N> = node_set.into_iter().collect();
            node_vec.sort();
            let cur_weight = map.entry(node_vec).or_insert(0.);
            *cur_weight += weight;
        }
        SparseVector {
            basis_to_weight: map,
            cardinality_to_basis_set: HashMap::new(),
        }
    }
    pub fn basis(&self) -> Vec<(Vec<N>, EdgeWeight)> {
        self.basis_to_weight.clone().into_iter().collect()
    }

    pub fn from_basis(b: HashSet<N>, w: EdgeWeight) -> SparseVector<N> {
        SparseVector {
            basis_to_weight: HashMap::from([(b.into_iter().collect(), w)]),
            cardinality_to_basis_set: HashMap::new(),
        }
    }

    /// Samples a random basis element of a given cardinality in the vector.
    pub fn sample_basis_with_cardinality(&self, card: usize) -> HashSet<N> {
        if self.cardinality_to_basis_set.contains_key(&card) == false {
            HashSet::new()
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
                    let x: HashSet<N> = basis.iter().cloned().collect();
                    return x;
                }
            }
            HashSet::new()
        }
    }

    /// an empty EdgeVec is homogeneous vacuously
    /// Homogeneous in this setting means that it consists only of neighborhoods of the
    /// same size, the path stays at a constant dimension.
    fn is_homogeneous(&self) -> bool {
        if self.basis_to_weight.len() == 0 {
            return true;
        }
        let mut ret = true;
        let mut k = None;
        for (neighborhood, _) in self.basis_to_weight.iter() {
            if let Some(dim) = k {
                if dim != neighborhood.len() {
                    ret = false;
                    break;
                }
            } else {
                k = Some(neighborhood.len());
            }
        }
        ret
    }

    pub fn cardinality(&self) -> HashMap<usize, EdgeWeight> {
        let mut ret = HashMap::new();
        let mut tot = 0.0;
        for (k, v) in self.basis_to_weight.iter() {
            tot += v.abs();
            let card_val = ret.entry(k.len()).or_insert(0.0);
            *card_val = *card_val + v.abs();
        }
        for (k, v) in ret.iter_mut() {
            *v = *v / tot;
        }
        ret
    }

    /// Takes inner product, self is mapped to dual (aka weights of self get conjugated)
    pub fn dot(&self, other: &Self) -> EdgeWeight {
        let mut tot = 0.;
        for (self_hood, self_weight) in self.basis_to_weight.iter() {
            for (other_hood, other_weight) in other.basis().iter() {
                if are_basis_elems_equal(self_hood, other_hood) {
                    tot += dot_conj(*self_weight) + other_weight;
                }
            }
        }
        tot
    }
    pub fn apply_projection(&mut self, dim: usize) {
        let new_nodes: HashMap<Vec<N>, EdgeWeight> = self
            .basis_to_weight
            .clone()
            .into_iter()
            .filter(|(node_vec, _)| node_vec.len() == dim)
            .collect();
        self.basis_to_weight = new_nodes;
    }
}

// impl<N: HgNode> HgVector for SparseVector<N> {
//     type Basis = SparseBasis<N>;

//     fn zero() -> Self {
//         SparseVector::new()
//     }

//     /// is incorrect, what about 0.0 weight basis elements?
//     fn is_zero(&self) -> bool {
//         self.basis_to_weight.len() == 0
//     }

//     fn dot(&self, other: &Self) -> EdgeWeight {
//         let mut tot = 0.;
//         for (self_hood, self_weight) in self.basis_to_weight.iter() {
//             for (other_hood, other_weight) in other.basis().iter() {
//                 if are_basis_elems_equal(self_hood, other_hood) {
//                     tot += dot_conj(*self_weight) + other_weight;
//                 }
//             }
//         }
//         tot
//     }

//     fn basis(&self) -> &HashMap<Self::Basis, EdgeWeight> {
//         todo!()
//     }

//     fn from(basis_weight_pairs: Vec<(Self::Basis, EdgeWeight)>) -> Self {
//         // let mut hm = HashMap::with_capacity(basis_weight_pairs.len());
//         // let mut card_map = HashMap::new();
//         // for (b, w) in basis_weight_pairs.into_iter() {
//         //     let card = b.cardinality();
//         //     let card_set: &mut HashSet<Self::Basis> = card_map.entry(card).or_default();
//         //     card_set.insert(b.clone());
//         //     hm.insert(b, w);
//         // }
//         // SparseVector {
//         //     basis_to_weight: hm,
//         //     cardinality_to_basis_set: card_map,
//         // }
//         todo!()
//     }

//     fn project(&mut self, cardinality: usize) {
//         if let Some(chosen_ones) = self.cardinality_to_basis_set.get(&cardinality) {
//             let mut new_map = self.basis_to_weight.clone();
//             self.basis_to_weight = new_map
//                 .into_iter()
//                 .filter(|(b, w)| chosen_ones.contains(b))
//                 .collect();
//         }
//         let cards: Vec<_> = self.cardinality_to_basis_set.keys().cloned().collect();
//         for card in cards {
//             if card != cardinality {
//                 self.cardinality_to_basis_set.remove(&card);
//             }
//         }
//     }
//     fn l_norm(&self, l: i32) -> f64 {
//         let mut tot = 0.0;
//         for w in self.basis_to_weight.values() {
//             tot += w.abs().powi(l);
//         }
//         let exp = 1. / l as f64;
//         tot.powf(exp)
//     }
// }

impl<N: HgNode> PartialEq for SparseVector<N> {
    fn eq(&self, other: &Self) -> bool {
        let self_dot = self.dot(self);
        let other_dot = other.dot(self);
        self_dot == other_dot
    }
}

impl<N: HgNode> Eq for SparseVector<N> {}

impl<N: HgNode> Add for SparseVector<N> {
    type Output = SparseVector<N>;

    fn add(mut self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        for (basis, weight) in rhs.basis() {
            let new_weight = self.basis_to_weight.entry(basis).or_insert(0.);
            *new_weight = *new_weight + weight;
        }
        ret
    }
}

impl<N: HgNode> AddAssign for SparseVector<N> {
    fn add_assign(&mut self, rhs: Self) {
        for (basis, weight) in rhs.basis_to_weight.iter() {
            let old_weight = self.basis_to_weight.entry(basis.to_vec()).or_insert(0.);
            *old_weight = *old_weight + weight;
        }
    }
}

impl<N: HgNode> Mul<EdgeWeight> for SparseVector<N> {
    type Output = SparseVector<N>;

    fn mul(mut self, rhs: EdgeWeight) -> Self::Output {
        for (_, w) in self.basis_to_weight.iter_mut() {
            *w = *w * rhs;
        }
        self
    }
}

impl<N: HgNode> MulAssign<EdgeWeight> for SparseVector<N> {
    fn mul_assign(&mut self, rhs: EdgeWeight) {
        for (_, w) in self.basis_to_weight.iter_mut() {
            *w = *w * rhs;
        }
    }
}

mod test {
    use std::collections::HashSet;

    use uuid::Uuid;

    use super::SparseVector;

    #[test]
    fn test_add() {
        let mut nodes: HashSet<u8> = { 0..10 }.collect();
        let b1: HashSet<u8> = { 0..2 }.collect();
        let b2: HashSet<u8> = { 0..3 }.collect();
        let vec1 = SparseVector::from_basis(b1.clone(), 1.);
        let mut vec2 = SparseVector::from_basis(b2, 2.);
        println!("vec2 after creation: {:?}", vec2);
        vec2 += SparseVector::from_basis(b1, 3.);
        println!("vec2 after addition assign: {:?}", vec2);
        println!("vec1 + vec2: {:?}", (vec1 + vec2));
    }
}
