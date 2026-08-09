#![forbid(unsafe_code)]
//!# Matt's HyperGraph Library (mhgl)
//!
//! A library for working with undirected [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph). An undirected hypergraph consists of a set of nodes and a collection of subsets of these nodes, a normal undirected "graph" is therefore a hypergraph where each subset only has 2 nodes. The most generic datatype provided by this library is the [`HGraph`](`crate::HGraph`) which allows for you to specify the data type used to store
//! both Node and Edge ID's (from any unsigned integer type) and is generic over the data type stored with each node and edge.
//!
//! Here is a sample usage of adding nodes, edges, computing containing and
//! maximal containing edges, links, and boundaries. The hypergraph specific
//! functions are collected in the [`HyperGraph`](`crate::HyperGraph`) trait and shared with the
//! connectivity only datastructure [`ConGraph`](`crate::ConGraph`).
//! ```rust
//! use mhgl::*;
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
//!
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

pub use congraph::ConGraph;
pub use edge::EdgeSet;
pub use hgraph::HGraph;
pub use hypergraph::HyperGraph;
#[cfg(feature = "edge_id_u32")]
pub type EdgeID = u32;

#[cfg(not(feature = "edge_id_u32"))]
pub type EdgeID = u64;

#[cfg(feature = "node_id_u32")]
pub type NodeID = u32;

#[cfg(not(feature = "node_id_u32"))]
pub type NodeID = u64;
