use std::collections::HashMap;

use crate::structs::GraphID;

use super::{bit_edge::BitEdge, EdgeID};
pub struct BitGraph<const K: usize> {
    id: GraphID,
    edges: HashMap<EdgeID, BitEdge<K>>,
}