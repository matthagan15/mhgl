
use std::collections::{HashSet};

use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::structs::{
    EdgeDirection, EdgeWeight, GeneroEdge, GeneroGraph, NodeID, SparseBasis, EdgeID,
};

use crate::traits::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The simplest to use hypergraph structure. Encodes nodes as `u128` numbers (via the `uuid` crate internally, converted to `u128` for end user)  and
/// uses a sparse representation to store hyperedges. Creating nodes does not
/// fail, unlike `PGraph<N>` which may run out of encodable nodes if small enough integer sizes are used. With `HGraph` you can also create and delete nodes, unlike `BGraph` which is fixed at compile time.
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
    nodes: HashSet<u128>,
    pub graph: GeneroGraph<SparseBasis<u128>>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            name: String::new(),
            nodes: HashSet::new(),
            graph: GeneroGraph::new(),
        }
    }

    pub fn create_nodes(&mut self, num_nodes: usize) -> Vec<u128> {
        let mut ret = Vec::with_capacity(num_nodes);
        for _ in 0..num_nodes {
            let id = Uuid::new_v4();
            self.nodes.insert(id.as_u128());
            ret.push(id.as_u128());
        }
        ret
    }

    pub fn remove_node(&mut self, node: u128) {
        let node_basis = SparseBasis::from(HashSet::from([node]));
        let edges = self.graph.get_containing_edges(&node_basis);
        for edge in edges {
            if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                old_edge.remove_node(&node_basis);
                self.graph.add_edge(old_edge);
            }
        }
        self.nodes.remove(&node);
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
        inputs: &[u128],
        outputs: &[u128],
        weight: EdgeWeight,
        direction: EdgeDirection,
    ) -> u128 {
        let mut input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
        let mut output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
        if direction == EdgeDirection::Blob || direction == EdgeDirection::Loop {
            input_basis.union_with(&output_basis);
            output_basis = SparseBasis::new_empty();
        }
        let e = GeneroEdge::from(input_basis, output_basis, weight, direction);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    /// Returns true if the edge was properly removed, false if it was not found.
    pub fn remove_edge(&mut self, edge_id: u128) -> bool {
        let id = Uuid::from_u128(edge_id);
        let e = self.graph.remove_edge(&id);
        e.is_some()
    }

    /// Takes a single step in the graph, returning the subsets the given nodes map to with their respective edge weights.
    pub fn step(&self, nodes: &[u128]) -> Vec<(HashSet<u128>, EdgeWeight)> {
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
    type Basis = SparseBasis<u128>;
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
