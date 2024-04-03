use std::{
    collections::HashSet, fmt::{Debug, Display}
};


use serde::{Deserialize, Serialize};

use crate::traits::{HgBasis, HgNode};

/// A subset for an overall set system, note that for things like
/// deserializing and using `From`'s we default always to `Undirected`, so
/// if you want to make a `Edge::Simplex` from a `Vec` you have to do something like
/// ```
/// let vec = vec![1, 2, 3];
/// let simplex = Edge::from(vec).make_simplex();
/// ```
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum Edge<N: HgNode> {
    Undirected (Vec<N>),
    Simplex (Vec<N>),
}

impl<N: HgNode> Edge<N> {
    /// Creates an empty edge.
    pub fn new(simplex: bool) -> Self {
        if simplex {
            Edge::Simplex (Vec::new())
        } else {
            Edge::Undirected(Vec::new())
        }
    }

    pub fn is_simplex(&self) -> bool {
        match self {
            Edge::Undirected(_) => false,
            Edge::Simplex(_) => true,
        }
    }

    pub fn is_undirected(&self) -> bool {
        !self.is_simplex()
    }

    /// Number of nodes in the edge
    /// ```rust 
    /// let e = Edge::from([1,2,3]);
    /// assert_eq!(e.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.nodes_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_node(&self) -> bool {
        self.len() == 1
    }

    pub fn get_first_node(&self) -> Option<N> {
        self.nodes_ref().first().cloned()
    }

    /// Consumes `self`
    pub fn make_simplex(self) -> Self {
        match self {
            Edge::Undirected(nodes) => {
                Edge::Simplex(nodes)
            },
            Edge::Simplex(_) => {
                self
            },
        }
    }

    /// Consumes `self`
    pub fn make_undirected(self) -> Self {
        match self {
            Edge::Undirected(_) => {
                self
            },
            Edge::Simplex(nodes) => {
                Edge::Undirected(nodes)
            },
        }
    }

    pub fn node_set(&self) -> HashSet<N> {
        match self {
            Edge::Undirected(nodes) => {
                nodes.clone().into_iter().collect()
            },
            Edge::Simplex(nodes) => {
                nodes.clone().into_iter().collect()
            },
        }
    }

    pub fn node_vec(&self) -> Vec<N> {
        match self {
            Edge::Undirected(nodes) => {
                nodes.clone().into_iter().collect()
            },
            Edge::Simplex(nodes) => {
                nodes.clone().into_iter().collect()
            },
        }
    }

    pub fn to_node_set(self) -> HashSet<N> {
        match self {
            Edge::Undirected(nodes) => {
                nodes.into_iter().collect()
            },
            Edge::Simplex(nodes) => {
                nodes.into_iter().collect()
            },
        }
    }

    pub fn to_node_vec(self) -> Vec<N> {
        match self {
            Edge::Undirected(nodes) => {
                nodes.into_iter().collect()
            },
            Edge::Simplex(nodes) => {
                nodes.into_iter().collect()
            },
        }
    }

    pub fn add_node(&mut self, node: N) {
        match self {
            Edge::Undirected(nodes) => {
                if let Err(ix) = nodes.binary_search(&node) {
                    nodes.insert(ix, node);
                }
            },
            Edge::Simplex(nodes) => {
                if let Err(ix) = nodes.binary_search(&node) {
                    nodes.insert(ix, node);
                }
            },
        }
    }

    pub fn contains_node(&self, node: &N) -> bool {
        let searcher = |nodes: &Vec<N>| {
            if nodes.len() == 0 {
                return false;
            }
            let mut ret = false;
            if let Ok(ix) = nodes.binary_search(node) {
                if nodes[ix] == *node {
                    ret = true;
                }
            }
            ret
        };
        match self {
            Edge::Undirected(nodes) => searcher(nodes),
            Edge::Simplex(nodes) => searcher(nodes),
        }
    }

    pub fn intersect_with(&mut self, rhs: &Self) {
        let nodes: &mut Vec<N> = self.nodes_ref_mut();
        let mut good_nodes = Vec::with_capacity(nodes.len());
        for _ in 0..nodes.len() {
            if let Some(node) = nodes.pop() {
                if rhs.contains_node(&node) {
                    good_nodes.insert(0, node);
                }
            }
        }
        nodes.clear();
        nodes.append(&mut good_nodes);
    }

    pub fn intersection(&self, rhs: &Self) -> Self {
        let self_nodes = self.nodes_ref();
        let rhs_nodes = rhs.nodes_ref();
        let mut ret = Vec::new();
        let l1 = self_nodes.len();
        let l2 = rhs_nodes.len();
        let mut left_counter = 0;
        let mut right_counter = 0;
        for _ix in 0..(l1 + l2) {
            if left_counter == l1 || right_counter == l2 {
                break;
            }
            if self_nodes[left_counter] < rhs_nodes[right_counter] {
                left_counter += 1;
            } else if self_nodes[left_counter] > rhs_nodes[right_counter] {
                right_counter += 1;
            } else {
                ret.push(self_nodes[left_counter].clone());
                left_counter += 1;
                right_counter += 1;
            }
        }
        match self {
            Edge::Undirected(_) => Edge::Undirected(ret),
            Edge::Simplex(_) => Edge::Simplex(ret),
        }
    }

    pub fn union_with(&mut self, rhs: &Self) {
        let nodes = rhs.nodes_ref();
        for node in nodes.iter() {
            self.add_node(node.clone());
        }
    }

    pub fn nodes_ref<'a>(&'a self) -> &'a Vec<N> {
        match self {
            Edge::Undirected(nodes) => nodes,
            Edge::Simplex(nodes) => nodes,
        }
    }

    pub fn nodes_ref_mut<'a>(&'a mut self) -> &'a mut Vec<N> {
        match self {
            Edge::Undirected(nodes) => nodes,
            Edge::Simplex(nodes) => nodes,
        }
    }

    // TODO: This is pretty unoptimal but it works.
    pub fn union(&self, rhs: &Self) -> Self {
        let mut tot = HashSet::new();
        let lhs_nodes = self.nodes_ref();
        let rhs_nodes = rhs.nodes_ref();
        for node in lhs_nodes {
            tot.insert(*node);
        }
        for node in rhs_nodes {
            tot.insert(*node);
        }
        let mut union: Vec<N>= tot.into_iter().collect();
        union.sort();
        match self {
            Edge::Undirected(_) => Edge::Undirected(union),
            Edge::Simplex(_) => Edge::Simplex(union),
        }
    }

    /// If `rhs` is contained in self, returns the complement of rhs
    /// within self. If `rhs` is not fully contained in self returns the
    /// empty set. Note this is the same return value as `self.link(&self)`.
    /// This could lead to major bugs in the future, but the other option
    /// is that link returns an option. 
    pub fn link(&self, rhs: &Self) -> Option<Self> {
        if self.contains(rhs) == false {
            return None;
        }
        let mut ret_nodes = self.node_set();
        for node in rhs.nodes_ref() {
            ret_nodes.remove(node);
        }
        let nodes_vec: Vec<N> = ret_nodes.into_iter().collect();
        match self {
            Edge::Undirected(_) => Some(Edge::from(nodes_vec)),
            Edge::Simplex(_) => Some(Edge::from(nodes_vec).make_simplex()),
        }
    }

    pub fn remove_node(&mut self, node: &N) {
        let nodes = self.nodes_ref_mut();
        if let Ok(ix) = nodes.binary_search(node) {
            nodes.remove(ix);
        }
    }

    pub fn remove_nodes(&mut self, nodes: &Vec<N>) {
        for node in nodes.iter() {
            self.remove_node(node);
        }
        let nodes = self.nodes_ref_mut();
        nodes.shrink_to_fit();
    }

    pub fn contains(&self, other: &Self) -> bool {
        let lhs_nodes = self.nodes_ref();
        let rhs_nodes = other.nodes_ref();
        let mut left_ix = 0;
        let mut right_ix = 0;
        while right_ix < rhs_nodes.len() {
            if left_ix == lhs_nodes.len() {
                return false;
            }
            if lhs_nodes[left_ix] == rhs_nodes[right_ix] {
                right_ix += 1;
            } else {
                left_ix += 1;
            }
        }
        true
    }
}

impl<N: HgNode> Serialize for Edge<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let nodes = match self {
            Edge::Undirected(nodes) => nodes,
            Edge::Simplex(nodes) => nodes,
        };
        if nodes.len() == 0 {
            return serializer.serialize_str("[]");
        }
        let mut s = String::new();
        s.push_str("[");
        for node in nodes.iter() {
            s.push_str(&format!("{:?},", node));
        }
        if s.ends_with(',') {
            s.remove(s.len() - 1);
        }
        s.push_str("]");
        serializer.serialize_str(&s)
    }
}

impl<'de, N: HgNode> Deserialize<'de> for Edge<N> {
    /// Note: will default to Edge::Undirected
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
            Ok(Edge::Undirected(v))
        } else {
            if let Ok(n) = data.parse::<N>() {
                Ok(Edge::Undirected(vec![n]))
            } else {
                if data.len() == 0 {
                    Ok(Edge::Undirected(vec![]))
                } else {
                    println!("Data: {:?}", data);
                    panic!("Could not parse single input.");
                }
            }
        }
    }
}

impl<N: HgNode> Display for Edge<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nodes = match self {
            Edge::Undirected(nodes) => nodes,
            Edge::Simplex(nodes) => nodes,
        };
        let mut s = String::new();
        s.push_str("{");
        for node in nodes.iter() {
            s.push_str(&format!("{:?},", node));
        }
        s.truncate(s.len() - 1);
        s.push_str("}");
        f.write_str(&s)
    }
}

impl<N: HgNode> From<Vec<N>> for Edge<N> {
    fn from(value: Vec<N>) -> Self {
        let mut nodes = value;
        nodes.sort();
        nodes.dedup();
        Edge::Undirected(nodes)
    }
}

impl<N: HgNode> From<&[N]> for Edge<N> {
    fn from(value: &[N]) -> Self {
        let v: Vec<N> = value.iter().cloned().collect();
        Edge::from(v)
    }
}

impl<N: HgNode, const K: usize> From<[N; K]> for Edge<N> {
    fn from(value: [N; K]) -> Self {
        Edge::from(value.to_vec())
    }
}

impl<N: HgNode> From<HashSet<N>> for Edge<N> {
    fn from(value: HashSet<N>) -> Self {
        let v: Vec<N> = value.into_iter().collect();
        Edge::from(v)
    }
}

/// A subset of nodes of a set sytem. Empty sets provided with `new`
/// and `from` can be used to create a a SparseNodeSet object from a HashSet. Can
/// be converted to a HashSet using `to_node_set()`. Can take `union` and
/// `intersection` with other basis to yield a new basis or `union_with` and
/// `intersect_with` to mutate the basis. See `HgBasis` for complete methods.
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct SparseNodeSet<N: HgNode> {
    nodes: Vec<N>,
}

impl<N: HgNode> From<HashSet<N>> for SparseNodeSet<N> {
    fn from(value: HashSet<N>) -> Self {
        let mut nodes: Vec<N> = value.into_iter().collect();
        nodes.sort();
        SparseNodeSet { nodes }
    }
}

impl<N: HgNode> From<&[N]> for SparseNodeSet<N> {
    /// Clones the provided nodes. 
    fn from(value: &[N]) -> Self {
        let mut nodes: Vec<N> = value.iter().cloned().collect();
        nodes.sort();
        SparseNodeSet { nodes }
    }
}

impl<N: HgNode> From<&N> for SparseNodeSet<N> {
    /// Clones the provided nodes. 
    fn from(value: &N) -> Self {
        SparseNodeSet {nodes: vec![*value] }
    }
}


impl<N: HgNode> SparseNodeSet<N> {
    pub fn new() -> Self {
        SparseNodeSet { nodes: Vec::new() }
    }

    pub fn from_slice(nodes: &[N]) -> Self {
        let mut basis = nodes.to_vec();
        basis.sort();
        basis.dedup();
        SparseNodeSet { nodes: basis }
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

impl<N: HgNode> Serialize for SparseNodeSet<N> {
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

impl<'de, N: HgNode> Deserialize<'de> for SparseNodeSet<N> {
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
            Ok(SparseNodeSet { nodes: v })
        } else {
            if let Ok(n) = data.parse::<N>() {
                Ok(SparseNodeSet { nodes: vec![n] })
            } else {
                if data.len() == 0 {
                    Ok(SparseNodeSet { nodes: vec![] })
                } else {
                    println!("Data: {:?}", data);
                    panic!("Could not parse single input.");
                }
            }
        }
        
    }
}

impl<N: HgNode> Display for SparseNodeSet<N> {
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

impl<N: HgNode + Debug> HgBasis for SparseNodeSet<N> {
    fn new_empty() -> Self {
        SparseNodeSet::<N>::new()
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
    fn intersection(&self, rhs: &Self) -> SparseNodeSet<N> {
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
        SparseNodeSet { nodes: ret }
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
        SparseNodeSet { nodes: union }
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
        SparseNodeSet { nodes: ret }
    }

    fn nodes(&self) -> HashSet<Self> {
        self.nodes
            .iter()
            .map(|n| SparseNodeSet {
                nodes: vec![n.clone()],
            })
            .collect()
    }
}

mod test {
    use std::collections::HashMap;

    use crate::{HgBasis, SparseNodeSet};

    use super::Edge;

    

    

    #[test]
    fn test_contains() {
        let e1 = Edge::from([1_u8, 2, 3, 4]);
        let e2 = Edge::from([1_u8, 2, 3]);
        let e3 = Edge::from([0_u8, 7, 9]);
        assert!(e1.contains(&e2));
        assert!(!e2.contains(&e1));
        assert!(!e1.contains(&e3));
        assert!(!e3.contains(&e1));
    }

    #[test]
    fn test_intersect_with() {
        let mut b1: SparseNodeSet<u8> = SparseNodeSet {
            nodes: vec![1, 3, 5, 6, 7, 8, 9],
        };
        let b2: SparseNodeSet<u8> = SparseNodeSet {
            nodes: vec![2, 3, 4, 5, 6, 7, 8, 9, 10],
        };
        b1.intersect_with(&b2);
        println!("b1 post intersection: {:?}", b1);
    }

    #[test]
    fn test_add_node() {
        let mut b1: SparseNodeSet<u8> = SparseNodeSet {
            nodes: vec![0, 1, 2, 3, 5, 6, 7, 8],
        };
        b1.add_node(5);
        println!("b1: {:?}", b1);
        b1.add_node(5);
        b1.add_node(4);
        println!("b1: {:?}", b1);
        let six = SparseNodeSet { nodes: vec![6_u8] };
        b1.remove_node(&six);
        println!("post removal: {:?}", b1);
        println!("try removing 6 again.");
        b1.remove_node(&six);
        println!("post removal: {:?}", b1);
    }

    #[test]
    fn test_serialization() {
        let b: SparseNodeSet<u32> = SparseNodeSet::from_slice(&[1, 2, 3]);
        let empty_basis: SparseNodeSet<u32> = SparseNodeSet::new_empty();
        let empty_s = serde_json::to_string(&empty_basis).expect("could not serialize emtpy.");
        println!("empty_s: {:?}", empty_s);
        let s = serde_json::to_string(&b).expect("could not serialize basis");
        dbg!(&s);
        let hm = HashMap::from([(b, 2)]);
        let s_hm = serde_json::to_string(&hm);
        dbg!(&s_hm);
        let vec: SparseNodeSet<u32> = serde_json::from_str(&s).unwrap();
        dbg!(vec);
        let empty: SparseNodeSet<u32> = serde_json::from_str(&"\"\"").expect("could not parse empty string");
        let single_node = "\"[1]\"";
        println!("input string: {:?}", single_node);
        let parsed: SparseNodeSet<u32> = serde_json::from_str(single_node).expect("where we parse at");
        println!("parsed: {:}", parsed);
    }
}
