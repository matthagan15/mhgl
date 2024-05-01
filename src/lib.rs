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
//! let nodes = cg.add_nodes(5);
//! let mut edges = Vec::new();
//! for ix in 1..nodes.len() {
//!     let edge = cg.add_edge(&nodes[0..=ix]);
//!     edges.push(edge);
//! }
//! let maxs_of_edge = cg.maximal_edges(&edges[0]);
//! let maxs_of_nodes = cg.maximal_edges_of_nodes([0, 1, 2]);
//!
//! assert_eq!(maxs_of_edge[0], edges[edges.len() - 1]);
//! assert_eq!(maxs_of_nodes[0], edges[edges.len() - 1]);
//! assert_eq!(cg.boundary_up(&edges[0]), vec![edges[1]]);
//!
//! #[derive(Debug)]
//! struct Foo(u8);
//!
//! #[derive(Debug)]
//! struct Bar(u32);
//!
//! let mut hg = HGraph::<Foo, Bar>::new();
//! let n0 = hg.add_node(Foo(1));
//! let n1 = hg.add_node(Foo(2));
//! let e = hg.add_edge(&[n0, n1], Bar(42)).unwrap();
//! let e_mut = hg.borrow_edge_mut(&e).unwrap();
//! e_mut.0 = 12;
//! let bar = hg.remove_edge(e).unwrap();
//! assert_eq!(bar.0, 12);
//!
//! let mut kvgraph = KVGraph::new();
//! let n0 = kvgraph.add_node_with_label("toronto");
//! let n1 = kvgraph.add_node_with_label("seattle");
//! let edge = kvgraph.add_edge_with_label(&[n0, n1], "AC123").unwrap();
//! kvgraph.insert(&n0, "darkness", 0.6);
//! kvgraph.insert(&n1, "darkness", 0.8);
//! let df = kvgraph.dataframe();
//! println!("{:}", df);
//! ```
//! The last line in the above code will output:
//! ```text
//!┌────────────┬───────────────────────────────────┬───────────────────────────────────┬───────────────────┬──────────┐
//! │ label      ┆ id                                ┆ nodes                             ┆ labelled_nodes    ┆ darkness │
//! │ ---        ┆ ---                               ┆ ---                               ┆ ---               ┆ ---      │
//! │ str        ┆ str                               ┆ str                               ┆ str               ┆ f64      │
//! ╞════════════╪═══════════════════════════════════╪═══════════════════════════════════╪═══════════════════╪══════════╡
//! │ toronto    ┆ 6347a42e-0bde-4d80-aad3-7e8c59d3… ┆ [6347a42e-0bde-4d80-aad3-7e8c59d… ┆ [toronto]         ┆ 0.6      │
//! │ seattle    ┆ 032e1a16-ec39-4045-8ebd-381c2b06… ┆ [032e1a16-ec39-4045-8ebd-381c2b0… ┆ [seattle]         ┆ 0.8      │
//! │ AC123      ┆ 1b233128-22d2-4158-850d-b4b814d5… ┆ [1b233128-22d2-4158-850d-b4b814d… ┆ [seattle,toronto] ┆ null     │
//! └────────────┴───────────────────────────────────┴───────────────────────────────────┴───────────────────┴──────────┘
//! ```
//! Currently data schema is shared between nodes and edges, which is
//! unfortunate.
//!
//!# Algorithms
//! Mostly under construction, currently there is only a simple random walk either using link,
//! boundary_up * boundary_down, and boundary_down * boundary_up to determine the next subset to move to. I plan to
//! port some algorithms, such as the connected components, s_walk, and homology algorithms from `HyperNetX` to this library over time.
//!
//! This library should be considered as an **alpha** version. Here are a few hypergraph libraries I found, the most mature of which is HyperNetX developed by Pacific Northwest National Laboratory (PNNL).
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

mod tests {
    #[cfg(feature = "polars")]
    #[cfg(feature = "uuid")]
    #[test]
    fn simple_dataframe() {
        use crate::KVGraph;

        let mut kvgraph = KVGraph::new();
        let n0 = kvgraph.add_node_with_label("toronto");
        let n1 = kvgraph.add_node_with_label("seattle");
        let edge = kvgraph.add_edge_with_label(&[n0, n1], "AC123").unwrap();
        kvgraph.insert(&n0, "darkness", 0.6);
        kvgraph.insert(&n1, "darkness", 0.8);
        let df = kvgraph.dataframe();
        println!("{:}", df);
    }
}
