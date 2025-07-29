#![forbid(unsafe_code)]
//!# Matt's HyperGraph Library (mhgl)
//!
//! A library for working with undirected [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph). An undirected hypergraph consists of a set of nodes and a collection of subsets of these nodes, a normal undirected "graph" is therefore a hypergraph where each subset only has 2 nodes. The most generic datatype provided by this library is the [`HGraph`](`crate::HGraph`) which allows for you to specify the data type used to store
//! both Node and Edge ID's (from any unsigned integer type) and is generic over the data type stored with each node and edge.
//!
//! Here is a sample usage of adding nodes, edges, computing containing and
//! maximal containing edges, links, and boundaries. The hypergraph specific
//! functions are collected in the [`HyperGraph`](`crate::HyperGraph`) trait and shared with the
//! connectivity only datastructure [`ConGraph`](`crate::ConGraph`) and the generic key-value structure [`KVGraph`](`crate::KVGraph`).
//! ```rust
//! let mut hg = HGraph::<i32, String>::new();
//! let a = hg.add_node(-1);
//! let b = hg.add_node(3);
//! let c = hg.add_node(5);
//! let d = hg.add_node(7);
//!
//! let e1 = hg.add_edge([a, b], "one".to_string());
//! let e2 = hg.add_edge([a, b, c], "two".to_string());
//! let e3 = hg.add_edge([a, b, c, d], "three".to_string());
//!
//! let found_id = hg.find_id([a, b]);
//! let found_string = found_id.map(|id| hg.get_edge(&id)).flatten();
//! assert_eq!(found_string, Some(&String::from("one")));
//! assert_eq!(None, hg.find_id([a]));
//!
//! let mut containing_edges = hg.containing_edges(&e1);
//! containing_edges.sort();
//! assert_eq!(containing_edges, vec![e2, e3]);
//!
//! let mut containing_edges = hg.containing_edges_of_nodes([a]);
//! containing_edges.sort();
//! assert_eq!(containing_edges, vec![e1, e2, e3]);
//!
//! let max_edge1 = hg.maximal_edges(&e1);
//! let max_edge2 = hg.maximal_edges(&e2);
//! assert_eq!(max_edge1.first(), Some(&e3));
//! assert_eq!(max_edge2.first(), Some(&e3));
//!
//! let mut link = hg.link(&e1);
//! link.sort_by_key(|x| x.0);
//! assert_eq!(link, vec![(e2, vec![c]), (e3, vec![c, d])]);
//!
//! // Boundaries currently only work on "basis sets", or
//! // single edges in the graph. They do not function as
//! // linear operators acting on linear combinations of
//! // edges yet.
//! let boundary_down = hg.boundary_down_of_nodes([a, b, c]);
//! let boundary_down_id = hg.find_id(&boundary_down[0][..]);
//! assert_eq!(boundary_down_id, Some(e1));
//!
//! let mut boundary_single_nodes = hg.boundary_down_of_nodes([a, b]);
//! boundary_single_nodes.sort();
//! assert_eq!(boundary_single_nodes, vec![vec![a], vec![b]]);
//!
//! let boundary_up = hg.boundary_up_of_nodes([a, b, c]);
//! let boundary_up_id = hg.find_id(&boundary_up[0][..]);
//! assert_eq!(boundary_up_id, Some(e3));
//! ```
//!
//!
//! # Hypergraph Structs
//! - [`ConGraph`](`crate::ConGraph`) - a connectivity only option that uses `u32`'s as IDs for
//! nodes and `u64`'s for edge IDs with each being a simple counter starting at 0. No data that can be stored within the
//! `ConGraph` structure itself. Has simpler API if all you need to analyze is
//! connectivity.
//! -[`HGraph`](`crate::HGraph`) - A struct generic over four types: the node data, the edge data, the node IDs, and the edge IDs. There are no trait bounds on the node and edge types. Additionally generic over the size of integers `u8` through `u128`
//! to store NodeIDs and EdgeIDs with `u32` and `u64` as the default for the respective IDs.
//! Utilizes an adjacency-list storage system.
//! - [`KVGraph`](`crate::KVGraph`) - A key-value hypergraph where each node and edge allows you
//! to store simple [`kvgraph::Value`](`crate::kvgraph::Value`)s (of type [`kvgraph::ValueTypes`](`crate::kvgraph::ValueTypes`)) modeled after a simple subset of the Polars `AnyValue<'a>`.
//!
//! `ConGraph` and `KVGraph` are essentially wrappers around `HGraph` with
//! slightly tweaked function signatures for adding and deleting nodes or edges
//! (for example
//! you don't need to provide data for adding nodes to a `ConGraph` but you do
//! for `HGraph`).
//!
//! # Example
//! ```rust
//! use mhgl::*;
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
//! This print statement gives the following table
//! ```
//! shape: (5, 8)
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
//! # Features
//! There are 2 features related to the [`KVGraph`](`crate::kvgraph`) module
//! - **"uuid"** to enable the use of [`KVGraph`] as it uses `Uuid`s as the ID
//! type for both nodes and edges.
//! - **"polars"** to compute [`polars`](https://www.pola.rs) dataframes of
//! any collection of nodes or edges.
//!
//! # Traits
//! - [`HyperGraph`](`crate::HyperGraph`) - A collection of functions for querying the adjacency
//! structure of a hypergraph. There are a few main functions, each of which
//! takes as an input an edge ID and returns related edges in the hypergraph.
//! Each function also has an "of_nodes" variant which allows you to find the
//! same info but instead of requiring an input edge of the hypergraph you can
//! provide a slice of nodes.
//!     - [`containing_edges`](`HyperGraph::containing_edges`) finds all edges which are strict supersets of the input edge.
//!     - [`maximal_edges`](`HyperGraph::maximal_edges`) finds all edges containing the input edge that are not themselves contained in another edge.
//!     - [`link`](`HyperGraph::link`) takes all edges which contain the given edge and computes the complement of the input within that edge.
//!     - [`boundary_up`](`HyperGraph::boundary_up`) the boundary up operator comes from topology and the terminology of simplicial complexes. It takes the input edge and finds all edges that are only a single extra node added to the input.
//!     - [`boundary_down`](`HyperGraph::boundary_down`) similar to the `boundary_up` operator but removes a node.
//!
//! - [`HgNode`](`crate::HgNode`) - A marker trait for indicating which types are usuable for
//! node and edge IDs (spoiler: `u8`, `u16, `u32`, `u64`, and `u132`. Don't use `Uuid`s even though they implement the trait.)
//!
//! # Alternative Hypergraph Libraries
//! This library should be considered as an **alpha** version. Here are a few
//! hypergraph libraries I found, the most mature of which is HyperNetX
//! developed by Pacific Northwest National Laboratory (PNNL).
//! - [HyperNetX](https://pnnl.github.io/HyperNetX/) (Python): The most complete hypergraph library with algorithms
//! for homology computations. Based on python and the underlying datastructure
//! seems to be pandas arrays.
//! - [Gudhi](https://gudhi.inria.fr/index.html) (C++): This library is focused on computing persistent homology bargraphs. As such it has datastructures for simplicial complexes and more.
//! - [HypergraphDB](https://hypergraphdb.org/) (Java): A database backend for storing and querying data, seems unmaintained.
//! - [Hypergraph](https://crates.io/crates/hypergraph) (Rust): Seemed limited in scope and a bit complicated to me.

mod congraph;
mod edge;
mod hgraph;
mod hypergraph;
mod node_trait;

pub use congraph::ConGraph;
pub use edge::EdgeSet;
pub use hgraph::HGraph;
pub use hypergraph::HyperGraph;

#[cfg(feature = "uuid")]
pub mod kvgraph;
#[cfg(feature = "uuid")]
pub use kvgraph::KVGraph;

pub use node_trait::HgNode;

#[cfg(test)]
mod tests {
    #[cfg(feature = "polars")]
    #[cfg(feature = "uuid")]
    #[test]
    fn simple_dataframe() {
        use crate::KVGraph;

        let mut kvgraph = KVGraph::new();
        let n0 = kvgraph.add_node_with_label("toronto");
        let n1 = kvgraph.add_node_with_label("seattle");
        kvgraph.add_edge_with_label(&[n0, n1], "AC123");
        kvgraph.insert(&n0, "darkness", 0.6).unwrap();
        kvgraph.insert(&n1, "darkness", 0.8).unwrap();
        let df = kvgraph.dataframe();
        println!("{:}", df);
    }
}
