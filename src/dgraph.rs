use std::collections::{HashSet, VecDeque};

use serde::{Serialize};
use uuid::Uuid;

use crate::structs::{
    EdgeDirection, EdgeID, EdgeWeight, GeneroEdge, GeneroGraph, SparseBasis,
};

use crate::traits::*;

#[derive(Debug, Clone, Serialize)]
/// The simplest to use Directed hyperGraph structure. Encodes nodes as `u32` numbers and
/// uses a sparse representation to store hyperedges.
///
/// ## Example Usage
/// ```
/// let hg = DGraph::new();
/// let nodes = hg.create_nodes(10);
/// hg.create_directed_edge(&nodes[0..3], &nodes[0..=1], 1.2);
/// assert_eq!(hg.step(&nodes[0..3]), vec![(HashSet::from(&nodes[0..=1]), 1.2)]);
/// ```
///
/// Currently do not support labeling nodes.
pub struct DGraph {
    // TODO: Move storage of nodes from underlying graph structure to container structures.
    nodes: HashSet<u32>,
    next_usable_node: u32,
    reusable_nodes: VecDeque<u32>,
    graph: GeneroGraph<SparseBasis<u32>>,
}

impl DGraph {
    pub fn new() -> DGraph {
        DGraph {
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

    /// Creates an edge in the hypergraph with the specified inputs, outputs,
    /// weight, and direction. Returns a unique `u128` that can be used to reference the edge in the future for deletion. Possible edge directions:
    /// - `EdgeDirection::Directed` the most straightforward option.
    /// - `EdgeDirection::Undirected` basically creates two directed edges
    /// but with one having inputs and outputs swapped relative to the other.
    /// - `EdgeDirection::Oriented` same as undirected but the opposite
    /// direction gets mapped with an extra minus sign (-1 * weight)
    /// - `EdgeDirection::Loop` creates a loop that maps the union of the
    /// `inputs`
    /// and `outputs` to itself. To avoid unnecessary nodes being added it is
    /// recommended to simply provid an empty outputs variable.
    /// - `EdgeDirection::Blob` creates a "blob" type edge that maps any subset
    /// of the provided nodes (the union of the passed in `inputs` and
    /// `outputs`) to it's complement within the blob.
    pub fn create_edge(
        &mut self,
        inputs: &[u32],
        outputs: &[u32],
        weight: EdgeWeight,
        direction: EdgeDirection,
    ) -> Uuid {
        let mut input_basis = SparseBasis::from(inputs);
        let mut output_basis = SparseBasis::from(outputs);
        if direction == EdgeDirection::Undirected || direction == EdgeDirection::Loop {
            input_basis.union_with(&output_basis);
            output_basis = SparseBasis::new_empty();
        }
        let e = GeneroEdge::from(input_basis, output_basis, weight, direction);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id
    }

    /// Returns true if the edge was properly removed, false if it was not found.
    pub fn remove_edge(&mut self, edge_id: &Uuid) -> bool {
        let e = self.graph.remove_edge(edge_id);
        e.is_some()
    }

    /// Takes a single step in the graph, returning the subsets the given nodes map to with their respective edge weights.
    pub fn step(&self, nodes: &[u32]) -> Vec<(HashSet<u32>, EdgeWeight)> {
        let start_basis = SparseBasis::from(nodes);
        let out_vector = self.graph.map_basis(&start_basis);
        out_vector
            .to_tuples()
            .into_iter()
            .map(|(b, w)| (b.to_node_set(), w))
            .collect()
    }
}

impl HyperGraph for DGraph {
    type Basis = SparseBasis<u32>;
    fn edges(&self) -> Vec<crate::structs::EdgeID> {
        self.graph.clone_edges()
    }

    fn get_outbound_edges(&self, node: &Self::Basis) -> Vec<EdgeID> {
        self.graph.get_outbound_edges(node).into_iter().collect()
    }

    fn query_edges(
        &self,
        input: &Self::Basis,
        output: &Self::Basis,
    ) -> Vec<crate::structs::EdgeID> {
        self.graph.query_edges(input, output)
    }

    fn query_weight(&self, input: &Self::Basis, output: &Self::Basis) -> EdgeWeight {
        self.graph.query_weight(input, output)
    }

    fn map_basis(&self, input: &Self::Basis) -> Vec<(Self::Basis, EdgeWeight)> {
        self.graph.map_basis(input).to_tuples()
    }

    fn map_vector(
        &self,
        input: &crate::structs::GeneroVector<Self::Basis>,
    ) -> crate::structs::GeneroVector<Self::Basis> {
        self.graph.map(input)
    }
}

mod test {
    use crate::{DGraph, EdgeDirection};

    

    #[test]
    fn test_node_creation_deletion() {
        let mut hg = DGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[0..3], &nodes[0..=1], 1., EdgeDirection::Directed);
        println!("step:{:#?}", hg.step(&nodes[0..3]));
        println!("before removal:\n{:#?}", hg);
        hg.remove_node(nodes[0]);
        println!("post removal:\n{:#?}", hg);
        let b = hg.create_edge(&nodes[5..=9], &[], 2.2, EdgeDirection::Undirected);
        println!("post blob:{:#?}", hg);
        println!("step output:\n{:?}", hg.step(&nodes[6..=8]));
        hg.remove_edge(&b);
        println!("post blob removal:{:#?}", hg);
    }
}
