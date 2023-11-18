use core::prelude::v1;
use std::collections::{HashSet, VecDeque};
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::{
    EdgeDirection, EdgeID, EdgeWeight, GeneroEdge, GeneroGraph, NodeID, SparseBasis,
};

use crate::traits::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The simplest to use hypergraph structure. An Undirected and unweighted variant
/// that utilizes u32's for nodes. The directed variant of `HGraph` is
/// `DGraph`. For smaller memory footprints, use
/// `UGraph<N>` for undirected graphs or `PGraph<N>` for directed variants.
/// Duplicate edges are disallowed.
/// ## Example Usage
/// ```
/// let hg = HGraph::new();
/// let nodes = hg.create_nodes(10);
/// hg.create_directed_edge(&nodes[0..3], &nodes[0..=1], 1.2);
/// assert_eq!(hg.step(&nodes[0..3]), vec![(HashSet::from(&nodes[0..=1]), 1.2)]);
/// ```
///
/// Currently do not support labeling nodes.
/// Here is how to store labeled data
/// ```
/// let mut hg = HGraph::new();
/// let mut hm: HashMap<Uuid, NodeType> = HashMap::new();
/// let node_data: Vec<NodeType> = data_set.load();
/// let node_ids: Vec<Uuid> = HGraph::add_nodes(node_data.len());
/// for ix in node_data.into_iter() {
///     hm.insert(node_ids[ix], node_data[ix])
/// }
/// ```
/// Then data can be accessed by querying `hm[id]`.
pub struct HGraph {
    nodes: HashSet<u32>,
    next_usable_node: u32,
    reusable_nodes: VecDeque<u32>,
    graph: GeneroGraph<SparseBasis<u32>>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            nodes: HashSet::new(),
            next_usable_node: 0,
            reusable_nodes: VecDeque::new(),
            graph: GeneroGraph::new(),
        }
    }

    /// Adds `num_nodes` nodes to the graph, returning a vector containing
    /// the nodes created. The number of nodes returned may be less than
    /// the number of nodes requested due to the use of u32 to store nodes.
    /// Nodes that get deleted are reused in a First In First Out (FIFO) format.
    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<u32> {
        let mut ret = Vec::with_capacity(num_nodes);
        let mut counter = self.next_usable_node;
        let mut nodes_available = counter < u32::max_number() || self.reusable_nodes.len() > 0;
        while nodes_available && ret.len() < num_nodes {
            // Prefer adding never before seen nodes.
            if counter < u32::max_number() {
                if self.nodes.contains(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.nodes.insert(counter);
                    ret.push(counter);
                }
                counter += 1;
            } else {
                // If the counter has reached the max, then we start reusing nodes
                // TODO: This is rather inefficient, can just cache a boolean
                // if we already added the max value or not.
                if self.nodes.contains(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.nodes.insert(counter);
                    ret.push(counter);
                } else {
                    if let Some(old_node) = self.reusable_nodes.pop_front() {
                        if self.nodes.contains(&old_node) == false {
                            self.nodes.insert(old_node);
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
        if self.nodes.contains(&node) == false {
            return;
        }
        let node_basis = SparseBasis::from(HashSet::from([node]));
        let edges = self.graph.get_containing_edges(&node_basis);
        for edge in edges {
            if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                old_edge.remove_node(&node_basis);
                self.graph.add_edge(old_edge);
            }
        }
        self.nodes.remove(&node);
        self.reusable_nodes.push_back(node);
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: Vec<u32>) {
        for node in nodes {
            self.remove_node(node);
        }
    }

    pub fn nodes(&self) -> Vec<u32> {
        self.nodes.clone().into_iter().collect()
    }

    /// Creates an undirected edge among the given nodes. Duplicate inputs are removed. Allows for duplicate edges. Returns the Uuid of the created edge.
    pub fn create_edge(&mut self, nodes: &[u32]) -> Uuid {
        // TODO: This can be made much faster for HGraph if we
        // take a memory hit by storing a HashSet of each
        // subset/edge we have seen.
        let input_basis = SparseBasis::from_slice(nodes);
        let e: GeneroEdge<SparseBasis<u32>> = input_basis.into();
        let id = e.id.clone();
        self.graph.add_edge(e);
        id
    }

    pub fn remove_edge(&mut self, nodes: &[u32]) {
        let input_basis = SparseBasis::from_slice(nodes);
        let e = self.graph.query_undirected(&input_basis);
        if let Some(id) = e.first() {
            self.graph.remove_edge(id);
        }
    }

    pub fn query_edge(&self, nodes: &[u32]) -> bool {
        let input_basis = SparseBasis::from_slice(nodes);
        self.graph.query_undirected(&input_basis).len() > 0
    }

    pub fn query_edge_id(&self, id: &Uuid) -> Option<Vec<u32>> {
        if let Some(e) = self.graph.query_edge(id) {
            Some(e.in_nodes.to_node_vec())
        } else {
            None
        }
    }

    pub fn get_edge_id(&self, nodes: &[u32]) -> Option<Uuid> {
        let e = self.graph.query_undirected(&SparseBasis::from_slice(nodes));
        e.first().copied()
    }

    /// Computes the link of the provided nodes in the HyperGraph but returns a
    /// list of sets as opposed to a new HyperGraph.
    pub fn link_as_vec(&self, nodes: &[u32]) -> Vec<(HashSet<u32>, EdgeWeight)> {
        let start_basis = SparseBasis::from(nodes.iter().cloned().collect());
        let out_vector = self.graph.map_basis(&start_basis);
        out_vector
            .to_tuples()
            .into_iter()
            .map(|(b, w)| (b.to_node_set(), w))
            .collect()
    }

    pub fn edges_of_size(&self, card: usize) -> Vec<Uuid> {
        self.graph.edges_of_size(card).into_iter().collect()
    }

    pub fn get_containing_edges(&self, nodes: &[u32]) -> Vec<Uuid> {
        self.graph
            .get_containing_edges(&SparseBasis::from_slice(nodes))
            .into_iter()
            .collect()
    }

    pub fn get_edges(&self) -> Vec<(Uuid, HashSet<u32>)> {
        let edge_ids = self.graph.clone_edges();
        edge_ids
            .into_iter()
            .filter_map(|id| {
                self.graph
                    .query_edge(&id)
                    .map(|edge| (edge.id, edge.in_nodes.to_node_set()))
            })
            .collect()
    }

    /// Computes the number of edges that have one vertex in the
    /// provided `cut_nodes` and one in the remaining set. For example,
    /// an edge with only support on the `cut_nodes` would not count. Neither
    /// would an edge without any nodes in `cut_nodes`.
    pub fn cut(&self, cut_nodes: &[u32]) -> f64 {
        let mut counted_edges: HashSet<Uuid> = HashSet::new();
        for node in cut_nodes {
            let out_edges = self.graph.get_outbound_edges(&SparseBasis::from_node(node));
            for edge_id in out_edges {
                if let Some(e) = self.graph.query_edge(&edge_id) {}
            }
        }
        0.0
    }

    /// Computes the link of the provided set.
    pub fn link(&self, face: HashSet<u32>) -> HGraph {
        let v: Vec<u32> = face.clone().into_iter().collect();
        let face_basis = SparseBasis::from_slice(&v[..]);
        let out = self.graph.map_basis(&face_basis);
        let mut link = HGraph {
            nodes: self.nodes.clone(),
            next_usable_node: self.next_usable_node,
            reusable_nodes: self.reusable_nodes.clone(),
            graph: GeneroGraph::new(),
        };
        for (b, _) in out.to_tuples() {
            link.create_edge(&b.node_vec()[..]);
        }
        link
    }

    /// Computes the k-skeleton of this hypergraph and returns a new `HGraph`.
    /// To mutate a given `HGraph` use `HGraph::project_onto`
    pub fn k_skeleton(&self, k: usize) -> HGraph {
        let mut ret = HGraph::new();
        let mut new_graph = self.graph.clone();
        new_graph.edges = new_graph.edges.into_iter().filter(|(_, e)| {
            e.input_cardinality() <= k + 1
        }).collect();
        ret.nodes = self.nodes.clone();
        ret.next_usable_node = self.next_usable_node;
        ret.reusable_nodes = self.reusable_nodes.clone();
        ret.graph = new_graph;
        ret
    }
}

impl Display for HGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str("nodes: [");
        let x: Vec<String> = self
            .nodes
            .clone()
            .into_iter()
            .map(|n| n.to_string())
            .collect();
        for ix in 0..x.len() - 1 {
            s.push_str(&x[ix]);
            s.push_str(", ");
        }
        s.push_str(x.last().unwrap());
        s.push_str("]\n");
        s.push_str("edges:\n");
        for e in self.graph.clone_edges() {
            let e = self.graph.query_edge(&e).unwrap();
            s.push_str(&e.in_nodes.to_string());
            s.push_str("\n");
        }
        f.write_str(&s)
    }
}

mod test {
    use std::collections::{HashMap, HashSet, VecDeque};

    use crate::{EdgeDirection, HGraph};

    #[test]
    fn test_creating_and_deleting_nodes() {
        let mut hg = HGraph::new();
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
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[0..5]);
        hg.create_edge(&nodes[0..6]);
        hg.remove_edge(&nodes[0..5]);
    }

    #[test]
    fn test_serialization() {
        let mut hg = HGraph::new();
        hg.add_nodes(10);
        hg.create_edge(&[0, 1]);
        println!("hg:\n{:}", hg);
        let g = hg.graph.clone();
        dbg!(serde_json::to_string(&g).unwrap());
        dbg!(&hg.graph);
        let s2 = serde_json::to_string(&hg.nodes).expect("could not serialize nodes");
        let s3 = serde_json::to_string(&hg.next_usable_node)
            .expect("could not serialize next_usable_node");
        let s4 =
            serde_json::to_string(&hg.reusable_nodes).expect("could not serialize reusable_nodes");
        let s5 = serde_json::to_string(&hg.graph).expect("could not serialize graph");

        dbg!(s2);
        dbg!(s3);
        dbg!(s4);
        dbg!(s5);
    }

    #[test]
    fn test_link() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[0..=5]);
        hg.create_edge(&nodes[5..]);
        let link = hg.link(HashSet::from([nodes[5], nodes[4]]));
        println!("hg\n{:}", hg);
        println!("link\n{:}", link);
    }

    #[test]
    fn test_skeleton() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        for size in 0..8 {
            hg.create_edge(&nodes[0..=size]);
        }
        for size in 1..10 {
            println!("{:}-skeleton", size);
            println!("{:}", hg.k_skeleton(size));
        }
    }
}
