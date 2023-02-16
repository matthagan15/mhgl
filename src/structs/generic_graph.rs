use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::traits::HgBasis;

use super::{generic_edge::GeneroEdge, EdgeID, GraphID, SparseEdge, EdgeWeight, EdgeDirection};

#[derive(Debug)]
pub struct GeneroGraph<B: HgBasis> {
    pub id: GraphID,
    edges: HashMap<EdgeID, GeneroEdge<B>>,
    input_cardinality_to_edges: HashMap<usize, HashSet<EdgeID>>,
    output_cardinality_to_edges: HashMap<usize, HashSet<EdgeID>>,
    basis_to_outbound_edges: HashMap<B, HashSet<EdgeID>>,
}

impl<B: HgBasis> GeneroGraph<B> {
    pub fn new() -> Self {
        GeneroGraph {
            id: Uuid::new_v4(),
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            basis_to_outbound_edges: HashMap::new(),
        }
    }

    pub fn update_edge_weight(&mut self, edge_id: &EdgeID, new_weight: EdgeWeight) {
        if new_weight.is_nan() == false {
            if let Some(e) = self.edges.get_mut(edge_id) {
                e.change_weight(new_weight);
            }
        }
    }

    pub fn get_outbound_edges(&self, basis: &B) -> HashSet<EdgeID> {
        if let Some(hs) = self.basis_to_outbound_edges.get(basis) {
            hs.clone()
        } else {
            HashSet::new()
        }
    }

    pub fn add_edge(&mut self, new_edge: GeneroEdge<B>) {
        // TODO: Need to modify for edge direction
        let e_id = new_edge.id.clone();
        let in_card = new_edge.input_cardinality();
        let out_card = new_edge.output_cardinality();
        let in_basis = new_edge.clone_input_nodes();
        self.edges.insert(e_id.clone(), new_edge);
        self.input_cardinality_to_edges.entry(in_card).or_default().insert(e_id.clone());
        self.output_cardinality_to_edges.entry(out_card).or_default().insert(e_id.clone());
        self.basis_to_outbound_edges.entry(in_basis).or_default().insert(e_id);
    }

    pub fn change_edge_input(&mut self, edge_id: &EdgeID, new_input: &B) {
        if let Some(edge) = self.edges.get_mut(edge_id) {
            // out with old
            self.input_cardinality_to_edges.entry(edge.input_cardinality()).and_modify(|h| {h.remove(&edge.id);});
            self.basis_to_outbound_edges.entry(edge.clone_input_nodes()).and_modify(|hs| {hs.remove(&edge.id);});

            // in with new
            edge.change_input(new_input.clone());
            self.input_cardinality_to_edges.entry(edge.input_cardinality()).or_default().insert(edge.id.clone());
            self.basis_to_outbound_edges.entry(edge.clone_input_nodes()).or_default().insert(edge.id.clone());
        }
    }

    pub fn change_edge_output(&mut self, edge_id: &EdgeID, new_output: &B) {
        if let Some(edge) = self.edges.get_mut(edge_id) {
            // out with old
            self.output_cardinality_to_edges.entry(edge.output_cardinality()).and_modify(|h| {h.remove(&edge.id);});

            // in with new
            edge.change_output(new_output.clone());
            self.output_cardinality_to_edges.entry(edge.output_cardinality()).or_default().insert(edge.id.clone());
        }
    }

    pub fn remove_edge(&mut self, edge_id: &EdgeID) {
        if let Some(edge) = self.edges.remove(edge_id) {
            match edge.direction {
                EdgeDirection::Directed | EdgeDirection::Loop => {
                    self.input_cardinality_to_edges.entry(edge.input_cardinality()).and_modify(|h| {h.remove(edge_id);});
                    self.output_cardinality_to_edges.entry(edge.output_cardinality()).and_modify(|h| {h.remove(edge_id);});
                    self.basis_to_outbound_edges.entry(edge.in_nodes).and_modify(|h| {h.remove(edge_id);});
                },
                EdgeDirection::Oriented | EdgeDirection::Undirected => {
                    self.input_cardinality_to_edges.entry(edge.input_cardinality()).and_modify(|h| {h.remove(edge_id);});
                    self.input_cardinality_to_edges.entry(edge.output_cardinality()).and_modify(|h| {h.remove(edge_id);});
                    self.output_cardinality_to_edges.entry(edge.input_cardinality()).and_modify(|h| {h.remove(edge_id);});
                    self.output_cardinality_to_edges.entry(edge.output_cardinality()).and_modify(|h| {h.remove(edge_id);});
                    self.basis_to_outbound_edges.entry(edge.in_nodes).and_modify(|h| {h.remove(edge_id);});
                    self.basis_to_outbound_edges.entry(edge.out_nodes).and_modify(|h| {h.remove(edge_id);});
                },
                EdgeDirection::Blob => {
                    // Need to implement power set on basis :(

                },
            }
        }
    }

}
