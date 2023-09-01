use std::collections::{HashSet, VecDeque};

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
    pub name: String,
    nodes: HashSet<u32>,
    next_usable_node: u32,
    reusable_nodes: VecDeque<u32>,
    graph: GeneroGraph<SparseBasis<u32>>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            name: String::new(),
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

    /// Creates an undirected edge among the given nodes. Duplicate inputs are removed.
    pub fn create_edge(&mut self, nodes: &[u32]) {
        // TODO: This can be made much faster for HGraph if we
        // take a memory hit by storing a HashSet of each
        // subset/edge we have seen.
        let input_basis = SparseBasis::from_slice(nodes);
        if self.graph.query_undirected(&input_basis).len() == 0 {
            let e = GeneroEdge::from(
                input_basis,
                SparseBasis::new_empty(),
                1.0,
                EdgeDirection::Undirected,
            );
            self.graph.add_edge(e);
        }
    }

    pub fn remove_edge(&mut self, nodes: &[u32]) {
        let input_basis = SparseBasis::from_slice(nodes);
        let e = self
            .graph
            .query_undirected(&input_basis);
        if let Some(id) = e.first() {
            self.graph.remove_edge(id);
        }
    }

    pub fn query_edge(&self, nodes: &[u32]) -> bool {
        let input_basis = SparseBasis::from_slice(nodes);
        self.graph.query_undirected(&input_basis).len() > 0
    }

    pub fn query_edge_id(&self, nodes: &[u32]) -> Option<u128> {
        let e = self.graph.query_undirected(&SparseBasis::from_slice(nodes));
        e.first().map(|id| id.as_u128())
    }

    /// Takes a single step in the graph, returning the subsets the given nodes map to with their respective edge weights.
    pub fn step(&self, nodes: &[u32]) -> Vec<(HashSet<u32>, EdgeWeight)> {
        let start_basis = SparseBasis::from(nodes.iter().cloned().collect());
        let out_vector = self.graph.map_basis(&start_basis);
        out_vector
            .to_tuples()
            .into_iter()
            .map(|(b, w)| (b.to_node_set(), w))
            .collect()
    }

    pub fn edges_of_size(&self, card: usize) -> Vec<u128> {
        self.graph
            .edges_of_size(card)
            .into_iter()
            .map(|x| x.as_u128())
            .collect()
    }

    pub fn get_containing_edges(&self, nodes: &[u32]) -> Vec<u128> {
        self.graph
            .get_containing_edges(&SparseBasis::from_slice(nodes))
            .into_iter()
            .map(|id| id.as_u128())
            .collect()
    }

    /// Computes the number of edges that have one vertex in the 
    /// provided `cut_nodes` and one in the remaining set. For example,
    /// an edge with only support on the `cut_nodes` would not count. Neither
    /// would an edge without any nodes in `cut_nodes`.
    pub fn cut(&self, cut_nodes: &[u32]) {}
}

mod test {
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
}
