//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type NodeID = Uuid;
pub type EdgeID = Uuid;
pub type PathID = Uuid;
pub type GraphID = Uuid;
pub type EdgeWeight = f64;

mod hyperedge;
pub mod bit_edge;
mod hypergraph;
mod node_vec;

pub use hyperedge::HyperEdge;
pub use hypergraph::HyperGraph;
pub use hyperedge::EdgeDirection;
pub use node_vec::HgVector;
