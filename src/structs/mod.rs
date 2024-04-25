//! This module stores the "more abstract" datastructures related to hypergraph
//! storage and communication.

use uuid::Uuid;

pub type EdgeID = u64;
pub type EdgeWeight = f64;

mod edge_set;
mod generic_vec;

pub use generic_vec::GeneroVector;

pub use edge_set::EdgeSet;
