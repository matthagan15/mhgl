//!# Matt's HyperGraph Library (mhgl)
//! This crate provides a small number of hypergraph datastructures and some algorithms for working
//! with them. The datastructures provided are based on the most general possible hypergraph mathematically, that is a directed, weighted hypergraph. Directed and weighted hypergraphs are fairly different than the usually studied "undirected" hypergraph, in which case an edge is simply a subset of nodes. A directed and weighted hypergraph maps a subset of nodes to another subset of nodes with a specified edge weight. We also provide options for traditional "undirected" hypergraphs, which we refer to as "blob" type hyperedges, and prefer to use the term undirected to refer to a hyperedge which maps an input subset to an output subset and in addition can map the output subset to the input susbset.
//!
//! We provide the following three hypergraph variants:
//! 1. HGraph - Represents nodes as UUIDs that are randomly assigned. The easiest and most straightforward to use, Assumed to not receive the same UUID twice.
//! 2. PGraph<N> - A "performance" oriented version of HGraph that represents nodes as unsigned integers and is generic over which unsigned integer to use. (Untested) Should allow for smaller memory profiles than HGraph.
//! 3. BGraph [Under Construction] - Represents subsets of nodes using a binary encoding. Each node is assigned to a bit, so any subset of nodes can be represented using n bits. This is advantageous for dense hypergraphs on fewer nodes.
//!
//!
//! The main difference between these types of graphs are the basis elements used. A hypergraph basis element is represented with the trait `HgBasis` which captures the behavior of a subset (take unions, intersections, determine subsets, etc.) that is necessary for a use in hypergraph. See `SparseBasis` and `BitBasis` for implementation details.
//!
//! # Usage
//! Current status is that this is a minimal product and all you can really do
//! is create a graph, create nodes/edges, remove nodes/edges, query the graph,
//! and take steps starting from a specified basis. More useful behavior is
//! in progress. Here is an example of the API as is:
//! ```
//! use mhgl::{HGraph, EdgeDirection};
//! use uuid::Uuid;
//! let mut hg = HGraph::new();
//! let nodes = hg.create_nodes(10);
//! hg.create_edge(&nodes[0..2], &nodes[0..3], 1.23, EdgeDirection::Undirected);
//! let expected: Vec<(HashSet<Uuid>, EdgeWeight)> = vec![
//!    (nodes[0..3].iter().cloned().collect(), 1.23)
//! ];
//! assert_eq!(hg.step(&nodes[0..2]), expected);
//! ```
//!
//! # Algorithms
//! Under Construction.
//!
//! # Alternative Hypergraph Libraries
//! - HyperNetX (Python): focused on "blob" type hypergraphs.
//! - HypergraphDB (Java): A database backend for storing and querying data
//! - Hypergraph (Rust): Seems hard to use and not maintained.

#![forbid(unsafe_code)]

use uuid::Uuid;

pub mod algs;
mod bgraph;
mod hgraph;
mod pgraph;
mod structs;
mod traits;
mod utils;

pub use bgraph::BGraph;
pub use hgraph::HGraph;
pub use pgraph::PGraph;

pub use structs::EdgeDirection;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use uuid::Uuid;

    use crate::structs::NodeID;

    // use crate::{*, structs::EdgeWeight};

    #[test]
    fn hgraph_works() {
        use crate::{EdgeDirection, HGraph};
        let mut hg = HGraph::new();
        let nodes = hg.create_nodes(10);
        hg.create_edge(&nodes[0..2], &nodes[0..3], 1.23, EdgeDirection::Undirected);
        let expected: Vec<(HashSet<NodeID>, f64)> =
            vec![(nodes[0..3].iter().cloned().collect(), 1.23)];
        assert_eq!(hg.step(&nodes[0..2]), expected);
    }
}
