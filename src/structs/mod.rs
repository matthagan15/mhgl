//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type NodeID = Uuid;
pub type EdgeID = Uuid;
pub type PathID = Uuid;
pub type GraphID = Uuid;
pub type EdgeWeight = f64;

pub mod bit_edge;
mod bit_graph;
pub mod bit_nodes;
mod bit_vec;
mod generic_edge;
mod generic_graph;
mod generic_vec;
mod sparse_basis;
mod sparse_edge;
mod sparse_graph;
mod sparse_vec;
mod path;

pub use generic_edge::GeneroEdge;
pub use generic_graph::GeneroGraph;
pub use generic_vec::GeneroVector;

pub use bit_graph::BitGraph;
pub use bit_vec::BitVec;

pub use sparse_basis::SparseBasis;
pub use sparse_edge::EdgeDirection;
pub use sparse_edge::SparseEdge;
pub use sparse_graph::SparseGraph;
pub use sparse_vec::SparseVector;

pub use path::HgPath;