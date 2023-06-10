use std::{collections::HashSet, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::traits::{HgBasis, HgNode};

/// Searches a sorted vector (does not check if it is sorted) and returns
/// the index of the node that would directly precede the provided node if
/// it were to be inserted. If the node is present in the vec then it returns the index of the provided node. We assume that duplicates are not allowed.
fn binary_search<N: HgNode>(sorted: &Vec<N>, node: &N) -> Option<usize> {
    if sorted.len() == 0 {
        return Some(0);
    }
    let max_count = (f64::log2(sorted.len() as f64).ceil() as usize) + 1;
    let mut l = 0;
    let mut r = sorted.len() - 1;
    let mut mid = (l + r) / 2;
    let mut reached_end = false;
    for _ in 0..max_count {
        if sorted[mid] < *node {
            l = mid;
        } else if sorted[mid] > *node {
            r = mid;
        } else {
            reached_end = true;
            break;
        }
        if r - l <= 1 {
            if sorted[l] < *node && sorted[r] > *node {
                mid = l;
            } else if sorted[l] < *node && sorted[r] <= *node {
                mid = r;
            } else if sorted[l] == *node {
                mid = l;
            }
            reached_end = true;
            break;
        } else {
            mid = (l + r) / 2;
        }
    }
    if reached_end {
        Some(mid)
    } else {
        None
    }
}

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

    pub fn node_set(&self) -> HashSet<N> {
        self.nodes.iter().cloned().collect()
    }

    pub fn to_node_set(self) -> HashSet<N> {
        self.nodes.into_iter().collect()
    }

    /// Returns true if node was not present and was added correctly, false otherwise (includes node already being present)
    pub fn add_node(&mut self, node: N) -> bool {
        if self.nodes.len() == 0 {
            self.nodes.push(node);
            return true;
        }
        if let Some(ix) = binary_search(&self.nodes, &node) {
            if ix == self.nodes.len() - 1 {
                if self.nodes[ix] < node {
                    self.nodes.push(node);
                    true
                } else {
                    false
                }
            } else {
                // ix + 1 indexing is safe due to above check
                // ix == self.nodes.len() - 1
                // Also this is where we guarantee that a node cannot
                // be inserted twice, if self.nodes[ix + 1] == node
                // then we do nothing.
                if self.nodes[ix] < node && self.nodes[ix + 1] > node {
                    self.nodes.insert(ix + 1, node);
                    true
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    fn contains_node(&self, node: &N) -> bool {
        if self.nodes.len() == 0 {
            return false;
        }
        let mut ret = false;
        if let Some(ix) = binary_search(&self.nodes, node) {
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
                if binary_search(&rhs.nodes, &node).is_some() {
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
            if let Some(ix) = binary_search(&self.nodes, &node_int) {
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

    use crate::{
        structs::{sparse_basis::binary_search, SparseBasis},
        traits::HgBasis,
    };

    #[test]
    fn test_bin_search() {
        let _rng = thread_rng();
        let mut data: Vec<u8> = (0..25).collect();
        data.sort();
        let out = binary_search(&data, &24);
        println!("data: {:?}", data);
        println!("binary search for last elem: {:?}", out);
    }

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
        let added_properly = b1.add_node(5);
        println!("added_properly: {:}", added_properly);
        println!("b1: {:?}", b1);
        let added_twice = b1.add_node(5);
        b1.add_node(4);
        println!("added twice? {:}", added_twice);
        println!("b1: {:?}", b1);
        let six = SparseBasis { nodes: vec![6_u8] };
        b1.remove_node(&six);
        println!("post removal: {:?}", b1);
        let maybe_ix = binary_search(&b1.nodes, &6);
        println!("maybe_ix: {:?}", maybe_ix);
        println!("try removing 6 again.");
        b1.remove_node(&six);
        println!("post removal: {:?}", b1);
    }
}
