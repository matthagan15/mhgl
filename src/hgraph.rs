
use std::collections::{HashSet};

use uuid::Uuid;

use crate::structs::{
    EdgeDirection, EdgeWeight, GeneroEdge, GeneroGraph, NodeID, SparseBasis, EdgeID,
};

use crate::traits::*;

#[derive(Debug, Clone)]
/// The simplest to use hypergraph structure. Utilizes Uuid to store nodes and
/// uses a sparse representation to store hyperedges. Creating nodes does not
/// fail, unlike PGraph which may run out of storage, and can create and delete nodes, unlike BGraph which is fixed at compile time.
///
/// ## Example Usage
/// ```
/// let hg = HGraph::new();
/// let nodes = hg.create_nodes(10);
/// hg.create_directed_edge(&nodes[0..3], &nodes[0..=1], 1.2);
/// assert_eq!(hg.step(&nodes[0..3]), vec![(HashSet::from(&nodes[0..=1]), 1.2)]);
/// ```
///
/// Currently do not support labeling nodes as no consistent API has been worked out yet.
pub struct HGraph {
    // TODO: Move storage of nodes from underlying graph structure to container structures.
    pub name: String,
    nodes: HashSet<NodeID>,
    pub graph: GeneroGraph<SparseBasis<NodeID>>,
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
        let edges = self.graph.get_containing_edges(&node_basis);
        for edge in edges {
            if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                old_edge.remove_node(&node_basis);
                self.graph.add_edge(old_edge);
            }
        }
        self.nodes.remove(&node);
    }

    pub fn create_edge(
        &mut self,
        inputs: &[NodeID],
        outputs: &[NodeID],
        weight: EdgeWeight,
        direction: EdgeDirection,
    ) -> u128 {
        match direction {
            EdgeDirection::Directed | EdgeDirection::Oriented | EdgeDirection::Undirected => {
                let input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
                let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
                let e = GeneroEdge::from(input_basis, output_basis, weight, direction);
                let id = e.id.clone();
                self.graph.add_edge(e);
                id.as_u128()
            }
            EdgeDirection::Loop | EdgeDirection::Blob => {
                let mut input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
                let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
                input_basis.union_with(&output_basis);
                let e = GeneroEdge::from(input_basis, SparseBasis::new_empty(), weight, direction);
                let id = e.id.clone();
                self.graph.add_edge(e);
                id.as_u128()
            }
        }
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
        out_vector
            .to_tuples()
            .into_iter()
            .map(|(b, w)| (b.to_node_set(), w))
            .collect()
    }
}

impl HyperGraph for HGraph {
    type Basis = SparseBasis<NodeID>;
    fn edges(&self) -> Vec<crate::structs::EdgeID> {
        self.graph.clone_edges()
    }

    fn get_outbound_edges(&self, node: &Self::Basis) -> Vec<EdgeID> {
        self.graph.get_outbound_edges(node).into_iter().collect()
    }

    fn query_edges(&self, input: &Self::Basis, output: &Self::Basis) -> Vec<crate::structs::EdgeID> {
        self.graph.query_edges(input, output)
    }

    fn query_weight(&self, input: &Self::Basis, output: &Self::Basis) -> EdgeWeight {
        self.graph.query_weight(input, output)
    }

    fn map_basis(&self, input: &Self::Basis) -> Vec<(Self::Basis, EdgeWeight)> {
        self.graph.map_basis(input).to_tuples()
    }

    fn map_vector(&self, input: &crate::structs::GeneroVector<Self::Basis>) -> crate::structs::GeneroVector<Self::Basis> {
        self.graph.map(input)
    }
}

mod test {
    use crate::{HGraph, EdgeDirection};

    

    

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
        hg.create_edge(&nodes[0..3], &nodes[0..=1], 1., EdgeDirection::Directed);
        println!("step:{:#?}", hg.step(&nodes[0..3]));
        println!("before removal:\n{:#?}", hg);
        hg.remove_node(nodes[0]);
        println!("post removal:\n{:#?}", hg);
        let b = hg.create_edge(&nodes[5..=9], &[], 2.2, EdgeDirection::Blob);
        println!("post blob:{:#?}", hg);
        println!("step output:\n{:?}", hg.step(&nodes[6..=8]));
        hg.remove_edge(b);
        println!("post blob removal:{:#?}", hg);
    }
}
