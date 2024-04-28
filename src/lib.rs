//!# Matt's HyperGraph Library (mhgl)
//! A collection of three hypergraph structures
//! 1. A connectivity only structure `ConGraph`
//! 2. An option generic over the types stored in both the nodes and edges, and optionally you can specify what size of unsized integer to use to represent the node and edge ids.
//! 3. A key-value hypergraph where each node and edge allows you to store simple values modeled after a subset of the Polars data types. Both nodes and edges are assigned a `Uuid`. You can collect the data of a collection of `Uuid`s into a dataframe, with helpers for all nodes, all edges, and the entire hypergraph.
//!
//! The common behavior between these three structures is collected in the `HyperGraph` trait, which mostly consists of various ways of collecting "local" information about a node or a set of nodes within the hypergraph. For example you can query for the link of an edge or a set of nodes, the maximal edges that contain a given edge, or boundary up and down operators.
//!
//!
//!# Algorithms
//! Under Construction, currently we only have random walks (link, boundary_up * boundary_down, and boundary_down * boundary_up).
//!
//! # Alternative Hypergraph Libraries
//! - HyperNetX (Python): The most complete hypergraph library with algorithms for homology computations. Based on python and the underlying datastructure seems to be pandas arrays, so I would
//!   love to benchmark to compare this library with HyperNetX.
//! - HypergraphDB (Java): A database backend for storing and querying data, seems unmaintained but probably was ahead of its time.
//! - Hypergraph (Rust): Appears very limited in scope and not maintained.

#[forbid(unsafe_code)]
mod congraph;
mod edge;
mod hgraph;
mod hypergraph;
mod kvgraph;
mod node_trait;

pub use congraph::ConGraph;
pub use edge::EdgeSet;
pub use hgraph::HGraph;
pub use hypergraph::HyperGraph;
pub use kvgraph::KVGraph;
pub use node_trait::HgNode;
