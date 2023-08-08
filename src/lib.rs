//!# Matt's HyperGraph Library (mhgl)
//! This crate provides a small number of hypergraph datastructures and some algorithms for working
//! with them. The datastructures provided are based on the most general possible hypergraph mathematically, that is a directed, weighted hypergraph. Directed and weighted hypergraphs are fairly different than the usually studied "undirected" hypergraph, in which case an edge is simply a subset of nodes. A directed and weighted hypergraph maps a subset of nodes to another subset of nodes with a specified edge weight. We also provide options for traditional "undirected" hypergraphs, which we refer to as "blob" type hyperedges, and prefer to use the term undirected to refer to a hyperedge which maps an input subset to an output subset and in addition can map the output subset to the input susbset.
//!
//! We provide the following three hypergraph variants with slightly different APIs:
//! 1. [`HGraph`](crate::hgraph::HGraph) HGraph - Represents nodes as UUIDs that are randomly assigned. The easiest and most straightforward to use.
//! 2. PGraph<N> - A "performance" oriented version of HGraph that represents nodes as unsigned integers and is generic over which unsigned integer to use. Due to smaller encoding of node IDs adding nodes may fail.
//! 3. BGraph - Represents subsets of nodes using a binary encoding. Each node is assigned to a bit, so any subset of nodes can be represented using an n bit string. This is advantageous for dense hypergraphs on fewer nodes.
//!
//! Each of these variants implement their own methods for interacting (such as
//! create node, step, etc) but commonalities are pulled into a provided `HyperGraph` trait
//! that captures basic functionality. See the trait docs
//! for more details.
//!
//! # Basis Elements
//! The main difference between these types of graphs are the "basis elements" used. By a "basis element" we are simply referring to a subset of nodes. There are different ways of storing such a subset that is more useful than simply using a `HashSet` (which is not amenable to storage within `HashMap`s). A hypergraph basis element is represented with the trait `HgBasis` which captures the behavior of a subset (take unions, intersections, determine subsets, etc.) that is necessary for a use in hypergraph. See `SparseBasis` and `BitBasis` for the two main implementations used, `SparseBasis` is used for `HGraph` and `PGraph` while `BitBasis` is used for `BGraph`. These are the objects returned from `HyperGraph` trait methods.
//!
//! # Usage
//! Current status is that this is a minimal product and all you can really do
//! is create a graph, create nodes/edges, remove nodes/edges, query the graph,
//! and take steps starting from a specified basis. More useful behavior is
//! in progress. Here is an example of the API using `HGraph`:
//! ```
//! use mhgl::{HGraph, EdgeDirection};
//!
//! // Create a graph with 10 nodes and 1 edge
//! let mut hg = HGraph::new();
//! let nodes = hg.create_nodes(10);
//! hg.create_edge(&nodes[0..2], &nodes[0..3], 1.23, EdgeDirection::Undirected);
//!
//! // Steps work as expected
//! let expected: Vec<(HashSet<u128>, EdgeWeight)> = vec![
//!    (nodes[0..3].iter().cloned().collect(), 1.23)
//! ];
//! assert_eq!(hg.step(&nodes[0..2]), expected);
//! ```
//!
//! And similar example with `PGraph` and `BGraph`
//!
//! ```
//! use mhgl::{PGraph, EdgeDirection};
//!
//! let mut hg = PGraph::<u8>::new();
//! let nodes = hg.create_nodes(255).expect("Still space left.");
//! assert_eq!(hg.create_nodes(1), None);
//! assert!(nodes.contains(42));
//!
//! // Currently requires the number of nodes to stay fixed due to resizing
//! // of bitvecs.
//! let mut bin_hg = BGraph::new(20);
//!
//! // Nodes accessed with zero-indexing simply by number (usize).
//! bin_hg.create_edge(&[1,2,3], &[], 3., EdgeDirection::Loop);
//! ```
//!
//! # Algorithms
//! Under Construction.
//!
//! # Alternative Hypergraph Libraries
//! - HyperNetX (Python): focused on "blob" type hypergraphs.
//! - HypergraphDB (Java): A database backend for storing and querying data
//! - Hypergraph (Rust): Appears very limited in scope and not maintained.

#![forbid(unsafe_code)]

pub mod algs;
mod graph;
mod bgraph;
mod dgraph;
mod pgraph;
mod hgraph;
mod stackgraph;
mod structs;
mod traits;
mod utils;

pub use graph::Graph;
pub use bgraph::BGraph;
pub use dgraph::DGraph;
pub use pgraph::PGraph;
pub use hgraph::HGraph;
pub use structs::BitBasis;
pub use structs::EdgeDirection;
pub use structs::SparseBasis;

pub use traits::HgBasis;
pub use traits::HyperGraph;

#[cfg(test)]
mod tests {
    use crate::structs::NodeID;
    use std::collections::HashSet;

    #[test]
    fn hgraph_works() {
        use crate::{EdgeDirection, DGraph};
        let mut hg = DGraph::new();
        let nodes = hg.create_nodes(10);
        hg.create_edge(&nodes[0..2], &nodes[0..3], 1.23, EdgeDirection::Symmetric);
        let expected: Vec<(HashSet<u128>, f64)> =
            vec![(nodes[0..3].iter().cloned().collect(), 1.23)];
        assert_eq!(hg.step(&nodes[0..2]), expected);
    }
}
