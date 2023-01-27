use rand::{thread_rng, Rng};
use serde::{Serialize, Deserialize};

use crate::structs::{hyperedge::SparseEdge, EdgeWeight, NodeUUID};
use std::{collections::{HashMap, HashSet}, ops::{Add, AddAssign}};

use super::nodes::NodeID;

/// This is basic dot product that tells us if two basis vectors are orthonormal. This is the
/// most straightforward way, if they are the same. Fails if duplicates in either vector is detected.
fn are_basis_elems_equal<N: NodeID>(a: &Vec<N>, b: &Vec<N>) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let dim = a.len();
    let mut a_set = HashSet::new();
    let mut b_set = HashSet::new();
    for ix in 0..dim {
        // prepare sets and check for duplicates. If duplicates detected, return false.
        if a_set.insert(a[ix]) == false {
            return false;
        }
        if b_set.insert(b[ix]) == false {
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
pub fn distance<N: NodeID>(x: &HgVector<N>, y: &HgVector<N>) -> f64 {
    let mut tmp = y.clone();
    tmp.multiply_scalar(-1.);
    tmp += x.clone();
    let mut tot = 0.0_f64;
    for (_, w) in tmp.nodes {
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
pub struct HgVector<N: NodeID> {
    pub nodes: HashMap<Vec<N>, EdgeWeight>,
}

impl<N: NodeID> HgVector<N> {
    pub fn basis(&self) -> Vec<(Vec<N>, EdgeWeight)> {
        self.nodes.clone().into_iter().collect()
    }

    pub fn from_basis(b: HashSet<N>, w: EdgeWeight) -> HgVector<N> {
        HgVector {
            nodes: HashMap::from([(b.into_iter().collect(), w)]),
        }
    }

    /// Samples a random basis element of a given cardinality in the vector.
    pub fn sample_dim(&self, cardinality: usize) -> HashSet<N> {
        let good_ones: Vec<(Vec<N>, f64)> = self.nodes.iter().filter(|(x, _)| x.len() == cardinality).map(|(v, w)| (v.clone(), w.clone())).collect();
        let mut tot = 0.0_f64;
        for ix in 0..good_ones.len() {
            tot += good_ones[ix].1;
        }
        let mut acc_prob = 0.0_f64;
        let mut rng = thread_rng();
        for ix in 0..good_ones.len() {
            let sampled_number: f64 = rng.gen();
            if sampled_number < acc_prob / tot {
                return good_ones[ix].0.iter().cloned().collect()
            }
        }
        HashSet::new()
    }



    pub fn multiply_scalar(&mut self, s: EdgeWeight) {
        for (_, weight) in self.nodes.iter_mut() {
            *weight *= s;
        }
    }
    pub fn new() -> HgVector<N> {
        HgVector {
            nodes: HashMap::new(),
        }
    }

    pub fn new_from(input: Vec<(HashSet<N>, EdgeWeight)>) -> HgVector<N> {
        let mut map = HashMap::new();
        for (node_set, weight) in input.into_iter() {
            let mut node_vec: Vec<N> = node_set.into_iter().collect();
            node_vec.sort();
            let cur_weight = map.entry(node_vec).or_insert(0.);
            *cur_weight += weight;
        }
        HgVector { nodes: map }
    }

    /// an empty EdgeVec is homogeneous vacuously
    /// Homogeneous in this setting means that it consists only of neighborhoods of the
    /// same size, the path stays at a constant dimension.
    fn is_homogeneous(&self) -> bool {
        if self.nodes.len() == 0 {
            return true;
        }
        let mut ret = true;
        let mut k = None;
        for (neighborhood, _) in self.nodes.iter() {
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

    /// Takes inner product, self is mapped to dual (aka weights of self get conjugated)
    pub fn dot(&self, other: &Self) -> EdgeWeight {
        let mut tot = 0.;
        for (self_hood, self_weight) in self.nodes.iter() {
            for (other_hood, other_weight) in other.basis().iter() {
                if are_basis_elems_equal(self_hood, other_hood) {
                    tot += dot_conj(*self_weight) + other_weight;
                }
            }
        }
        1.
    }
    pub fn projector(&mut self, dim: usize) {
        let new_nodes: HashMap<Vec<N>, EdgeWeight> = self
            .nodes
            .clone()
            .into_iter()
            .filter(|(node_vec, _)| node_vec.len() == dim)
            .collect();
        self.nodes = new_nodes;
    }
}

impl<N: NodeID> Add for HgVector<N> {
    type Output = HgVector<N>;

    fn add(mut self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        for (basis, weight) in rhs.basis() {
            let new_weight = self.nodes.entry(basis).or_insert(0.);
            *new_weight = *new_weight + weight;
        }
        ret
    }
}

impl<N: NodeID> AddAssign for HgVector<N> {
    fn add_assign(&mut self, rhs: Self) {
        for (basis, weight) in rhs.nodes.iter() {
            let old_weight = self.nodes.entry(basis.to_vec()).or_insert(0.);
            *old_weight = *old_weight + weight;
        }
    }
}

mod test {
    use std::collections::HashSet;

    use uuid::Uuid;

    use super::HgVector;

    #[test]
    fn test_add() {
        let mut nodes: HashSet<u8> = {0..10}.collect();
        let b1: HashSet<u8> = {0..2}.collect();
        let b2: HashSet<u8> = {0..3}.collect();
        let vec1 = HgVector::from_basis(b1.clone(), 1.);
        let mut vec2 = HgVector::from_basis(b2, 2.);
        println!("vec2 after creation: {:?}", vec2);
        vec2 += HgVector::from_basis(b1, 3.);
        println!("vec2 after addition assign: {:?}", vec2);
        println!("vec1 + vec2: {:?}", (vec1 + vec2));
    }
}