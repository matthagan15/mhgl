//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type NodeUUID = Uuid;
pub type EdgeID = Uuid;
pub type PathID = Uuid;
pub type GraphID = Uuid;
pub type EdgeWeight = f64;

pub mod bit_edge;
pub mod hgraph;
mod hyperedge;
mod node_vec;
pub mod nodes;
mod sparse_graph;
mod vec_trait;

pub use hyperedge::EdgeDirection;
pub use hyperedge::SparseEdge;
pub use node_vec::SparseVector;
pub use sparse_graph::SparseGraph;
