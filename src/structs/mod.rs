//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type NodeID = Uuid;
pub type EdgeID = Uuid;
pub type PathID = Uuid;
pub type GraphID = Uuid;
pub type EdgeWeight = f64;

mod bit_basis;
mod const_generics_bit_basis;
mod generic_edge;
mod generic_graph;
mod generic_vec;
mod path;
mod sparse_basis;

pub use generic_edge::EdgeDirection;
pub use generic_edge::GeneroEdge;
pub use generic_graph::GeneroGraph;
pub use generic_vec::GeneroVector;

pub use bit_basis::BitBasis;
pub use const_generics_bit_basis::ConstGenBitBasis;
pub use sparse_basis::SparseBasis;

pub use path::HgPath;
