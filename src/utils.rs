use std::{collections::HashSet, ops::Add, u8};
use uuid::Uuid;

use crate::traits::*;

/// Returns subset of power set of given id's with the condition that each set returned has dims number
/// of elements.
pub fn power_set(v: Vec<Uuid>, dims: usize) -> HashSet<Vec<Uuid>> {
    if dims == 0 {
        return HashSet::new();
    }
    if dims == 1 {
        return HashSet::from([v]);
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
struct PowerSetBits {
    bits: Vec<u8>,
    length_needed: usize,
}
impl PowerSetBits {
    fn new() -> PowerSetBits {
        PowerSetBits {
            bits: Vec::new(),
            length_needed: 0,
        }
    }
    fn from_length(length: usize) -> PowerSetBits {
        let mut v = Vec::with_capacity(length / 8 + 1);
        v.fill(0 as u8);
        PowerSetBits {
            bits: v,
            length_needed: length,
        }
    }
}

impl Add<usize> for PowerSetBits {
    type Output = PowerSetBits;

    fn add(self, rhs: usize) -> Self::Output {
        todo!()
    }
}

pub fn lazy_power_set<N: HgNode>(v: Vec<N>, prob_keep: f64) {
    let mut bits = PowerSetBits::from_length(v.len());
}

mod test {
    use uuid::Uuid;

    use super::power_set;

    #[test]
    fn test_power_set() {
        let ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        println!("power set:\n{:?}", power_set(ids, 2));
    }
}
