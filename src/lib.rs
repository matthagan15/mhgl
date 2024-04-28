//!# Matt's HyperGraph Library (mhgl)
//!
//!# Algorithms
//! Under Construction.
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
