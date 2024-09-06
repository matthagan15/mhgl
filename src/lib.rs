#![forbid(unsafe_code)]
//!# Matt's HyperGraph Library (mhgl)
//!
//! A library for working with undirected [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph), which are a generalization of a normal graph. A hypergraph consists of a set of nodes, denoted `N`, and a collection of edges  where each edge is a subset of `N`. For a normal graph each edge is required to be of size 2, for example an edge `(u, v)` between nodes `u` and `v`, whereas in a hypergraph there is no limit on the size of an edge. Each node and edge are assigned IDs, with the type for the ID depending on the struct used. The [`HyperGraph`] trait provides a common api for developing struct independent algorithms.
//!
//! # Hypergraph Structs
//! - [`ConGraph`](https://docs.rs/mhgl/latest/mhgl/struct.ConGraph.html) - a connectivity only option that uses `u32`'s as IDs for
//! nodes and `u64`'s for edge IDs with each being a simple counter starting at 0. No data that can be stored within the
//! `ConGraph` structure itself.
//! -(`HGraph`) - A struct generic over four types: the node data, the edge data, the node IDs, and the edge IDs. There are no trait bounds on the node and edge typesaAdditionally generic over the size of integers `u8` through `u128`
//! to store NodeIDs and EdgeIDs with `u32` and `u64` as the default for the respective IDs.
//! - (`KVGraph`) - A key-value hypergraph where each node and edge allows you
//! to store simple (`kvgraph::Value`)s modeled after a simple subset of the Polars `AnyValue<'a>`.
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
//! The last line in the above code when ran output:
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
//! - [`HyperGraph`] - A collection of functions for querying the adjacency
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
//! - [`HgNode`] - A marker trait for indicating which types are usuable for
//! node and edge IDs (spoiler: `u8`, `u16, `u32`, `u64`, and `u132`. Don't use `Uuid`s even though they implement the trait.)
//!
//!# Algorithms
//!
//! [`algs`](`crate::algs`)
//!
//! Mostly under construction, currently there is only a simple random walk either using link,
//! `boundary_up` * `boundary_down`, and `boundary_down` * `boundary_up` to determine the next subset to move to. I plan to
//! port some algorithms, such as the connected components, s_walk, and homology algorithms from `HyperNetX` to this library over time.
//!
//! # Alternative Hypergraph Libraries
//! This library should be considered as an **alpha** version. Here are a few
//! hypergraph libraries I found, the most mature of which is HyperNetX
//! developed by Pacific Northwest National Laboratory (PNNL).
//! - [HyperNetX](https://pnnl.github.io/HyperNetX/) (Python): The most complete hypergraph library with algorithms
//! for homology computations. Based on python and the underlying datastructure
//! seems to be pandas arrays.
//! - [HypergraphDB](https://hypergraphdb.org/) (Java): A database backend for storing and querying data, seems unmaintained but probably was ahead of its time.
//! - [Hypergraph](https://crates.io/crates/hypergraph) (Rust): Seemed limited in scope and a bit complicated to me.
//! - [Gudhi](https://gudhi.inria.fr/index.html) (C++): This library is focused on computing persistent homology bargraphs. As such it has datastructures for simplicial complexes and more.
pub mod algs;
mod congraph;
mod edge;
mod hgraph;
mod hypergraph;
#[cfg(feature = "uuid")]
pub mod kvgraph;
mod node_trait;

pub use congraph::ConGraph;
use edge::EdgeSet;
pub use hgraph::HGraph;
pub use hypergraph::HyperGraph;

#[cfg(feature = "uuid")]
pub use kvgraph::KVGraph;

pub use node_trait::HgNode;

mod hg_macro {

    #[macro_export]
    macro_rules! hg {
    ( $( $x:expr ),* ) => {
        {
            // TODO: change this to
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}
}

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
