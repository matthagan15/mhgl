use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use uuid::Uuid;

use crate::{structs::GraphID, traits::HgBasis};

use super::{bit_edge::BitEdge, bit_nodes::BitNodes, BitVec, EdgeID};

#[derive(Debug, Clone)]
pub struct BitGraph<const K: usize> {
    pub id: GraphID,
    edges: HashMap<EdgeID, BitEdge<K>>,
    input_card_to_edges: HashMap<usize, HashSet<EdgeID>>,
    output_card_to_edges: HashMap<usize, HashSet<EdgeID>>,
}

impl<const K: usize> BitGraph<K> {
    pub fn new() -> Self {
        BitGraph {
            id: Uuid::new_v4(),
            edges: HashMap::new(),
            input_card_to_edges: HashMap::new(),
            output_card_to_edges: HashMap::new(),
        }
    }
    pub fn from_edges(edges: Vec<BitEdge<K>>) -> Self {
        let mut input_card = HashMap::new();
        let mut output_card = HashMap::new();
        let hm = edges
            .into_iter()
            .map(|e| {
                let ins: &mut HashSet<EdgeID> =
                    input_card.entry(e.input_cardinality()).or_default();
                let outs: &mut HashSet<EdgeID> =
                    output_card.entry(e.output_cardinality()).or_default();
                ins.insert(e.id.clone());
                outs.insert(e.id.clone());
                (e.id.clone(), e)
            })
            .collect();
        BitGraph {
            id: Uuid::new_v4(),
            edges: hm,
            input_card_to_edges: input_card,
            output_card_to_edges: output_card,
        }
    }
    pub fn clone_id(&self) -> GraphID {
        self.id.clone()
    }
    pub fn edge_ids(&self) -> Vec<EdgeID> {
        self.edges.keys().cloned().collect()
    }

    pub fn get_edges_copy(&self) -> Vec<BitEdge<K>> {
        self.edges.values().cloned().collect()
    }
    pub fn get_edge_copy() {}
    pub fn update_edge_weight() {}
    pub fn find_connecting_edges() {}
    pub fn find_outbound_edges() {}
    pub fn add_input_node_to_edge() {}
    pub fn add_output_node_to_edge() {}
    pub fn remove_input_node_from_edge() {}
    pub fn remove_output_node_from_edge() {}
    pub fn remove_edge() {}
    pub fn map_basis(&self, input: BitNodes<K>) {
        let mut tot = 0.0;
        if let Some(potentials) = self.input_card_to_edges.get(&input.cardinality()) {
            for potential in potentials {
                if let Some(edge) = self.edges.get(potential) {
                    if edge.matches_input(&input) {}
                }
            }
        }
    }
    pub fn map_vec() {}
}
