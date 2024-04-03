use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::{EdgeWeight, GeneroGraph, SparseNodeSet};
use crate::structs::HGraphCore;


use crate::{traits::*, Edge};

pub type NodeID = Uuid;
pub type EdgeID = Uuid;

/// A hypergraph structure that can store key-value pairs
/// for each node and edge. Utilizes Uuid's for nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVGraph {
    hgraph: HGraphCore<Uuid>,
    kv_store: HashMap<Uuid, HashMap<String, String>>,
}

impl KVGraph {
    pub fn new() -> KVGraph {
        KVGraph {
            hgraph: HGraphCore::new(),
            kv_store: HashMap::new(),
        }
    }

    /// Gives the number of edges containing the provided node, where
    /// each edge counts equally regardless of it's cardinality.
    pub fn degree(&self, node: u32) -> usize {
        self.get_containing_edges(&[node]).len()
    }

    pub fn to_disk(&self, path: &Path) {
        let mut s = serde_json::to_string(self).expect("Could not serialize KVGraph.");
        let mut file = File::create(path).expect("Cannot create File.");
        file.write_all(s.as_bytes()).expect("Cannot write");
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        // check if path is a given file
        if path.is_file() == false {
            return None;
        }
        if let Ok(hg_json) = fs::read_to_string(path) {
            if let Ok(serde_out) = serde_json::from_str::<KVGraph>(&hg_json) {
                Some(serde_out)
            } else {
                None
            }
        } else {
            None
        }
    }

    // TODO: Need to overhaul the add_nodes api to panic if new nodes
    // cannot be added. I also do not like the idea of reusing nodes.
    pub fn add_node(&mut self) -> u32 {
        if self.next_usable_node < u32::MAX {
            let ret = self.next_usable_node;
            self.next_usable_node += 1;
            self.hgraph.add_node(ret);
            ret
        } else if self.reusable_nodes.len() > 0 {
            self.reusable_nodes.pop_front().expect("No nodes left.")
        } else {
            panic!("No nodes remaining to be added.")
        }
    }

    /// Adds `num_nodes` nodes to the graph, returning a vector containing
    /// the nodes created. The number of nodes returned may be less than
    /// the number of nodes requested due to the use of u32 to store nodes.
    /// Nodes that get deleted are reused in a First In First Out (FIFO) format.
    // TODO: This should panic if it cannot offer the right amount of nodes.
    // Or return a Ret<Ok, Err> type. That would be the best option.
    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<u32> {
        // TODO: Should the user control what nodes are present? We don't
        // really care what numbers are used to store nodes, so why go through
        // all this hassle
        let mut ret = Vec::with_capacity(num_nodes);
        let mut counter = self.next_usable_node;
        let mut nodes_available = counter < u32::max_number() || self.reusable_nodes.len() > 0;
        while nodes_available && ret.len() < num_nodes {
            // Prefer adding never before seen nodes.
            if counter < u32::max_number() {
                if self.hgraph.node_to_containing_edges.contains_key(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.hgraph.add_node(counter);
                    ret.push(counter);
                }
                counter += 1;
            } else {
                // If the counter has reached the max, then we start reusing nodes
                // TODO: This is rather inefficient, can just cache a boolean
                // if we already added the max value or not.
                if self.hgraph.node_to_containing_edges.contains_key(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.hgraph.add_node(counter);
                    ret.push(counter);
                } else {
                    if let Some(old_node) = self.reusable_nodes.pop_front() {
                        if self.hgraph.node_to_containing_edges.contains_key(&old_node) == false {
                            self.hgraph.add_node(old_node);
                            ret.push(old_node);
                        }
                    }
                }
            }
            nodes_available = counter < u32::max_number() || self.reusable_nodes.len() > 0;
        }
        self.next_usable_node = counter;
        ret
    }

    /// Removes a node from the node set. The deleted node will be added to a
    /// dequeue to be reused later once all possible nodes have been created.
    pub fn remove_node(&mut self, node: u32) {
        if self.hgraph.node_to_containing_edges.contains_key(&node) == false {
            return;
        }
        self.hgraph.remove_node(node);
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: &Vec<u32>) {
        self.hgraph.remove_nodes(nodes);
    }

    pub fn nodes(&self) -> Vec<u32> {
        self.hgraph
        .node_to_containing_edges
        .keys()
        .cloned().collect()
    }

    /// Creates an undirected edge among the given nodes. Duplicate inputs are removed. Allows for duplicate edges. Returns the Uuid of the created edge.
    // TODO: rename to add_edge
    pub fn add_edge(&mut self, nodes: &[u32]) -> Uuid {
        let id = self.hgraph.add_edge(nodes);
        id.expect("Graph busted")
        
    }

    pub fn remove_edge(&mut self, nodes: &[u32]) {
        let e = self.hgraph.query_id(nodes);
        if let Some(id) = e {
            self.hgraph.remove_edge(id);
        }
    }

    pub fn remove_edge_id(&mut self, edge_id: Uuid) {
        self.hgraph.remove_edge(edge_id);
    }

    /// Returns true if the provided nodes form an existing edge in
    /// the graph, false if they do not.
    pub fn query_edge(&self, nodes: &[u32]) -> bool {
        self.hgraph.query(nodes)
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn query_edge_id(&self, edge_id: &Uuid) -> Option<Vec<u32>> {
        self.hgraph.edges.get(edge_id).map(|e| e.node_vec())
    }

    pub fn get_edge_id(&self, nodes: &[u32]) -> Option<Uuid> {
        self.hgraph.query_id(nodes)
    }

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    pub fn edges_of_size(&self, card: usize) -> Vec<Uuid> {
        self.hgraph.edges.iter().filter(|(id, e)| e.len() == card).map(|(id, e)| id).cloned().collect()
    }

    pub fn get_containing_edges(&self, nodes: &[u32]) -> Vec<Uuid> {
        self.hgraph
            .get_containing_edges(nodes)
            .into_iter()
            .collect()
    }

    /// Returns the hyperedges that contain the provided edge, not
    /// including the provided edge.
    /// Ex: Edges = [{a, b, c}, {a,b,c,d}, {a,b}, {a,b,c,d,e}]
    /// star({a,b,c}) = [{a,b,c,d}, {a,b,c,d,e}]
    pub fn star_id(&self, edge_id: &Uuid) -> Vec<Uuid> {
        self.hgraph
            .get_containing_edges_id(edge_id)
            .into_iter()
            .filter(|id| edge_id != id)
            .collect()
    }

    /// Returns a list of all edges in the graph.
    pub fn get_edges(&self) -> Vec<EdgeID> {
        self.hgraph.edges.keys().cloned().collect()
    }

    /// Computes the number of edges that have one vertex in the
    /// provided `cut_nodes` and one in the remaining set. For example,
    /// an edge with only support on the `cut_nodes` would not count. Neither
    /// would an edge without any nodes in `cut_nodes`.
    /// The type `ToSet` is any collection that can be converted to a sparse
    /// set representation.
    ///
    /// Example
    /// ```
    /// let mut hg = KVGraph::new();
    /// let nodes = hg.add_nodes(10);
    /// hg.create_edge(&nodes[..2]);
    /// hg.create_edge(&nodes[..3]);
    /// hg.create_edge(&nodes[..4]);
    /// assert_eq!(hg.cut(&nodes[..2]), 2);
    /// assert_eq!(hg.cut(&nodes[..3]), 1);
    /// assert_eq!(hg.cut(&nodes[..4]), 0);
    /// ```
    pub fn cut(&self, cut_nodes: &[u32]) -> usize
    {
        let cut_as_edge = Edge::from(cut_nodes.clone());
        let mut counted_edges: HashSet<Uuid> = HashSet::new();
        for node in cut_nodes {
            let out_edges: Vec<Uuid> = self
                .hgraph
                .get_containing_edges([*node])
                .into_iter()
                .filter(|e_id| counted_edges.contains(e_id) == false)
                .collect();
            for edge_id in out_edges {
                if let Some(e) = self.hgraph.edges.get(&edge_id) {
                    let intersection = cut_as_edge.intersection(e);
                    if intersection.len() > 0 && intersection.len() < e.len() {
                        counted_edges.insert(edge_id);
                    }
                }
            }
        }
        counted_edges.len()
    }

    /// Computes the link of the provided set. The link of a single
    /// hyperedge is computed using the complement, so a hyperedge
    /// of nodes {a, b, c, d} and a provided `face` of {a, b} would
    /// yield a link of {c, d}. The link of the graph is then the
    /// union of all the links of each hyperedge.
    pub fn link<E>(&self, nodes: &E) -> Option<Vec<Edge<u32>>>
        where E: Into<Edge<u32>>
    {
        todo!()
    }

    /// Returns the set of edge of size less than or equal to `k`,
    /// inclusive. Also note that `k` refers to the cardinality of the
    /// provided sets, not the dimension.
    pub fn k_skeleton(&self, k: usize) -> HashSet<EdgeID> {
        self.hgraph
            .edges
            .iter()
            .filter(|(_, e)| e.len() <= k)
            .map(|(id, _)| id.clone())
            .collect()
    }
}


impl Display for KVGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hgraph.node_to_containing_edges.len() == 0 {
            println!("Graph is empty. Add nodes for more fun.");
            return Ok(());
        }
        let mut s = String::new();
        s.push_str("nodes: [");
        let x: Vec<String> = self
            .hgraph
            .node_to_containing_edges
            .keys()
            .map(|n| n.to_string())
            .collect();
        for ix in 0..x.len() - 1 {
            s.push_str(&x[ix]);
            s.push_str(", ");
        }
        s.push_str(x.last().unwrap());
        s.push_str("]\n");
        s.push_str("edges:\n");
        for e in self.hgraph.edges.values() {
            s.push_str(&e.to_string());
            s.push_str("\n");
        }
        f.write_str(&s)
    }
}

mod test {

    use std::{collections::HashSet, path::Path};

    use crate::KVGraph;

    #[test]
    fn test_creating_and_deleting_nodes() {
        let mut hg = KVGraph::new();
        let first_100 = hg.add_nodes(100);
        assert_eq!(first_100, (0_u32..100_u32).collect::<Vec<u32>>());
        let removed = 99_u32;
        hg.remove_node(removed);
        let one_hundred = hg.add_nodes(1);
        assert_eq!(one_hundred[0], 100_u32);
        // WARNING: The below was performed once to verify accuracy, do not
        // uncomment as this test will take forever.
        // hg.add_nodes((u32::MAX - 101_u32) as usize );
        // let get_removed = hg.add_nodes(1);
        // assert_eq!(get_removed[0], removed);
    }

    #[test]
    fn test_edge_creation_removal() {
        let mut hg = KVGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..5]);
        hg.add_edge(&nodes[0..6]);
        hg.remove_edge(&nodes[0..5]);
        assert!(hg.query_edge(&[nodes[4], nodes[3], nodes[2], nodes[1], nodes[0]]) == false);
        assert!(hg.query_edge(&nodes[0..6]))
    }

    #[test]
    fn test_serialization() {
        let mut hg = KVGraph::new();
        hg.add_nodes(10);
        hg.add_edge(&[0, 1]);
        println!("hg:\n{:}", hg);
        let g = hg.hgraph.clone();
        dbg!(serde_json::to_string(&g).unwrap());
        dbg!(&hg.hgraph);
        let s3 = serde_json::to_string(&hg.next_usable_node)
            .expect("could not serialize next_usable_node");
        let s4 =
            serde_json::to_string(&hg.reusable_nodes).expect("could not serialize reusable_nodes");
        let s5 = serde_json::to_string(&hg.hgraph).expect("could not serialize graph");

        dbg!(s3);
        dbg!(s4);
        dbg!(s5);
    }

    #[test]
    fn test_deserialization() {
        let mut hg = KVGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..2]);
        hg.add_edge(&nodes[0..3]);
        hg.add_edge(&nodes[0..4]);
        hg.add_edge(&nodes[0..5]);
        let mut s = String::new();
        s = serde_json::to_string_pretty(&hg).unwrap();
        println!("s: {:}", s);
        let hg2: KVGraph = serde_json::from_str(&s[..]).unwrap();
        println!("hg2:{:}", hg2);
    }

    #[test]
    fn test_link() {
        let mut hg = KVGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..=5]);
        hg.add_edge(&nodes[5..]);
        let link = hg.link(&HashSet::from([nodes[5], nodes[4]]));
        println!("hg\n{:}", hg);
        dbg!(link);
    }

    #[test]
    fn test_skeleton() {
        let mut hg = KVGraph::new();
        let nodes = hg.add_nodes(10);
        for size in 0..8 {
            hg.add_edge(&nodes[0..=size]);
        }
        for size in 1..10 {
            println!("{:}-skeleton", size);
            println!("{:?}", hg.k_skeleton(size));
        }
    }

    fn simple_test_hg() -> KVGraph {
        let mut hg = KVGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..=5]);
        hg.add_edge(&nodes[5..]);
        hg
    }
    #[test]
    fn test_cut_with_traits() {
        let mut hg = KVGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[..2]);
        hg.add_edge(&nodes[..3]);
        hg.add_edge(&nodes[..4]);
        println!("hg\n{:}", hg);
        assert_eq!(hg.cut(&nodes[..2]), 2);
        assert_eq!(hg.cut(&nodes[..3]), 1);
        assert_eq!(hg.cut(&nodes[..4]), 0);
    }

    #[test]
    fn test_node_as_edge() {
        let mut hg = KVGraph::new();
        let nodes = hg.add_nodes(3);
        let e0 = hg.add_edge(&[0]);
        let e1 = hg.add_edge(&[0, 1]);
        let e2 = hg.add_edge(&[0, 1, 2]);
        let star = hg.star_id(&e0);
        dbg!(star);
        dbg!(&hg);
        println!(
            "degrees in order: {:}, {:}, {:}.",
            hg.degree(0),
            hg.degree(1),
            hg.degree(2)
        )
    }

    #[test]
    fn test_kvgraph() {
        let hg = KVGraph::new();
        let n1 = hg.add_node();
        hg.store(n1, "key", "value");
        assert_eq(hg.query(n1, "key"), String::from("value"));
    }
}
