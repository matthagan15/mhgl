use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};

use crate::HgNode;

/// A subset for an overall set system, note that for things like
/// deserializing and using `From`'s we default always to `Undirected`, so
/// if you want to make a `Edge::Simplex` from a `Vec` you have to do something like
/// ```
/// let vec = vec![1, 2, 3];
/// let simplex = Edge::from(vec).make_simplex();
/// ```
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct EdgeSet<N: HgNode>(pub Vec<N>);

impl<N: HgNode> EdgeSet<N> {
    /// Creates an empty edge.
    pub fn new() -> Self {
        EdgeSet(Vec::new())
    }

    /// Number of nodes in the edge
    /// ```rust
    /// let e = Edge::from([1,2,3]);
    /// assert_eq!(e.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_node(&self) -> bool {
        self.len() == 1
    }

    pub fn get_first_node(&self) -> Option<N> {
        self.0.first().cloned()
    }

    pub fn node_set(&self) -> HashSet<N> {
        self.0.clone().into_iter().collect()
    }

    pub fn node_vec(&self) -> Vec<N> {
        self.0.clone().into_iter().collect()
    }

    pub fn to_node_set(self) -> HashSet<N> {
        self.0.into_iter().collect()
    }

    pub fn to_node_vec(self) -> Vec<N> {
        self.0
    }

    pub fn add_node(&mut self, node: N) {
        if let Err(ix) = self.0.binary_search(&node) {
            self.0.insert(ix, node);
        }
    }

    pub fn contains_node(&self, node: &N) -> bool {
        if self.0.len() == 0 {
            return false;
        }
        let mut ret = false;
        if let Ok(ix) = self.0.binary_search(node) {
            if self.0[ix] == *node {
                ret = true;
            }
        }
        ret
    }

    pub fn intersect_with(&mut self, rhs: &Self) {
        let mut good_nodes = Vec::with_capacity(self.0.len());
        for _ in 0..self.0.len() {
            if let Some(node) = self.0.pop() {
                if rhs.contains_node(&node) {
                    good_nodes.insert(0, node);
                }
            }
        }
        self.0.clear();
        self.0.append(&mut good_nodes);
    }

    pub fn intersection(&self, rhs: &Self) -> Self {
        let mut ret = Vec::new();
        let l1 = self.0.len();
        let l2 = rhs.0.len();
        let mut left_counter = 0;
        let mut right_counter = 0;
        for _ix in 0..(l1 + l2) {
            if left_counter == l1 || right_counter == l2 {
                break;
            }
            if self.0[left_counter] < rhs.0[right_counter] {
                left_counter += 1;
            } else if self.0[left_counter] > rhs.0[right_counter] {
                right_counter += 1;
            } else {
                ret.push(self.0[left_counter].clone());
                left_counter += 1;
                right_counter += 1;
            }
        }
        EdgeSet(ret)
    }

    pub fn union_with(&mut self, rhs: &Self) {
        for node in rhs.0.iter() {
            self.add_node(*node);
        }
    }

    // TODO: This is pretty unoptimal but it works.
    pub fn union(&self, rhs: &Self) -> Self {
        let mut tot = HashSet::new();
        for node in self.0.iter() {
            tot.insert(*node);
        }
        for node in rhs.0.iter() {
            tot.insert(*node);
        }
        let mut union: Vec<N> = tot.into_iter().collect();
        union.sort();
        EdgeSet(union)
    }

    /// If `rhs` is contained in self, returns the complement of rhs
    /// within self. If `rhs` is not fully contained in self returns the
    /// empty set. Note this is the same return value as `self.link(&self)`.
    /// This could lead to major bugs in the future, but the other option
    /// is that link returns an option.
    pub fn link(&self, rhs: &Self) -> Option<Self> {
        if self.contains_strict(rhs) == false {
            return None;
        }
        let mut ret_nodes = self.node_set();
        for node in rhs.0.iter() {
            ret_nodes.remove(node);
        }
        let nodes_vec: Vec<N> = ret_nodes.into_iter().collect();
        Some(EdgeSet(nodes_vec))
    }

    pub fn remove_node(&mut self, node: &N) {
        if let Ok(ix) = self.0.binary_search(node) {
            self.0.remove(ix);
        }
    }

    pub fn remove_nodes(&mut self, nodes: &Vec<N>) {
        for node in nodes.iter() {
            self.remove_node(node);
        }
        self.0.shrink_to_fit();
    }

    /// This is equivalent to self \subseteq other
    pub fn contains(&self, other: &Self) -> bool {
        let mut left_ix = 0;
        let mut right_ix = 0;
        while right_ix < other.0.len() {
            if left_ix == self.0.len() {
                return false;
            }
            if self.0[left_ix] == other.0[right_ix] {
                right_ix += 1;
            } else {
                left_ix += 1;
            }
        }
        true
    }
    pub fn contains_strict(&self, other: &Self) -> bool {
        let mut left_ix = 0;
        let mut right_ix = 0;
        while right_ix < other.0.len() {
            if left_ix == self.0.len() {
                return false;
            }
            if self.0[left_ix] == other.0[right_ix] {
                right_ix += 1;
            } else {
                left_ix += 1;
            }
        }
        left_ix < self.0.len() - 1
    }
}

impl<N: HgNode> Serialize for EdgeSet<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0.len() == 0 {
            return serializer.serialize_str("[]");
        }
        let mut s = String::new();
        s.push_str("[");
        for node in self.0.iter() {
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
            Ok(EdgeSet(v))
        } else {
            if let Ok(n) = data.parse::<N>() {
                Ok(EdgeSet(vec![n]))
            } else {
                if data.len() == 0 {
                    Ok(EdgeSet(vec![]))
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
        let mut s = String::new();
        s.push_str("[");
        for node in self.0.iter() {
            s.push_str(&format!("{:?},", node));
        }
        s.truncate(s.len() - 1);
        s.push_str("]");
        f.write_str(&s)
    }
}

impl<N: HgNode, R: AsRef<[N]>> From<R> for EdgeSet<N> {
    fn from(value: R) -> Self {
        let ref_value = value.as_ref();
        let mut nodes: Vec<N> = ref_value.iter().cloned().collect();
        nodes.sort();
        nodes.dedup();
        EdgeSet(nodes)
    }
}

mod test {
    use std::collections::{HashMap, HashSet};

    use super::EdgeSet;

    #[test]
    fn test_contains() {
        let e1 = EdgeSet::from([1_u8, 2, 3, 4]);
        let e2 = EdgeSet::from([1_u8, 2, 3]);
        let e3 = EdgeSet::from([0_u8, 7, 9]);
        let e4 = EdgeSet::from([1_u8, 2, 3, 4]);
        assert!(e1.contains(&e2));
        assert!(e1.contains_strict(&e2));
        assert!(!e1.contains_strict(&e4));
        assert!(!e2.contains(&e1));
        assert!(!e1.contains(&e3));
        assert!(!e3.contains(&e1));
    }

    #[test]
    fn conversions() {
        let node_vec = vec![1_u8, 2, 3];
        let node_set = HashSet::from([1_u8, 2, 3]);
        let node_arr = [1_u8, 2, 3];
        let e1 = EdgeSet::from(&node_vec);
        let e2 = EdgeSet::from(&node_vec[..]);
        let e3 = EdgeSet::from(node_vec);
        let e4 = EdgeSet::from(node_arr.iter());
        let e5 = EdgeSet::from(node_arr);
        assert_eq!(e1, e2);
        assert_eq!(e1, e3);
        assert_eq!(e1, e4);
        assert_eq!(e1, e5);
    }
}
