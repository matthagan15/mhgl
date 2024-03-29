use std::collections::HashSet;

use serde::{Serialize};
use uuid::Uuid;

use crate::{
    structs::{EdgeDirection, EdgeID, EdgeWeight, GeneroEdge, GeneroGraph, SparseNodeSet},
    traits::{HgBasis, HgNode, HyperGraph},
};

#[derive(Debug, Clone, Serialize)]
/// # UNDER CONSTRUCTION
/// Hypergraph variant that allows for other integer types to store
/// nodes, which allows for either smaller memory footprints for smaller
/// graphs or for more nodes in larger graphs.
/// This functionality is pulled into a different type (PGraph) since we use smaller
/// integer types however, this means
/// that adding nodes could possibly fail. This results in a different API than
/// a generic
pub struct PGraph<N: HgNode> {
    pub name: String,
    nodes: HashSet<N>,
    next_usable_node: N,
    reusable_nodes: Vec<N>,
    graph: GeneroGraph<SparseNodeSet<N>>,
}

impl<N: HgNode> PGraph<N> {
    pub fn new() -> Self {
        PGraph {
            name: "".to_string(),
            nodes: HashSet::new(),
            next_usable_node: N::zero(),
            reusable_nodes: Vec::new(),
            graph: GeneroGraph::new(),
        }
    }

    /// May return no nodes if they cannot be created. For example, using u8 as the underlying storage method means only 255 nodes can be created. If you try adding more nodes after this then you get nothing back. Also it will reuse nodes if they are deleted.
    pub fn create_nodes(&mut self, num_nodes: usize) -> Option<HashSet<N>> {
        let mut ret = HashSet::with_capacity(num_nodes);
        let mut counter = self.next_usable_node;
        while ret.len() < num_nodes && counter < N::max_number() {
            if self.nodes.contains(&counter) && self.reusable_nodes.len() > 0 {
                if let Some(new_node) = self.reusable_nodes.pop() {
                    ret.insert(new_node);
                    self.nodes.insert(new_node);
                }
            } else if self.nodes.contains(&counter) && self.reusable_nodes.len() == 0 {
                counter.plus_one();
            } else if self.nodes.contains(&counter) == false {
                self.nodes.insert(counter);
                ret.insert(counter);
                counter.plus_one();
            }
        }
        if ret.len() > 0 {
            self.next_usable_node = counter;
            Some(ret)
        } else {
            None
        }
    }

    pub fn remove_node(&mut self, node: N) {
        if self.nodes.remove(&node) {
            let node_basis = SparseNodeSet::from(HashSet::from([node.clone()]));
            let edges = self.graph.get_containing_edges(&node_basis);
            for edge in edges {
                if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                    old_edge.remove_node(&node_basis);
                    self.graph.add_edge(old_edge);
                }
            }
            self.reusable_nodes.push(node);
        }
    }

    pub fn create_edge(
        &mut self,
        inputs: &[N],
        outputs: &[N],
        weight: EdgeWeight,
        direction: EdgeDirection,
    ) -> u128 {
        match direction {
            EdgeDirection::Directed | EdgeDirection::Symmetric => {
                let input_basis = SparseNodeSet::from(inputs);
                let output_basis = SparseNodeSet::from(outputs);
                let e = GeneroEdge::from(input_basis, output_basis, weight, direction);
                let id = self.graph.add_edge(e);
                id.as_u128()
            }
            EdgeDirection::Loop | EdgeDirection::Undirected | EdgeDirection::Simplicial => {
                let mut input_basis = SparseNodeSet::from(inputs);
                let output_basis = SparseNodeSet::from(outputs);
                input_basis.union_with(&output_basis);
                let e = GeneroEdge::from(input_basis, SparseNodeSet::new_empty(), weight, direction);
                let id = self.graph.add_edge(e);
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
    // TODO: There is no neighbors function? No Walk?
}

impl<N: HgNode> HyperGraph for PGraph<N> {
    type Basis = SparseNodeSet<N>;
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
    use crate::PGraph;

    

    #[test]
    fn test_node_creation_deletion() {
        let mut pg = PGraph::<u8>::new();
        let mut nodes: Vec<_> = pg
            .create_nodes(1000)
            .expect("no nodes?")
            .into_iter()
            .collect();
        nodes.sort();
        println!("nodes! {:?}", nodes);
        assert!(pg.create_nodes(1).is_none())
    }
}
