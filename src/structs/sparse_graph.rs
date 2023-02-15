use crate::traits::{HgNode, HgVector};
use crate::{
    structs::{
        sparse_edge::{EdgeDirection, SparseEdge},
        sparse_vec::SparseVector,
        EdgeID, EdgeWeight, NodeUUID,
    },
    traits::HyperGraph,
};

use core::num;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    collections::{hash_set::Iter, HashMap, HashSet},
    f32::consts::E,
    fmt::Display,
    fs,
    ops::{Add, AddAssign, Index},
};
use uuid::Uuid;

use num_integer::binomial;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::GraphID;

/// Current question that needs to be figured out: what kind of quick lookups do we want to support?
/// label to uuid seems obvious, currently also caching dimension to edges. Currently do not handle multiple nodes or even edges sharing the same label well.
/// TODO: Handle non-unique labels. we currently just ignore.
///
/// EDGES CANNOT BE SHARED AMONG HYPERGRAPHS.
/// ### Fields
/// - id: A unique 128 bit number, typically viewed hex.
/// - nodes: A HashSet of NodeIDs. a vertex set.
/// - edges: A HashMap from an EdgeID to the Edge object containing any edge specific data.
/// - dimension_to_edges_cache
///
/// The user cannot have ability to modify uuids. why use Uuids instead of strings? because they are smaller.
///
///
/// TODO:
/// - Need to remove graphs from holding the edges as this then means that a database is essentially duplicating edges if it holds muliple graphs supported over the same nodes.
/// - start with assumption that all nodes are assigned the same datatype. Then relax constaint. this should dictate what is needed of the nodes.
/// How does the user interact with it? A string?
/// interaction border is if something is hashable or not, so use this trait! If I can hash, then I can convert to a number, then convert to a Uuid.
/// All this should care about is the connections. Don't worry about storing data, that can be done by simply having a hashmap from a Uuid to a type you want.
/// edges have more information so they need an associated type.
///
/// - how can you compute the most important node in a hypergraph?
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SparseGraph<N: HgNode> {
    id: Uuid,
    pub nodes: HashSet<N>,
    pub edges: HashMap<EdgeID, SparseEdge<N>>,
    // TODO: This should be updated to a map from (input_dim, output_dim) -> EdgeID's.
    input_cardinality_to_edges: HashMap<usize, HashSet<EdgeID>>,
    output_cardinality_to_edges: HashMap<usize, HashSet<EdgeID>>,
    node_to_containing_edges: HashMap<N, HashSet<EdgeID>>,
}

impl SparseGraph<u8> {
    /// If num_nodes is greater than the underlying storage method (aka u8) then it returns empty hypergraph.
    pub fn new_with_num_nodes(num_nodes: usize) -> SparseGraph<u8> {
        SparseGraph {
            id: Uuid::new_v4(),
            nodes: (0..num_nodes as u8).collect(),
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }
}

impl SparseGraph<u16> {
    /// If num_nodes is greater than the underlying storage method (aka u8) then it returns empty hypergraph.
    pub fn new_with_num_nodes(num_nodes: usize) -> SparseGraph<u16> {
        SparseGraph {
            id: Uuid::new_v4(),
            nodes: (0..num_nodes as u16).collect(),
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }
}

impl SparseGraph<u32> {
    /// If num_nodes is greater than the underlying storage method (aka u8) then it returns empty hypergraph.
    pub fn new_with_num_nodes(num_nodes: usize) -> SparseGraph<u32> {
        SparseGraph {
            id: Uuid::new_v4(),
            nodes: (0..num_nodes as u32).collect(),
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }
}

impl SparseGraph<u64> {
    /// If num_nodes is greater than the underlying storage method (aka u8) then it returns empty hypergraph.
    pub fn new_with_num_nodes(num_nodes: usize) -> SparseGraph<u64> {
        SparseGraph {
            id: Uuid::new_v4(),
            nodes: (0..num_nodes as u64).collect(),
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }
}

impl SparseGraph<u128> {
    pub fn new_with_num_nodes(num_nodes: usize) -> SparseGraph<u128> {
        let mut node_set = HashSet::with_capacity(num_nodes);
        for _ in 0..num_nodes {
            node_set.insert(Uuid::new_v4().as_u128());
        }
        SparseGraph {
            id: Uuid::new_v4(),
            nodes: node_set,
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }
}

impl SparseGraph<Uuid> {
    pub fn new_with_num_nodes(num_nodes: usize) -> SparseGraph<Uuid> {
        let mut node_set = HashSet::with_capacity(num_nodes);
        for _ in 0..num_nodes {
            node_set.insert(Uuid::new_v4());
        }
        SparseGraph {
            id: Uuid::new_v4(),
            nodes: node_set,
            edges: HashMap::new(),
            input_cardinality_to_edges: HashMap::new(),
            output_cardinality_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }
}

impl<N: HgNode> SparseGraph<N> {
    pub fn new() -> SparseGraph<N> {
        SparseGraph { id: Uuid::new_v4(), nodes: HashSet::new(), edges: HashMap::new(), input_cardinality_to_edges: HashMap::new(), output_cardinality_to_edges: HashMap::new(), node_to_containing_edges: HashMap::new() }
    }

    /// adds the nodes to the internal set
    pub fn add_nodes(&mut self, nodes: HashSet<N>) {
        for node in nodes {
            self.nodes.insert(node);
        }
    }

    pub fn update_edge_weight(&mut self, edge_id: &EdgeID, new_weight: EdgeWeight) {
        if let Some(e) = self.edges.get_mut(edge_id) {
            e.weight = new_weight;
        }
    }

    pub fn find_edge_from_nodes(
        &self,
        input_nodes: &HashSet<N>,
        output_nodes: &HashSet<N>,
    ) -> Option<&EdgeID> {
        let in_dim = input_nodes.len();
        let out_dim = output_nodes.len();
        if let Some(possible_from_in) = self.input_cardinality_to_edges.get(&in_dim) {
            for possible in possible_from_in {
                if let Some(e) = self.edges.get(possible) {
                    if e.matches_input(input_nodes) {
                        if e.output_dim() == output_nodes.len() {
                            if e.matches_input(input_nodes) && e.matches_output(output_nodes) {
                                return Some(&e.id);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Returns a vec containing the edge ID of all edges that map from the given
    /// basis set provided.
    pub fn find_edges_from_basis(&self, input_nodes: &HashSet<N>) -> Vec<EdgeID> {
        let mut ret = Vec::new();
        for node in input_nodes {
            if let Some(new_edges) = self.node_to_containing_edges.get(node) {
                for edge in new_edges {
                    if let Some(e) = self.edges.get(edge) {
                        if e.matches_input(input_nodes) {
                            ret.push(edge.clone());
                        }
                    }
                }
            }
        }
        ret
    }

    pub fn from_edges(edges: Vec<SparseEdge<N>>) -> SparseGraph<N> {
        let mut hg = SparseGraph::new();
        for edge in edges {
            hg.add_edge(edge);
        }
        hg
    }

    pub fn add_input_node_to_edge(&mut self, node: N, edge_id: &EdgeID) -> bool {
        if let Some(edge) = self.edges.get_mut(edge_id) {
            edge.add_input_node(node);
            true
        } else {
            false
        }
    }

    pub fn add_output_node_to_edge(&mut self, node: N, edge_id: &EdgeID) -> bool {
        if let Some(edge) = self.edges.get_mut(edge_id) {
            edge.add_output_node(node);
            true
        } else {
            false
        }
    }

    pub fn remove_input_node_from_edge(&mut self, node: &N, edge: &EdgeID) {
        if let Some(edge) = self.edges.get_mut(edge) {
            edge.remove_input_node(node);
        }
    }
    pub fn remove_output_node_from_edge(&mut self, node: &N, edge: &EdgeID) {
        if let Some(edge) = self.edges.get_mut(edge) {
            edge.remove_output_node(node);
        }
    }
    pub fn remove_node(&mut self, node: &N) {
        for edge in self.edges.values_mut() {
            edge.remove_node(node);
        }
        self.nodes.remove(node);
    }

    pub fn remove_edge(&mut self, edge_id: &EdgeID) {
        if let Some(edge) = self.edges.remove(edge_id) {
            let in_dim = edge.in_nodes.len();
            let out_dim = edge.out_nodes.len();
            for node in edge.total_nodes() {
                if let Some(edge_set) = self.node_to_containing_edges.get_mut(&node) {
                    edge_set.remove(edge_id);
                }
            }
            match edge.direction {
                EdgeDirection::Directed => {
                    if let Some(edge_set) = self.input_cardinality_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_cardinality_to_edges.get_mut(&out_dim) {
                        edge_set.remove(edge_id);
                    }
                }
                EdgeDirection::Oriented | EdgeDirection::Undirected => {
                    if let Some(edge_set) = self.input_cardinality_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.input_cardinality_to_edges.get_mut(&out_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_cardinality_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_cardinality_to_edges.get_mut(&out_dim) {
                        edge_set.remove(edge_id);
                    }
                }
                EdgeDirection::Loop => {
                    if let Some(edge_set) = self.input_cardinality_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_cardinality_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                }
                EdgeDirection::Blob => {
                    for dim in 0..in_dim {
                        if let Some(edge_set) = self.input_cardinality_to_edges.get_mut(&dim) {
                            edge_set.remove(edge_id);
                        }
                        if let Some(edge_set) = self.output_cardinality_to_edges.get_mut(&dim) {
                            edge_set.remove(edge_id);
                        }
                    }
                }
                _ => {}
            };
        }
    }

    pub fn clone_id(&self) -> GraphID {
        self.id.clone()
    }
    /// Add an edge. There are no real failure modes with just HashMaps and HashSets.
    pub fn add_edge(&mut self, edge: SparseEdge<N>) {
        self.add_nodes(edge.total_nodes());
        let e_id = edge.clone_id();
        let in_dim = edge.in_nodes.len();
        let out_dim = edge.out_nodes.len();

        if edge.direction == EdgeDirection::Directed
            || edge.direction == EdgeDirection::Undirected
            || edge.direction == EdgeDirection::Oriented
        {
            self.input_cardinality_to_edges
                .entry(in_dim)
                .or_default()
                .insert(e_id.clone());
            self.output_cardinality_to_edges
                .entry(out_dim)
                .or_default()
                .insert(e_id.clone());
            for node in edge.in_nodes.iter() {
                self.node_to_containing_edges
                    .entry(node.clone())
                    .or_default()
                    .insert(e_id.clone());
            }
            if edge.direction == EdgeDirection::Undirected
                || edge.direction == EdgeDirection::Oriented
            {
                self.input_cardinality_to_edges
                    .entry(out_dim)
                    .or_default()
                    .insert(e_id.clone());
                self.output_cardinality_to_edges
                    .entry(in_dim)
                    .or_default()
                    .insert(e_id.clone());
                for node in edge.out_nodes.iter() {
                    self.node_to_containing_edges
                        .entry(node.clone())
                        .or_default()
                        .insert(e_id.clone());
                }
            }
        }

        if edge.direction == EdgeDirection::Loop {
            self.input_cardinality_to_edges
                .entry(in_dim)
                .or_default()
                .insert(e_id.clone());
            self.output_cardinality_to_edges
                .entry(in_dim)
                .or_default()
                .insert(e_id.clone());
            for node in edge.in_nodes.iter() {
                self.node_to_containing_edges
                    .entry(node.clone())
                    .or_default()
                    .insert(e_id.clone());
            }
        }

        if edge.direction == EdgeDirection::Blob {
            for dim in 0..edge.in_nodes.len() {
                self.input_cardinality_to_edges
                    .entry(dim)
                    .or_default()
                    .insert(e_id.clone());
                self.output_cardinality_to_edges
                    .entry(dim)
                    .or_default()
                    .insert(e_id.clone());
            }
            for node in edge.total_nodes().into_iter() {
                self.node_to_containing_edges
                    .entry(node)
                    .or_default()
                    .insert(e_id.clone());
            }
        }
        self.edges.insert(e_id, edge);
    }

    /// Number of nodes in the graph
    pub fn len(&self) -> usize {
        self.nodes.len() + self.edges.len()
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    /// iterator over nodes present.
    pub fn node_iter(&self) -> Iter<N> {
        self.nodes.iter()
    }

    /// Clone of the node set present
    pub fn nodes(&self) -> HashSet<N> {
        self.nodes.clone()
    }

    pub fn has_nodes(&self, nodes: &HashSet<N>) -> bool {
        self.nodes.is_superset(nodes)
    }

    pub fn edges(&self) -> Vec<Uuid> {
        self.edges.keys().cloned().collect()
    }

    pub fn get_edge_from_id(&self, id: &EdgeID) -> Option<&SparseEdge<N>> {
        self.edges.get(id)
    }

    pub fn get_edge_copy(&self, edge_id: &EdgeID) -> Option<SparseEdge<N>> {
        self.edges.get(edge_id).cloned()
    }

    /// Returns a list of EdgeIDs that have the provided node as a given input.
    pub fn get_outbound_edges(&self, node: &N) -> HashSet<EdgeID> {
        if let Some(edges) = self.node_to_containing_edges.get(node) {
            edges.clone()
        } else {
            HashSet::new()
        }
    }

    pub fn map_vec(&self, x: SparseVector<N>) -> SparseVector<N> {
        let mut ret = SparseVector::new();
        for (basis, coeff) in x.basis() {
            let mut tmp = self.map_basis(&basis.into_iter().collect());
            tmp *= coeff;
            ret += tmp;
        }
        ret
    }
    fn map_basis(&self, b: &HashSet<N>) -> SparseVector<N> {
        let mut potential_edges = HashSet::new();
        let input_dim = b.len();
        for node in b.iter() {
            if let Some(edges) = self.node_to_containing_edges.get(node) {
                for potential in edges {
                    if let Some(edges_by_dim) = self.input_cardinality_to_edges.get(&input_dim) {
                        if edges_by_dim.contains(potential) {
                            potential_edges.insert(potential);
                        }
                    }
                }
            }
        }
        let mut ret = SparseVector::<N>::new();
        for p in potential_edges {
            if let Some(e) = self.edges.get(p) {
                ret += e.map_basis(b);
            }
        }
        ret
    }

    /// Return a uniformly random basis vector.
    pub fn random_basis(&self) -> SparseVector<N> {
        let mut base = HashSet::new();
        let mut rng = thread_rng();
        for node in self.nodes.iter() {
            if rng.gen_bool(0.5) {
                base.insert(node.clone());
            }
        }
        SparseVector::<N>::from_basis(base, 1.)
    }
}

impl Display for SparseGraph<NodeUUID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let graph_string = self.id.clone().to_string();

        let mut node_string = String::new();
        let nodes: Vec<&NodeUUID> = self.nodes.iter().collect();
        for n_ix in 0..nodes.len() {
            node_string += &nodes[n_ix].to_string();
            if n_ix == nodes.len() - 1 {
                node_string += ".";
            } else {
                node_string += ", ";
            }
        }

        let mut edge_string = String::new();
        let edges_vec: Vec<&SparseEdge<NodeUUID>> = self.edges.values().collect();
        for e_ix in 0..self.edges.len() {
            edge_string += &edges_vec[e_ix].to_string();
            if e_ix == self.edges.len() - 1 {
                edge_string += ".";
            } else {
                edge_string += ", "
            }
        }
        write!(
            f,
            "Hypergraph: {graph_string}\nNodes:\n{node_string}\nEdges:\n{edge_string}"
        )
    }
}

mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::structs::sparse_graph::SparseGraph;
}
