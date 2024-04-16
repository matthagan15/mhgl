use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{hgraph::EdgeID, structs::HGraphCore, EdgeSet};

/// A generic hypergraph over (N)ode and (E)dge datatypes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NEGraph<NodeData, EdgeData> {
    next_usable_node: u32,
    reusable_nodes: VecDeque<u32>,
    core: HGraphCore<u32, NodeData, EdgeData>,
}

impl<NodeData, EdgeData> NEGraph<NodeData, EdgeData> {
    pub fn new() -> Self {
        Self {
            next_usable_node: 0,
            reusable_nodes: VecDeque::new(),
            core: HGraphCore::new(),
        }
    }

    // TODO: Need to overhaul the add_nodes api to panic if new nodes
    // cannot be added. I also do not like the idea of reusing nodes.
    pub fn add_node(&mut self, data: NodeData) -> u32 {
        if self.next_usable_node < u32::MAX {
            let ret = self.next_usable_node;
            self.next_usable_node += 1;
            self.core.add_node(ret, data);
            ret
        } else if self.reusable_nodes.len() > 0 {
            self.reusable_nodes.pop_front().expect("No nodes left.")
        } else {
            panic!("No nodes remaining to be added.")
        }
    }

    /// Panics if cannot add all nodes.
    /// TODO: The proper thing to do is return a
    /// result with the OK variant being the Vec<u32> of each node added and
    /// the Error variant containing the vec of the nodes added and
    /// all the data that was not properly added.
    pub fn add_nodes(&mut self, data: Vec<NodeData>) -> Vec<u32> {
        // TODO: Should the user control what nodes are present? We don't
        // really care what numbers are used to store nodes, so why go through
        // all this hassle
        let num_nodes = data.len();
        let mut data = data;
        data.reverse();
        let mut ret = Vec::with_capacity(num_nodes);
        let mut counter = self.next_usable_node;
        let mut nodes_available = counter < u32::MAX || self.reusable_nodes.len() > 0;
        while nodes_available && ret.len() < num_nodes {
            // Prefer adding never before seen nodes.
            if counter < u32::MAX {
                if self.core.nodes.contains_key(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.core.add_node(counter, data.pop().unwrap());
                    ret.push(counter);
                }
                counter += 1;
            } else {
                // If the counter has reached the max, then we start reusing nodes
                // TODO: This is rather inefficient, can just cache a boolean
                // if we already added the max value or not.
                if self.core.nodes.contains_key(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.core.add_node(counter, data.pop().unwrap());
                    ret.push(counter);
                } else {
                    if let Some(old_node) = self.reusable_nodes.pop_front() {
                        if self.core.nodes.contains_key(&old_node) == false {
                            self.core.add_node(old_node, data.pop().unwrap());
                            ret.push(old_node);
                        }
                    }
                }
            }
            nodes_available = counter < u32::MAX || self.reusable_nodes.len() > 0;
        }
        self.next_usable_node = counter;
        ret
    }

    /// returns the data with the associated node, panics
    /// if the node is not found because your dumb ass deserves
    /// it.
    pub fn node_ref(&self, node: &u32) -> &NodeData {
        &self.core.nodes.get(node).expect("Node not found").data
    }

    pub fn node_ref_mut(&mut self, node: &u32) -> &mut NodeData {
        &mut self.core.nodes.get_mut(node).expect("Node not found").data
    }

    pub fn edge_ref(&self, edge_id: &EdgeID) -> &EdgeData {
        &self.core.edges.get(edge_id).expect("Edge not found").data
    }

    pub fn edge_ref_mut(&mut self, edge_id: &EdgeID) -> &EdgeData {
        &mut self
            .core
            .edges
            .get_mut(edge_id)
            .expect("Edge not found")
            .data
    }

    /// Returns data of associated node, allowing the node for reuse.
    /// returns `None` if the node is not present
    pub fn remove_node(&mut self, node: &u32) -> Option<NodeData> {
        self.core.nodes.remove(node).map(|old_data| old_data.data)
    }
    pub fn nodes(&self) -> Vec<u32> {
        self.core.nodes.keys().cloned().collect()
    }

    pub fn add_edge<E>(&mut self, nodes: E, data: EdgeData) -> Uuid
    where
        E: Into<EdgeSet<u32>>,
    {
        self.core.add_edge(nodes, data).expect("Could not edge")
    }

    pub fn remove_edge(&mut self, edge_id: EdgeID) -> Option<EdgeData> {
        self.core.remove_edge(edge_id).map(|edge| edge.data)
    }

    pub fn query_edge<E>(&self, edge: E) -> bool
    where
        E: Into<EdgeSet<u32>>,
    {
        self.core.query(edge)
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn query_edge_id(&self, edge_id: &Uuid) -> Option<Vec<u32>> {
        self.core.edges.get(edge_id).map(|e| e.nodes.node_vec())
    }

    pub fn get_edge_id<E>(&self, edge: E) -> Option<Uuid>
    where
        E: Into<EdgeSet<u32>>,
    {
        self.core.query_id(edge)
    }

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    pub fn edges_of_size(&self, card: usize) -> Vec<Uuid> {
        self.core
            .edges
            .iter()
            .filter(|(id, e)| e.nodes.len() == card)
            .map(|(id, e)| id)
            .cloned()
            .collect()
    }

    pub fn get_containing_edges<E>(&self, edge: E) -> Vec<Uuid>
    where
        E: Into<EdgeSet<u32>>,
    {
        self.core
            .get_containing_edges_strict(edge)
            .into_iter()
            .collect()
    }

    /// Returns the hyperedges that contain the provided edge, not
    /// including the provided edge.
    /// Ex: Edges = [{a, b, c}, {a,b,c,d}, {a,b}, {a,b,c,d,e}]
    /// star({a,b,c}) = [{a,b,c,d}, {a,b,c,d,e}]
    pub fn get_containing_edges_id(&self, edge_id: &Uuid) -> Vec<Uuid> {
        self.core
            .get_containing_edges_strict_id(edge_id)
            .into_iter()
            .collect()
    }

    /// Returns a list of all edges in the graph.
    pub fn get_edges(&self) -> Vec<EdgeID> {
        self.core.edges.keys().cloned().collect()
    }

    /// Computes the link of the provided set. The link of a single
    /// hyperedge is computed using the complement, so a hyperedge
    /// of nodes {a, b, c, d} and a provided `face` of {a, b} would
    /// yield a link of {c, d}. The link of the graph is then the
    /// union of all the links of each hyperedge.
    pub fn link<E>(&self, nodes: E) -> Vec<EdgeSet<u32>>
    where
        E: Into<EdgeSet<u32>>,
    {
        self.core
            .link(nodes)
            .into_iter()
            .map(|(_, edge)| edge)
            .collect()
    }

    /// Returns the set of edge of size less than or equal to `k`,
    /// inclusive. Also note that `k` refers to the cardinality of the
    /// provided sets, not the dimension.
    pub fn k_skeleton(&self, k: usize) -> HashSet<EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= k)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn change_node_data(&mut self, node: &u32, new_data: NodeData) -> Option<NodeData> {
        self.core.change_node_data(node, new_data)
    }

    pub fn change_edge_data(&mut self, edge_id: &EdgeID, new_data: EdgeData) -> Option<EdgeData> {
        self.core.change_edge_data(edge_id, new_data)
    }
}

impl<NodeData, EdgeData> NEGraph<NodeData, EdgeData>
where
    NodeData: Serialize + for<'a> Deserialize<'a>,
    EdgeData: Serialize + for<'a> Deserialize<'a>,
{
    pub fn to_disk(&self, path: &Path) {
        let s = serde_json::to_string(self).expect("could not serialize NEGraph");
        let mut file = File::create(path).expect("Cannot create File.");
        file.write_all(s.as_bytes()).expect("Cannot write");
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        if path.is_file() == false {
            return None;
        }
        if let Ok(hg_string) = std::fs::read_to_string(path) {
            if let Ok(serde_out) = serde_json::from_str(&hg_string) {
                Some(serde_out)
            } else {
                None
            }
        } else {
            None
        }
    }
}

mod tests {
    use super::NEGraph;

    #[test]
    fn test_node_creation() {
        let mut hg = NEGraph::<String, String>::new();
        hg.add_node(String::from("node 1"));
        hg.add_nodes(vec![String::from("node 2"), String::from("node 3")]);
        dbg!(hg);
    }
}
