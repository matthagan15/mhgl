//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type NodeUUID = Uuid;
pub type EdgeID = Uuid;
pub type PathID = Uuid;
pub type GraphID = Uuid;
pub type EdgeWeight = f64;

pub mod bit_edge;
mod hyperedge;
mod hypergraph;
pub mod matrix_graph;
mod node_vec;
pub mod nodes;

pub use hyperedge::EdgeDirection;
pub use hyperedge::SparseEdge;
pub use hypergraph::HyperGraph;
pub use node_vec::HgVector;
