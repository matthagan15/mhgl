use std::{collections::HashSet, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::traits::{HgBasis, HgNode};

/// A sparse representation of a subset of nodes. Empty sets provided with `new`
/// and `from` can be used to create a a SparseBasis object from a HashSet. Can
/// be converted to a HashSet using `to_node_set()`. Can take `union` and
/// `intersection` with other basis to yield a new basis or `union_with` and
/// `intersect_with` to mutate the basis. See `HgBasis` for complete methods.
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct SparseBasis<N: HgNode> {
    nodes: Vec<N>,
}

impl<N: HgNode> SparseBasis<N> {
    pub fn new() -> Self {
        SparseBasis { nodes: Vec::new() }
    }

    pub fn from(subset: HashSet<N>) -> Self {
        let mut nodes: Vec<N> = subset.into_iter().collect();
        nodes.sort();
        SparseBasis { nodes: nodes }
    }

    pub fn from_node(node: &N) -> Self {
        SparseBasis {
            nodes: vec![node.clone()],
        }
    }

    pub fn from_slice(nodes: &[N]) -> Self {
        let mut basis = nodes.to_vec();
        basis.sort();
        basis.dedup();
        SparseBasis { nodes: basis }
    }

    pub fn node_set(&self) -> HashSet<N> {
        self.nodes.iter().cloned().collect()
    }

    pub fn node_vec(&self) -> Vec<N> {
        self.nodes.clone()
    }

    pub fn to_node_set(self) -> HashSet<N> {
        self.nodes.into_iter().collect()
    }

    pub fn to_node_vec(self) -> Vec<N> {
        self.nodes
    }

    pub fn add_node(&mut self, node: N) {
        if let Err(ix) = self.nodes.binary_search(&node) {
            self.nodes.insert(ix, node);
        }
    }

    fn contains_node(&self, node: &N) -> bool {
        if self.nodes.len() == 0 {
            return false;
        }
        let mut ret = false;
        if let Ok(ix) = self.nodes.binary_search(node) {
            if self.nodes[ix] == *node {
                ret = true;
            }
        }
        ret
    }
}

impl<N: HgNode + Debug> HgBasis for SparseBasis<N> {
    fn new_empty() -> Self {
        SparseBasis::<N>::new()
    }

    fn cardinality(&self) -> usize {
        self.nodes.len()
    }

    fn intersect_with(&mut self, rhs: &Self) {
        let mut good_nodes = Vec::with_capacity(self.nodes.len());
        for _ in 0..self.nodes.len() {
            if let Some(node) = self.nodes.pop() {
                if rhs.nodes.binary_search(&node).is_ok() {
                    good_nodes.insert(0, node);
                }
            }
        }
        self.nodes.clear();
        self.nodes.append(&mut good_nodes);
    }

    /// Works with assumption that vecs are sorted.
    fn intersection(&self, rhs: &Self) -> SparseBasis<N> {
        let mut ret = Vec::new();
        let l1 = self.nodes.len();
        let l2 = rhs.nodes.len();
        let mut left_counter = 0;
        let mut right_counter = 0;
        for _ix in 0..(l1 + l2) {
            if left_counter == l1 || right_counter == l2 {
                break;
            }
            if self.nodes[left_counter] < rhs.nodes[right_counter] {
                left_counter += 1;
            } else if self.nodes[left_counter] > rhs.nodes[right_counter] {
                right_counter += 1;
            } else {
                ret.push(self.nodes[left_counter].clone());
                left_counter += 1;
                right_counter += 1;
            }
        }
        SparseBasis { nodes: ret }
    }

    fn union_with(&mut self, rhs: &Self) {
        for node in rhs.nodes.iter() {
            self.add_node(node.clone());
        }
    }

    // TODO: This is pretty unoptimal but it works.
    fn union(&self, rhs: &Self) -> Self {
        let lhs_set: HashSet<N> = self.nodes.iter().cloned().collect();
        let rhs_set: HashSet<N> = rhs.nodes.iter().cloned().collect();
        let mut union: Vec<N> = lhs_set.union(&rhs_set).cloned().collect();
        union.sort();
        SparseBasis { nodes: union }
    }

    fn remove_node(&mut self, node: &Self) {
        if node.nodes.len() == 1 {
            let node_int = node.nodes[0].clone();
            if let Ok(ix) = self.nodes.binary_search(&node_int) {
                if self.nodes[ix] == node_int {
                    self.nodes.remove(ix);
                }
            }
        }
    }

    fn complement(&self, rhs: &Self) -> Self {
        let mut ret = Vec::new();
        for ix in 0..self.nodes.len() {
            if rhs.contains_node(&self.nodes[ix]) == false {
                ret.push(self.nodes[ix].clone());
            }
        }
        // self.nodes.clone().into_iter().filter(|x| rhs.contains_node(x) == false).collect();
        SparseBasis { nodes: ret }
    }

    fn nodes(&self) -> HashSet<Self> {
        self.nodes
            .iter()
            .map(|n| SparseBasis {
                nodes: vec![n.clone()],
            })
            .collect()
    }
}

mod test {
    use rand::thread_rng;

    use crate::{structs::SparseBasis, traits::HgBasis};

    #[test]
    fn test_intersect_with() {
        let mut b1: SparseBasis<u8> = SparseBasis {
            nodes: vec![1, 3, 5, 6, 7, 8, 9],
        };
        let b2: SparseBasis<u8> = SparseBasis {
            nodes: vec![2, 3, 4, 5, 6, 7, 8, 9, 10],
        };
        b1.intersect_with(&b2);
        println!("b1 post intersection: {:?}", b1);
    }

    #[test]
    fn test_add_node() {
        let mut b1: SparseBasis<u8> = SparseBasis {
            nodes: vec![0, 1, 2, 3, 5, 6, 7, 8],
        };
        b1.add_node(5);
        println!("b1: {:?}", b1);
        b1.add_node(5);
        b1.add_node(4);
        println!("b1: {:?}", b1);
        let six = SparseBasis { nodes: vec![6_u8] };
        b1.remove_node(&six);
        println!("post removal: {:?}", b1);
        println!("try removing 6 again.");
        b1.remove_node(&six);
        println!("post removal: {:?}", b1);
    }
}
