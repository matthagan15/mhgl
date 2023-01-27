use crate::structs::{
    hyperedge::{EdgeDirection, SparseEdge},
    node_vec::HgVector,
    EdgeID, EdgeWeight, NodeUUID,
    nodes::NodeID,
};
use core::num;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    collections::{hash_set::Iter, HashMap, HashSet},
    fmt::Display,
    fs,
    ops::{Add, AddAssign, Index}, f32::consts::E,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct HyperGraph {
    id: Uuid,
    pub nodes: HashSet<NodeUUID>,
    pub edges: HashMap<EdgeID, SparseEdge<Uuid>>,
    // TODO: This should be updated to a map from (input_dim, output_dim) -> EdgeID's.
    input_dim_to_edges: HashMap<usize, HashSet<EdgeID>>,
    output_dim_to_edges: HashMap<usize, HashSet<EdgeID>>,
    node_to_containing_edges: HashMap<NodeUUID, HashSet<EdgeID>>,
}

impl HyperGraph {
    pub fn new() -> HyperGraph {
        HyperGraph {
            id: Uuid::new_v4(),
            nodes: HashSet::new(),
            edges: HashMap::new(),
            input_dim_to_edges: HashMap::new(),
            output_dim_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }

    pub fn new_with_num_nodes(num_nodes: usize) -> HyperGraph {
        HyperGraph {
            id: Uuid::new_v4(),
            nodes: (0..num_nodes).map(|_| Uuid::new_v4()).collect(),
            edges: HashMap::new(),
            input_dim_to_edges: HashMap::new(),
            output_dim_to_edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
        }
    }

    /// adds the nodes to the internal set
    pub fn add_nodes(&mut self, nodes: HashSet<NodeUUID>) {
        for node in nodes {
            self.nodes.insert(node);
        }
    }

    pub fn update_edge_weight(&mut self, edge_id: &EdgeID, new_weight: EdgeWeight) {
        if let Some(e) = self.edges.get_mut(edge_id) {
            e.weight = new_weight;
        }
    }

    pub fn find_edge_from_nodes(&mut self, input_nodes: &HashSet<Uuid>, output_nodes: &HashSet<Uuid>) -> Option<&EdgeID> {
        let in_dim = input_nodes.len();
        let out_dim = output_nodes.len();
        if let Some(possible_from_in) = self.input_dim_to_edges.get(&in_dim) {
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

    pub fn from_edges(edges: Vec<SparseEdge<Uuid>>) -> HyperGraph {
        let mut hg = HyperGraph::new();
        for edge in edges {
            hg.add_edge(edge);
        }
        hg
    }

    pub fn add_input_node_to_edge(&mut self, node: NodeUUID, edge_id: &EdgeID) -> bool {
        if let Some(edge) = self.edges.get_mut(edge_id) {
            edge.add_input_node(node);
            true
        } else {
            false
        }
    }

    pub fn add_output_node_to_edge(&mut self, node: NodeUUID, edge_id: &EdgeID) -> bool {
        if let Some(edge) = self.edges.get_mut(edge_id) {
            edge.add_output_node(node);
            true
        } else {
            false
        }
    }

    pub fn remove_input_node_from_edge(&mut self, node: &NodeUUID, edge: &EdgeID) {
        if let Some(edge) = self.edges.get_mut(edge) {
            edge.remove_input_node(node);
        }
    }

    pub fn remove_node(&mut self, node: &NodeUUID) {
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
                    if let Some(edge_set) = self.input_dim_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_dim_to_edges.get_mut(&out_dim) {
                        edge_set.remove(edge_id);
                    }
                }
                EdgeDirection::Oriented | EdgeDirection::Undirected => {
                    if let Some(edge_set) = self.input_dim_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.input_dim_to_edges.get_mut(&out_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_dim_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_dim_to_edges.get_mut(&out_dim) {
                        edge_set.remove(edge_id);
                    }
                }
                EdgeDirection::Loop => {
                    if let Some(edge_set) = self.input_dim_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                    if let Some(edge_set) = self.output_dim_to_edges.get_mut(&in_dim) {
                        edge_set.remove(edge_id);
                    }
                }
                EdgeDirection::Blob => {
                    for dim in 0..in_dim {
                        if let Some(edge_set) = self.input_dim_to_edges.get_mut(&dim) {
                            edge_set.remove(edge_id);
                        }
                        if let Some(edge_set) = self.output_dim_to_edges.get_mut(&dim) {
                            edge_set.remove(edge_id);
                        }
                    }
                }
                _ => {}
            };
        }
    }

    pub fn remove_output_node_from_edge(&mut self, node: &NodeUUID, edge: &EdgeID) {
        if let Some(edge) = self.edges.get_mut(edge) {
            edge.remove_output_node(node);
        }
    }

    pub fn clone_id(&self) -> GraphID {
        self.id.clone()
    }
    /// Add an edge. There are no real failure modes with just HashMaps and HashSets.
    /// TODO: needs to be modified for different edge types.
    pub fn add_edge(&mut self, edge: SparseEdge<Uuid>) {
        self.add_nodes(edge.total_nodes());
        let e_id = edge.clone_id();
        let in_dim = edge.in_nodes.len();
        let out_dim = edge.out_nodes.len();

        if edge.direction == EdgeDirection::Directed
            || edge.direction == EdgeDirection::Undirected
            || edge.direction == EdgeDirection::Oriented
        {
            self.input_dim_to_edges
                .entry(in_dim)
                .or_default()
                .insert(e_id.clone());
            self.output_dim_to_edges
                .entry(out_dim)
                .or_default()
                .insert(e_id.clone());
            for node in edge.in_nodes.iter() {
                self.node_to_containing_edges
                    .entry(*node)
                    .or_default()
                    .insert(e_id.clone());
            }
            if edge.direction == EdgeDirection::Undirected || edge.direction == EdgeDirection::Oriented {
                self.input_dim_to_edges
                    .entry(out_dim)
                    .or_default()
                    .insert(e_id.clone());
                self.output_dim_to_edges
                    .entry(in_dim)
                    .or_default()
                    .insert(e_id.clone());
                for node in edge.out_nodes.iter() {
                    self.node_to_containing_edges
                        .entry(*node)
                        .or_default()
                        .insert(e_id.clone());
                }
            }
        }

        if edge.direction == EdgeDirection::Loop {
            self.input_dim_to_edges
                .entry(in_dim)
                .or_default()
                .insert(e_id.clone());
            self.output_dim_to_edges
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
                self.input_dim_to_edges
                    .entry(dim)
                    .or_default()
                    .insert(e_id.clone());
                self.output_dim_to_edges
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
        self.nodes.len()
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// iterator over nodes present.
    pub fn node_iter(&self) -> Iter<NodeUUID> {
        self.nodes.iter()
    }

    /// Clone of the node set present
    pub fn nodes(&self) -> HashSet<NodeUUID> {
        self.nodes.clone()
    }

    pub fn covers_nodes(&self, other_nodes: &HashSet<NodeUUID>) -> bool {
        self.nodes.is_superset(other_nodes)
    }

    pub fn edge_vec(&self) -> Vec<EdgeID> {
        self.edges.iter().map(|(id, _)| id.clone()).collect()
    }

    pub fn get_edge_from_id(&self, id: &EdgeID) -> Option<&SparseEdge<Uuid>> {
        self.edges.get(id)
    }

    pub fn get_edge_copy(&self, edge_id: &EdgeID) -> Option<SparseEdge<Uuid>> {
        self.edges.get(edge_id).cloned()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn create_node(&mut self) -> NodeUUID {
        let n = NodeUUID::new_v4();
        self.nodes.insert(n.clone());
        n
    }

    /// Returns a list of EdgeIDs that have the provided node as a given input.
    pub fn get_nodes_containing_edges(&self, node: &NodeUUID) -> HashSet<EdgeID> {
        if let Some(edges) = self.node_to_containing_edges.get(node) {
            edges.clone()
        } else {
            HashSet::new()
        }
    }
    pub fn create_nodes(&mut self, num_nodes: usize) -> HashSet<NodeUUID> {
        let mut added_ids = HashSet::with_capacity(num_nodes);
        for _ in 0..num_nodes {
            added_ids.insert(self.create_node());
        }
        added_ids
    }

    fn map_basis(&self, b: &HashSet<NodeUUID>) -> HgVector<NodeUUID> {
        let mut potential_edges = HashSet::new();
        let input_dim = b.len();
        for node in b.iter() {
            if let Some(edges) = self.node_to_containing_edges.get(node) {
                for potential in edges {
                    if let Some(edges_by_dim) = self.input_dim_to_edges.get(&input_dim) {
                        if edges_by_dim.contains(potential) {
                            potential_edges.insert(potential);
                        }
                    }
                }
            }
        }
        let mut ret = HgVector::new();
        for p in potential_edges {
            if let Some(e) = self.edges.get(p) {
                ret += e.map_basis(b);
            }
        }
        ret
    }

    pub fn map_vec(&self, x: HgVector<NodeUUID>) -> HgVector<NodeUUID> {
        let mut ret = HgVector::new();
        for (basis, coeff) in x.basis() {
            let mut tmp = self.map_basis(&basis.into_iter().collect());
            tmp.multiply_scalar(coeff);
            ret += tmp;
        }
        ret
    }

    pub fn project(&self, dim: usize, v: &mut HgVector<NodeUUID>) {
        v.projector(dim);
    }
}

impl Display for HyperGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let graph_string = self.id.clone().to_string();

        let mut node_string = String::new();
        let nodes: Vec<&NodeUUID> = self.node_iter().collect();
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

    use crate::structs::hypergraph::HyperGraph;


}
