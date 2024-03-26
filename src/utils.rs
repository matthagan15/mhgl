use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    u8,
};

use crate::traits::*;

/// Returns subset of power set of given id's with the condition that each set returned has dims number
/// of elements.
pub fn power_set<N: HgNode>(v: Vec<N>, dims: usize) -> HashSet<Vec<N>> {
    if dims == 0 {
        return HashSet::new();
    }
    if dims == 1 {
        return HashSet::from([Vec::new(), v]);
    }
    if dims > v.len() {
        let l = v.len();
        return power_set(v, l);
    }
    let copy = v.clone();
    let smallers = power_set(copy, dims - 1);
    let mut ret = HashSet::new();
    for node in v {
        for mut smaller in smallers.clone() {
            if smaller.contains(&node) == false {
                smaller.push(node.clone());
                smaller.sort();
                ret.insert(smaller);
            }
        }
    }
    ret
}