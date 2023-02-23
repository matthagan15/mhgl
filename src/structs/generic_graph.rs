use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::traits::HgBasis;

use super::{
    generic_edge::GeneroEdge, generic_vec::GeneroVector, EdgeDirection, EdgeID, EdgeWeight,
    GraphID, SparseEdge,
};

#[derive(Debug, Clone)]
pub struct GeneroGraph<B: HgBasis> {
    pub id: GraphID,
    edges: HashMap<EdgeID, GeneroEdge<B>>,
    input_cardinality_to_edges: HashMap<usize, HashSet<EdgeID>>,
    output_cardinality_to_edges: HashMap<usize, HashSet<EdgeID>>,

    // Decision made: if a node is present in an edges input basis
    // it should map here. Decided to not use the whole basis due to
    // the issue of blobs, as soon as you add blobs you have to compute power
    // sets and it becomes wasteful memory wise. Instead spend time searching
    // through acceptable edges.
    node_to_outbound_edges: HashMap<B, HashSet<EdgeID>>,
}

impl<B: HgBasis> GeneroGraph<B> {
    pub fn new() -> Self {
        GeneroGraph {
            id: Uuid::new_v4(),
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            node_to_outbound_edges: HashMap::new(),
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
        let mut ret = HashSet::new();
        for node in basis.nodes() {
            if let Some(potentials) = self.node_to_outbound_edges.get(&node) {
                for edge_id in potentials {
                    if let Some(edge) = self.edges.get(edge_id) {
                        if edge.can_map_basis(basis) {
                            ret.insert(edge_id.clone());
                        }
                    }
                }
            }
        }
        ret
    }

    pub fn get_containing_edges(&self, basis: &B) -> HashSet<EdgeID> {
        let mut ret = HashSet::new();
        for (id, edge) in self.edges.iter() {
            if edge.contains(basis) {
                ret.insert(id.clone());
            }
        }
        ret
    }

    pub fn add_edge(&mut self, new_edge: GeneroEdge<B>) {
        match new_edge.direction {
            EdgeDirection::Directed => {
                self.input_cardinality_to_edges
                    .entry(new_edge.input_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                self.output_cardinality_to_edges
                    .entry(new_edge.output_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                self.edges.insert(new_edge.id.clone(), new_edge);
            }
            EdgeDirection::Loop => {
                self.input_cardinality_to_edges
                    .entry(new_edge.input_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                self.output_cardinality_to_edges
                    .entry(new_edge.input_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                self.edges.insert(new_edge.id.clone(), new_edge);
            }
            EdgeDirection::Oriented | EdgeDirection::Undirected => {
                self.input_cardinality_to_edges
                    .entry(new_edge.input_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                self.input_cardinality_to_edges
                    .entry(new_edge.output_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                self.output_cardinality_to_edges
                    .entry(new_edge.input_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                self.output_cardinality_to_edges
                    .entry(new_edge.output_cardinality())
                    .or_default()
                    .insert(new_edge.id.clone());
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                for node in new_edge.out_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                self.edges.insert(new_edge.id.clone(), new_edge);
            }
            EdgeDirection::Blob => {
                for ix in 0..(new_edge.input_cardinality() + 1) {
                    self.input_cardinality_to_edges
                        .entry(ix)
                        .or_default()
                        .insert(new_edge.id.clone());
                    self.output_cardinality_to_edges
                        .entry(ix)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                self.edges.insert(new_edge.id.clone(), new_edge);
            }
        }
    }

    pub fn remove_edge(&mut self, edge_id: &EdgeID) -> Option<GeneroEdge<B>> {
        if let Some(edge) = self.edges.remove(edge_id) {
            match edge.direction {
                EdgeDirection::Directed | EdgeDirection::Loop => {
                    if let Some(set) = self
                        .input_cardinality_to_edges
                        .get_mut(&edge.input_cardinality())
                    {
                        set.remove(edge_id);
                    }
                    if let Some(set) = self
                        .output_cardinality_to_edges
                        .get_mut(&edge.output_cardinality())
                    {
                        set.remove(edge_id);
                    }
                    for node in edge.in_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    Some(edge)
                }
                EdgeDirection::Oriented | EdgeDirection::Undirected => {
                    if let Some(set) = self
                        .input_cardinality_to_edges
                        .get_mut(&edge.input_cardinality())
                    {
                        set.remove(edge_id);
                    }
                    if let Some(set) = self
                        .input_cardinality_to_edges
                        .get_mut(&edge.output_cardinality())
                    {
                        set.remove(edge_id);
                    }
                    if let Some(set) = self
                        .output_cardinality_to_edges
                        .get_mut(&edge.output_cardinality())
                    {
                        set.remove(edge_id);
                    }
                    if let Some(set) = self
                        .output_cardinality_to_edges
                        .get_mut(&edge.input_cardinality())
                    {
                        set.remove(edge_id);
                    }
                    for node in edge.in_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    for node in edge.out_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    Some(edge)
                }
                EdgeDirection::Blob => {
                    for ix in 0..=edge.input_cardinality() {
                        if let Some(set) = self.input_cardinality_to_edges.get_mut(&ix) {
                            set.remove(edge_id);
                        }
                        if let Some(set) = self.output_cardinality_to_edges.get_mut(&ix) {
                            set.remove(edge_id);
                        }
                    }
                    for node in edge.in_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    Some(edge)
                }
            }
        } else {
            None
        }
    }

    /// Change the input of an existing edge. If edge is a blob type it will
    /// simply replace the blob basis with the new basis, keeping the ID
    /// the same.
    pub fn change_edge_input(&mut self, edge_id: &EdgeID, new_input: B) {
        // TODO: Might be easier for blobs to simply create a new edge,
        // remove the old one, and insert the new one?
        if let Some(mut e) = self.remove_edge(edge_id) {
            e.change_input(new_input);
            self.add_edge(e);
        }
    }

    pub fn change_edge_output(&mut self, edge_id: &EdgeID, new_output: B) {
        // TODO: wrong because changing output of an undirected edge affects input maps.
        if let Some(mut e) = self.remove_edge(edge_id) {
            e.change_output(new_output);
            self.add_edge(e);
        }
    }

    /// Returns the sum total of all edge weights mapping input basis `input` to output basis `output`.
    pub fn query_weight(&self, input: &B, output: &B) -> EdgeWeight {
        let mut ret = 0.;
        for (b,w) in self.map_basis(input).to_tuples() {
            if b == *output {
                ret += w;
            }
        }
        ret
    }

    pub fn query_edges(&self, input: &B, output: &B) -> Vec<EdgeID> {
        let mut ret = Vec::new();
        for (id, edge) in self.edges.iter() {
            if edge.can_map_basis(basis)
        }
    }

    pub fn map_basis(&self, input: &B) -> GeneroVector<B> {
        let mut ret = GeneroVector::new();
        let mut good_edges = HashSet::new();
        for node in input.nodes() {
            if let Some(edges) = self.node_to_outbound_edges.get(&node) {
                for edge_id in edges {
                    if let Some(edge) = self.edges.get(edge_id) {
                        if edge.can_map_basis(input) {
                            good_edges.insert(edge_id);
                        }
                    }
                }
            }
        }
        for edge_id in good_edges {
            let e = self
                .edges
                .get(edge_id)
                .expect("This was checked in prior loop.");
            ret += &e.map(input);
        }
        ret
    }
    pub fn map(&self, input: &GeneroVector<B>) -> GeneroVector<B> {
        let mut ret = GeneroVector::new();
        for (b, w) in input.basis_to_weight.iter() {
            let mut tmp = self.map_basis(&b);
            tmp *= *w;
        }
        ret
    }
}
