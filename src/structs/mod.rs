//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type NodeUUID = Uuid;
pub type EdgeID = Uuid;
pub type PathID = Uuid;
pub type GraphID = Uuid;
pub type EdgeWeight = f64;

mod basis;
pub mod bit_edge;
pub mod bit_nodes;
pub mod hgraph;
mod hyperedge;
mod node_vec;
mod sparse_graph;

pub use hyperedge::EdgeDirection;
pub use hyperedge::SparseEdge;
pub use node_vec::SparseVector;
pub use sparse_graph::SparseGraph;
