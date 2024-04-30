//!# Matt's HyperGraph Library (mhgl)
//!
//! A small library that provides three undirected hypergraph structures:
//! 1.`ConGraph` - a connectivity only option that uses `u32`'s as IDs for
//! nodes and `u64`'s for edge IDs. No data that can be stored within the
//! `ConGraph` structure itself and NodeIDs and EdgeIDs are simple incremented
//! counters started at 0.
//! 2. `HGraph` - An option generic over the types stored in both the nodes and
//! edges. Additionally generic over the size of integers `u8` through `u128`
//! to store NodeIDs and EdgeIDs with `u32` and `u64` as the default for the respective IDs.
//! 3. `KVGraph` - A key-value hypergraph where each node and edge allows you
//! to store simple values modeled after a simple subset of the Polars data
//! types. There are two features for this crate, "uuid" which is necessary to use the
//! `KVGraph` struct as `Uuid`s are used for both node and edge ids and "polars"
//! is necessary if you want to retreive dataframes out of the hypergraph.
//!
//! `ConGraph` and `KVGraph` are essentially wrappers around `HGraph` with
//! slightly tweaked apis for adding and deleting nodes or edges (for example
//! you don't need to provide data for adding nodes to a `ConGraph` but you do
//! for `HGraph`).
//!
//! The common behavior between these three structures is collected in the
//! `HyperGraph` trait, which mostly consists of various ways of collecting
//! "local" information about a node or a set of nodes within the hypergraph.
//! With a `HyperGraph` object you can query for the link of an edge or a set
//! of nodes, the maximal edges that contain a given edge, or the action of
//! boundary up and down operators on an edge or set of nodes. Consistent
//! throughout the trait is the ability to interact with a `HyperGraph` either
//! through edge ids or any type that can be
//! cast `AsRef` into a slice &[NodeID]. Whenever a slice or such is passed the
//! hypergraph will clone the NodeIDs, sort, and make sure no duplicates exist.
//! The only other trait in the crate for now is the `HgNode` trait which is
//! used to mark the types suitable for `HGraph`.
//!
//! ```rust
//! use mhgl::*;
//! let mut cg = ConGraph::new();
//! let n0 = cg.add_node();
//! assert_eq!(n0, 0);
//! ```
//!
//!# Algorithms
//! Mostly under construction, currently we only have random walks (link,
//! boundary_up * boundary_down, and boundary_down * boundary_up). I plan to
//! port some algorithms, such as the connected components, s_walk, and homology algorithms from `HyperNetX` to this library over time.
//!
//! # Alternative Hypergraph Libraries
//! - HyperNetX (Python): The most complete hypergraph library with algorithms
//! for homology computations. Based on python and the underlying datastructure
//! seems to be pandas arrays.
//! - HypergraphDB (Java): A database backend for storing and querying data, seems unmaintained but probably was ahead of its time.
//! - Hypergraph (Rust): Appears very limited in scope and not maintained.

#[forbid(unsafe_code)]
mod algs;
mod congraph;
mod edge;
mod hgraph;
mod hypergraph;
#[cfg(feature = "uuid")]
mod kvgraph;
mod node_trait;

pub use congraph::ConGraph;
pub use edge::EdgeSet;
pub use hgraph::HGraph;
pub use hypergraph::HyperGraph;
#[cfg(feature = "uuid")]
pub use kvgraph::KVGraph;

pub use node_trait::HgNode;
