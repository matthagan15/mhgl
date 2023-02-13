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
mod bit_graph;
pub mod bit_nodes;
mod bit_vec;
pub mod hgraph;
mod hyperedge;
mod node_vec;
mod sparse_graph;
mod generic_vec;

pub use bit_graph::BitGraph;
pub use bit_vec::BitVec;

pub use hyperedge::EdgeDirection;
pub use hyperedge::SparseEdge;
pub use node_vec::SparseVector;
pub use sparse_graph::SparseGraph;
