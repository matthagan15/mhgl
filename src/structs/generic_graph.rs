use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use serde::{Deserialize, Serialize};


use uuid::Uuid;

use crate::traits::HgBasis;

use super::{
    generic_edge::{EdgeDirection, GeneroEdge},
    generic_vec::GeneroVector,
    EdgeID, EdgeWeight, GraphID,
};

/// The underlying structure for the directed graph types. Generic over
/// the basis type provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneroGraph<B: HgBasis> {
    pub id: GraphID,
    pub edges: HashMap<EdgeID, GeneroEdge<B>>,
    node_to_outbound_edges: HashMap<B, HashSet<EdgeID>>,
}

impl<B: HgBasis> GeneroGraph<B> {
    pub fn new() -> Self {
        GeneroGraph {
            id: Uuid::new_v4(),
            edges: HashMap::new(),
            node_to_outbound_edges: HashMap::new(),
        }
    }

    pub fn clone_edges(&self) -> Vec<EdgeID> {
        self.edges.keys().cloned().collect()
    }

    pub fn update_edge_weight(&mut self, edge_id: &EdgeID, new_weight: EdgeWeight) {
        if new_weight.is_nan() == false {
            if let Some(e) = self.edges.get_mut(edge_id) {
                e.change_weight(new_weight);
            }
        }
    }

    /// Returns all EdgeIDs that map from this basis to another.
    pub fn get_outbound_edges(&self, basis: &B) -> HashSet<EdgeID> {
        // TODO: This is inefficient. Take the intersection of edge_ids
        // from node_to_outbound_edges before checking if they can map the
        // given basis.
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

    pub fn query_edge(&self, edge_id: &EdgeID) -> Option<GeneroEdge<B>> {
        self.edges.get(edge_id).cloned()
    }

    /// Gets all edges such that the basis is contained in the union
    /// of the edges input and output
    pub fn get_containing_edges(&self, basis: &B) -> HashSet<EdgeID> {
        let mut ret = HashSet::new();
        for (id, edge) in self.edges.iter() {
            if edge.contains(basis) {
                ret.insert(id.clone());
            }
        }
        ret
    }

    /// Returns the cardinality of the edge for undirecteds
    pub fn get_edge_len(&self, edge_id: &EdgeID) -> Option<usize> {
        self.edges.get(edge_id).map(|e| e.input_cardinality())
    }

    /// Warning: currently only works for undirected graph types (such as HGraph)
    /// Should we include the provided edge or no? If the provided edge is 
    /// not present then we were given a bunk edge_id. If it is then we were
    /// able to at least find it.
    pub fn get_containing_edges_id(&self, edge_id: &EdgeID) -> HashSet<EdgeID> {
        if let Some(edge) = self.edges.get(edge_id) {
            let nodes: Vec<B> = edge.node_vec();
            if nodes.len() == 0 {
                HashSet::from([edge_id.clone()])
            } else {
                let n0 = &nodes[0];
                let outbounds = self.node_to_outbound_edges.get(n0).unwrap();
                outbounds.iter().filter(|e_id| {
                    let e = self.edges.get(e_id).unwrap();
                    e.contains(&edge.in_nodes)
                }).cloned().collect()
            }
        } else {
            HashSet::new()
        }
    }

    pub fn add_edge(&mut self, new_edge: GeneroEdge<B>) {
        match new_edge.direction {
            EdgeDirection::Directed => {
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                self.edges.insert(new_edge.id.clone(), new_edge);
            }
            EdgeDirection::Loop => {
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_edge.id.clone());
                }
                self.edges.insert(new_edge.id.clone(), new_edge);
            }
            EdgeDirection::Oriented | EdgeDirection::Symmetric => {
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
            EdgeDirection::Undirected => {
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
                    for node in edge.in_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    Some(edge)
                }
                EdgeDirection::Oriented | EdgeDirection::Symmetric => {
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
                EdgeDirection::Undirected => {
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

    pub fn edges_of_size(&self, size: usize) -> Vec<EdgeID> {
        self.edges
            .iter()
            .filter_map(|(k, v)| {
                if v.input_cardinality() == size {
                    Some(k)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }

    /// Change the input of an existing edge. If edge is a undirected type it will
    /// simply replace the undirected basis with the new basis, keeping the ID
    /// the same.
    pub fn change_edge_input(&mut self, edge_id: &EdgeID, new_input: B) {
        // Due to undirecteds it is simply easier to remove the edge and reinsert
        // the modified edge.
        if let Some(mut e) = self.remove_edge(edge_id) {
            e.change_input(new_input);
            self.add_edge(e);
        }
    }

    /// Change the output of the provided edge_id to the new basis. If edge
    /// is a undirected or loop then nothing is done, use `change_edge_input` instead.
    pub fn change_edge_output(&mut self, edge_id: &EdgeID, new_output: B) {
        // Edge is removed and re-added to avoid duplicating logic of undirected
        // or undirected style edges. For example changing output of an undirected
        // edge requires changing all of the inputs/outgoing edges from the
        // outbound map.
        if let Some(mut e) = self.remove_edge(edge_id) {
            e.change_output(new_output);
            self.add_edge(e);
        }
    }

    /// Returns the sum total of all edge weights mapping input basis `input` to output basis `output`.
    pub fn query_weight(&self, input: &B, output: &B) -> EdgeWeight {
        let mut ret = 0.;
        for (b, w) in self.map_basis(input).to_tuples() {
            if b == *output {
                ret += w;
            }
        }
        ret
    }

    pub fn query_edges(&self, input: &B, output: &B) -> Vec<EdgeID> {
        let outbounds = self.get_outbound_edges(input);
        outbounds
            .into_iter()
            .filter(|e| {
                if let Some(edge) = self.edges.get(e) {
                    edge.is_correctly_mapped(input, output)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns edge_id of undirected edges on the input basis.
    pub fn query_undirected(&self, input: &B) -> Vec<EdgeID> {
        let mut potential_edges = HashSet::new();
        for node in input.nodes() {
            if potential_edges.len() == 0 && self.node_to_outbound_edges.contains_key(&node) {
                potential_edges = potential_edges
                    .union(self.node_to_outbound_edges.get(&node).unwrap())
                    .cloned()
                    .collect();
            } else if potential_edges.len() > 0 && self.node_to_outbound_edges.contains_key(&node) {
                potential_edges = potential_edges
                    .intersection(self.node_to_outbound_edges.get(&node).unwrap())
                    .cloned()
                    .collect();
            }
        }
        potential_edges
            .into_iter()
            .filter(|potential_edge| {
                if let Some(e) = self.edges.get(&potential_edge) {
                    e.matches_undirected(input)
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn query_loop(&self, input: &B) -> Vec<Uuid> {
        let possible_loops = self.get_outbound_edges(input);
        possible_loops
            .into_iter()
            .filter(|id| {
                if let Some(e) = self.edges.get(id) {
                    e.is_correctly_mapped(input, input)
                } else {
                    false
                }
            })
            .collect()
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
            ret += &e.map_to_vector(input);
        }
        ret
    }
    pub fn map(&self, input: &GeneroVector<B>) -> GeneroVector<B> {
        let ret = GeneroVector::new();
        for (b, w) in input.basis_to_weight.iter() {
            let mut tmp = self.map_basis(&b);
            tmp *= *w;
        }
        ret
    }
}

impl<B: HgBasis> Hash for GeneroGraph<B> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

mod tests {
    use crate::{structs::{GeneroEdge, GeneroGraph}, SparseBasis};

    #[test]
    fn test_serialization() {
        let mut g: GeneroGraph<SparseBasis<u32>> = GeneroGraph::new();
        let e = GeneroEdge::from(
            SparseBasis::<u32>::from(&1),
            SparseBasis::from(&2),
            1.,
            crate::EdgeDirection::Symmetric,
        );
        g.add_edge(e);
        let s = serde_json::to_string(&g).expect("could not serialize graph");
        dbg!(s);
    }
}
