use std::collections::HashMap;

use uuid::Uuid;

use crate::structs::GraphID;

use super::{bit_edge::BitEdge, EdgeID};
pub struct BitGraph<const K: usize> {
    id: GraphID,
    edges: HashMap<EdgeID, BitEdge<K>>,
}

impl<const K: usize> BitGraph<K> {
    pub fn new() -> Self {
        BitGraph { id: Uuid::new_v4(), edges: HashMap::new() }
    }
    pub fn from_edges() {}
    pub fn clone_id() {}
    pub fn edge_ids() {}
    pub fn get_edges_copy() {}
    pub fn get_edge_copy() {}
    pub fn update_edge_weight() {}
    pub fn find_connecting_edges() {}
    pub fn find_outbound_edges() {}
    pub fn add_input_node_to_edge() {}
    pub fn add_output_node_to_edge() {}
    pub fn remove_input_node_from_edge() {}
    pub fn remove_output_node_from_edge() {}
    pub fn remove_edge() {}
    pub fn map_basis() {}
    pub fn map_vec() {}
    
}