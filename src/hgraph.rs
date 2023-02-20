use core::num;
use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::structs::{EdgeID, EdgeWeight, NodeID, SparseGraph, GeneroGraph, SparseBasis, GeneroEdge, EdgeDirection};

use crate::traits::*;

#[derive(Debug, Clone)]
struct HGraph {
    // TODO: Move storage of nodes from underlying graph structure to container structures.
    pub name: String,
    nodes: HashSet<NodeID>,
    graph: GeneroGraph<SparseBasis<NodeID>>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            name: String::new(),
            nodes: HashSet::new(),
            graph: GeneroGraph::new(),
        }
    }

    pub fn create_nodes(&mut self, num_nodes: usize) -> Vec<NodeID> {
        let mut ret = Vec::with_capacity(num_nodes);
        for _ in 0..num_nodes {
            let id = Uuid::new_v4();
            self.nodes.insert(id.clone());
            ret.push(id);
        }
        ret
    }

    pub fn remove_node(&mut self, node: NodeID) {
        let node_basis = SparseBasis::from(HashSet::from([node.clone()]));
        // TODO: WRONG! this currently gets edges that exactly match this basis
        // Need a function that gets containing edges, inbound and outbound.
        let edges = self.graph.get_containing_edges(&node_basis);
        for edge in edges {
            if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                old_edge.remove_node(&node_basis);
                self.graph.add_edge(old_edge);
            }
        }
        self.nodes.remove(&node);
    }

    pub fn create_directed_edge(&mut self, inputs: &[NodeID], outputs: &[NodeID], weight: EdgeWeight) -> u128 {
        let mut e = GeneroEdge::new();
        let input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
        e.add_input_nodes(&input_basis);
        
        let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
        e.add_output_nodes(&output_basis);
        e.change_direction(crate::structs::EdgeDirection::Directed);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_blob(&mut self, blob: &[NodeID], weight: EdgeWeight) -> u128 {
        let mut e = GeneroEdge::new();
        let basis = SparseBasis::from(blob.iter().cloned().collect());
        e.change_direction(EdgeDirection::Blob);
        e.add_input_nodes(&basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_loop(&mut self, nodes: &[NodeID], weight: EdgeWeight) -> u128 {
        let mut e = GeneroEdge::new();
        let basis = SparseBasis::from(nodes.iter().cloned().collect());
        e.change_direction(EdgeDirection::Loop);
        e.add_input_nodes(&basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_undirected_edge(&mut self, inputs: &[NodeID], outputs: &[NodeID], weight: EdgeWeight) -> u128 {
        let mut e = GeneroEdge::new();
        let input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
        e.change_direction(EdgeDirection::Undirected);
        e.add_input_nodes(&input_basis);
        
        let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
        e.add_output_nodes(&output_basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_oriented_edge(&mut self, inputs: &[NodeID], outputs: &[NodeID], weight: EdgeWeight) -> u128 {
        let mut e = GeneroEdge::new();
        e.change_direction(EdgeDirection::Oriented);
        let input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
        e.add_input_nodes(&input_basis);
        
        let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
        e.add_output_nodes(&output_basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn remove_edge(&mut self, edge_id: u128) {
        let id = Uuid::from_u128(edge_id);
        let e = self.graph.remove_edge(&id);
        if e.is_some() {
            for node in e.unwrap().nodes() {
                for x in node.node_set() {
                    self.nodes.remove(&x);
                }
            }
        }
    }

    /// Takes a single step in the graph, returning the subsets the given nodes map to with the weight.
    pub fn step(&self, nodes: &[NodeID]) -> Vec<(HashSet<NodeID>, EdgeWeight)> {
        let start_basis = SparseBasis::from(nodes.iter().cloned().collect());
        let out_vector = self.graph.map_basis(&start_basis);
        out_vector.to_tuples().into_iter().map(|(b, w)| {
            (b.to_node_set(), w)
        }).collect()
    }
}


mod test {
    use uuid::Uuid;

    use super::HGraph;

    #[test]
    fn test_hgraph_trait_ergonomics() {
        let h = HGraph::new();
        println!("bytes? {:?}", b"testing");
        println!("{:#?}", h);
    }

    #[test]
    fn test_node_creation_deletion() {
        let mut hg = HGraph::new();
        hg.name = String::from("tester :)");
        let nodes = hg.create_nodes(10);
        hg.create_directed_edge(&nodes[0..3], &nodes[0..=1], 1.);
        println!("nodes:{:#?}", nodes);
        println!("before removal:\n{:#?}", hg);
        hg.remove_node(nodes[0]);
        println!("post removal:\n{:#?}", hg);
        let b = hg.create_blob(&nodes[5..=9], 2.2);
        println!("post blob:{:#?}", hg);
        println!("step output:\n{:?}", hg.step(&nodes[6..=8]));
        hg.remove_edge(b);
        println!("post blob removal:{:#?}", hg);
    }
}
