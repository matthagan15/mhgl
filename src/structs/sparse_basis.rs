use std::{
    collections::HashSet, fmt::{Debug, Display}
};


use serde::{Deserialize, Serialize};

use crate::traits::{HgBasis, HgNode};

/// A sparse representation of a subset of nodes. Empty sets provided with `new`
/// and `from` can be used to create a a SparseBasis object from a HashSet. Can
/// be converted to a HashSet using `to_node_set()`. Can take `union` and
/// `intersection` with other basis to yield a new basis or `union_with` and
/// `intersect_with` to mutate the basis. See `HgBasis` for complete methods.
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct SparseBasis<N: HgNode> {
    nodes: Vec<N>,
}

impl<N: HgNode> From<HashSet<N>> for SparseBasis<N> {
    fn from(value: HashSet<N>) -> Self {
        let mut nodes: Vec<N> = value.into_iter().collect();
        nodes.sort();
        SparseBasis { nodes }
    }
}

impl<N: HgNode> From<&[N]> for SparseBasis<N> {
    /// Clones the provided nodes. 
    fn from(value: &[N]) -> Self {
        let mut nodes: Vec<N> = value.iter().cloned().collect();
        nodes.sort();
        SparseBasis { nodes }
    }
}

impl<N: HgNode> From<&N> for SparseBasis<N> {
    /// Clones the provided nodes. 
    fn from(value: &N) -> Self {
        SparseBasis {nodes: vec![*value] }
    }
}


impl<N: HgNode> SparseBasis<N> {
    pub fn new() -> Self {
        SparseBasis { nodes: Vec::new() }
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

impl<N: HgNode> Serialize for SparseBasis<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.nodes.len() == 0 {
            return serializer.serialize_str("[]");
        }
        let mut s = String::new();
        s.push_str("[");
        for node in self.nodes.iter() {
            s.push_str(&format!("{:?},", node));
        }
        if s.ends_with(',') {
            s.remove(s.len() - 1);
        }
        s.push_str("]");
        serializer.serialize_str(&s)
    }
}

impl<'de, N: HgNode> Deserialize<'de> for SparseBasis<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut data = <String>::deserialize(deserializer)?;

        if data.starts_with("[") {
            data.remove(0);
        }
        if data.ends_with("]") {
            data.remove(data.len() - 1);
        }
        if data.contains(",") {
            let mut v: Vec<N> = data
                .split(',')
                .filter_map(|x| -> Option<N> {
                    if let Ok(number) = x.parse() {
                        Some(number)
                    } else {
                        None
                    }
                })
                .collect();
            v.sort();
            Ok(SparseBasis { nodes: v })
        } else {
            if let Ok(n) = data.parse::<N>() {
                Ok(SparseBasis { nodes: vec![n] })
            } else {
                if data.len() == 0 {
                    Ok(SparseBasis { nodes: vec![] })
                } else {
                    println!("Data: {:?}", data);
                    panic!("Could not parse single input.");
                }
            }
        }
        
    }
}

impl<N: HgNode> Display for SparseBasis<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str("{");
        for node in self.nodes.iter() {
            s.push_str(&format!("{:?},", node));
        }
        s.truncate(s.len() - 1);
        s.push_str("}");
        f.write_str(&s)
    }
}

impl<N: HgNode + Debug> HgBasis for SparseBasis<N> {
    fn new_empty() -> Self {
        SparseBasis::<N>::new()
    }

    fn len(&self) -> usize {
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

    #[test]
    fn test_serialization() {
        let b: SparseBasis<u32> = SparseBasis::from_slice(&[1, 2, 3]);
        let empty_basis: SparseBasis<u32> = SparseBasis::new_empty();
        let empty_s = serde_json::to_string(&empty_basis).expect("could not serialize emtpy.");
        println!("empty_s: {:?}", empty_s);
        let s = serde_json::to_string(&b).expect("could not serialize basis");
        dbg!(&s);
        let hm = HashMap::from([(b, 2)]);
        let s_hm = serde_json::to_string(&hm);
        dbg!(&s_hm);
        let vec: SparseBasis<u32> = serde_json::from_str(&s).unwrap();
        dbg!(vec);
        let empty: SparseBasis<u32> = serde_json::from_str(&"\"\"").expect("could not parse empty string");
        let single_node = "\"[1]\"";
        println!("input string: {:?}", single_node);
        let parsed: SparseBasis<u32> = serde_json::from_str(single_node).expect("where we parse at");
        println!("parsed: {:}", parsed);
    }
}
