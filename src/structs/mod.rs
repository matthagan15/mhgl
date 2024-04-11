//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type NodeID = Uuid;
pub type EdgeID = Uuid;
pub type PathID = Uuid;
pub type GraphID = Uuid;
pub type EdgeWeight = f64;


mod generic_graph;
mod generic_vec;
mod sparse_basis;

pub use generic_graph::HGraphCore;
pub use generic_vec::GeneroVector;
pub use generic_graph::Edge;

pub use sparse_basis::EdgeSet;

