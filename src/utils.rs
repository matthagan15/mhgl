use std::collections::HashSet;
use uuid::Uuid;

/// Returns subset of power set of given id's with the condition that each set returned has dims number
/// of elements. 
pub fn power_set(v: Vec<Uuid>, dims: usize) -> HashSet<Vec<Uuid>> {
    if dims == 0 {
        return HashSet::new();
    }
    if dims == 1 {
        return v.into_iter().map(|node| vec![node]).collect();
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

mod test {
    use uuid::Uuid;

    use super::power_set;

    #[test]
    fn test_power_set() {
        let ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        println!("power set:\n{:?}", power_set(ids, 2));
    }
}