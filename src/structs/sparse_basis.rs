use std::{
    collections::HashSet,
    fmt::{Debug, Display},
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
pub enum EdgeSet<N: HgNode> {
    Undirected(Vec<N>),
    Simplex(Vec<N>),
}

impl<N: HgNode> EdgeSet<N> {
    /// Creates an empty edge.
    pub fn new(simplex: bool) -> Self {
        if simplex {
            EdgeSet::Simplex(Vec::new())
        } else {
            EdgeSet::Undirected(Vec::new())
        }
    }

    pub fn is_simplex(&self) -> bool {
        match self {
            EdgeSet::Undirected(_) => false,
            EdgeSet::Simplex(_) => true,
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
            EdgeSet::Undirected(nodes) => EdgeSet::Simplex(nodes),
            EdgeSet::Simplex(_) => self,
        }
    }

    /// Consumes `self`
    pub fn make_undirected(self) -> Self {
        match self {
            EdgeSet::Undirected(_) => self,
            EdgeSet::Simplex(nodes) => EdgeSet::Undirected(nodes),
        }
    }

    pub fn node_set(&self) -> HashSet<N> {
        match self {
            EdgeSet::Undirected(nodes) => nodes.clone().into_iter().collect(),
            EdgeSet::Simplex(nodes) => nodes.clone().into_iter().collect(),
        }
    }

    pub fn node_vec(&self) -> Vec<N> {
        match self {
            EdgeSet::Undirected(nodes) => nodes.clone().into_iter().collect(),
            EdgeSet::Simplex(nodes) => nodes.clone().into_iter().collect(),
        }
    }

    pub fn to_node_set(self) -> HashSet<N> {
        match self {
            EdgeSet::Undirected(nodes) => nodes.into_iter().collect(),
            EdgeSet::Simplex(nodes) => nodes.into_iter().collect(),
        }
    }

    pub fn to_node_vec(self) -> Vec<N> {
        match self {
            EdgeSet::Undirected(nodes) => nodes.into_iter().collect(),
            EdgeSet::Simplex(nodes) => nodes.into_iter().collect(),
        }
    }

    pub fn add_node(&mut self, node: N) {
        match self {
            EdgeSet::Undirected(nodes) => {
                if let Err(ix) = nodes.binary_search(&node) {
                    nodes.insert(ix, node);
                }
            }
            EdgeSet::Simplex(nodes) => {
                if let Err(ix) = nodes.binary_search(&node) {
                    nodes.insert(ix, node);
                }
            }
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
            EdgeSet::Undirected(nodes) => searcher(nodes),
            EdgeSet::Simplex(nodes) => searcher(nodes),
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
            EdgeSet::Undirected(_) => EdgeSet::Undirected(ret),
            EdgeSet::Simplex(_) => EdgeSet::Simplex(ret),
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
            EdgeSet::Undirected(nodes) => nodes,
            EdgeSet::Simplex(nodes) => nodes,
        }
    }

    pub fn nodes_ref_mut<'a>(&'a mut self) -> &'a mut Vec<N> {
        match self {
            EdgeSet::Undirected(nodes) => nodes,
            EdgeSet::Simplex(nodes) => nodes,
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
        let mut union: Vec<N> = tot.into_iter().collect();
        union.sort();
        match self {
            EdgeSet::Undirected(_) => EdgeSet::Undirected(union),
            EdgeSet::Simplex(_) => EdgeSet::Simplex(union),
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
            EdgeSet::Undirected(_) => Some(EdgeSet::from(nodes_vec)),
            EdgeSet::Simplex(_) => Some(EdgeSet::from(nodes_vec).make_simplex()),
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

impl<N: HgNode> Serialize for EdgeSet<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let nodes = match self {
            EdgeSet::Undirected(nodes) => nodes,
            EdgeSet::Simplex(nodes) => nodes,
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

impl<'de, N: HgNode> Deserialize<'de> for EdgeSet<N> {
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
            Ok(EdgeSet::Undirected(v))
        } else {
            if let Ok(n) = data.parse::<N>() {
                Ok(EdgeSet::Undirected(vec![n]))
            } else {
                if data.len() == 0 {
                    Ok(EdgeSet::Undirected(vec![]))
                } else {
                    println!("Data: {:?}", data);
                    panic!("Could not parse single input.");
                }
            }
        }
    }
}

impl<N: HgNode> Display for EdgeSet<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nodes = match self {
            EdgeSet::Undirected(nodes) => nodes,
            EdgeSet::Simplex(nodes) => nodes,
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

impl<N: HgNode> From<Vec<N>> for EdgeSet<N> {
    fn from(value: Vec<N>) -> Self {
        let mut nodes = value;
        nodes.sort();
        nodes.dedup();
        EdgeSet::Undirected(nodes)
    }
}

impl<N: HgNode> From<&[N]> for EdgeSet<N> {
    fn from(value: &[N]) -> Self {
        let v: Vec<N> = value.iter().cloned().collect();
        EdgeSet::from(v)
    }
}

impl<N: HgNode, const K: usize> From<[N; K]> for EdgeSet<N> {
    fn from(value: [N; K]) -> Self {
        EdgeSet::from(value.to_vec())
    }
}

impl<N: HgNode> From<HashSet<N>> for EdgeSet<N> {
    fn from(value: HashSet<N>) -> Self {
        let v: Vec<N> = value.into_iter().collect();
        EdgeSet::from(v)
    }
}

mod test {
    use std::collections::HashMap;

    use crate::HgBasis;

    use super::EdgeSet;

    #[test]
    fn test_contains() {
        let e1 = EdgeSet::from([1_u8, 2, 3, 4]);
        let e2 = EdgeSet::from([1_u8, 2, 3]);
        let e3 = EdgeSet::from([0_u8, 7, 9]);
        assert!(e1.contains(&e2));
        assert!(!e2.contains(&e1));
        assert!(!e1.contains(&e3));
        assert!(!e3.contains(&e1));
    }
}
