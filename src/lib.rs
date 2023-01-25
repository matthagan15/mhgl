//!# Matt's Hypergraph Library (MHL)
//! I initially called this project "graphene" before I found out that name was taken. Both "hyperion" and "graphite", my two backup names, were
//! also taken. So instead of wasting time trying to think of a cool name (hypergraphene seemed too verbose) I've decided to just put this out there.
//! 
//! This is a basic library intended to provide a few hypergraph datastructures and some algorithms for them.
//! The datastructures I plan on initially supporting are 
//! - 1 A sparse implementation
//! - 2 A bit-wise encoding, which would work well for more dense hypergraphs.
//! - 3 A matrix encoding, in which the hypergraph is represented as a graph in a higher dimensional space. This is intended for "nearly full" hypergraphs.
//! 
//! Currently 1 is the most developed, 2 is underway, and 3 has not been started but I think should be the fastest to implement. I have yet to isolate "hypergraph"-ness
//! into a trait to make interacting with these 2 (3) structures more coherent. Once these structures are more fleshed out and the API is more cohesive then a trait should
//! be added. 
//! 
//! Note that none of these structures are intended to be used out-of-the-box as databases, so they do not support storing data in nodes or edges.
//! This is typically done in some graph libraries through the use of generic types (for example a base `Node<N>` class allows you to put data of type in `N` in each node). I 
//! personally find this to be an instance of over-generalization on a library part and makes ergonomics of using the basic structures more annoying.
//! I am also working on a basic hypergraph database system that would allow you to do this. I will open source that project if it seems appropriate.
//! 
//! 
//! As hypergraphs are relatively understudied compared to their ancestors (graphs), the algorithms will be a bit underexplored and will be filled out as
//! the field progresses. I currently plan on adding basic stuff like traversal, cut computations, and random walks first. Another first task is simple constructors,
//! such as erdos-renyi, complete, or k-uniform hypergraphs.

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
