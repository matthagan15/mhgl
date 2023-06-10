use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    structs::{BitBasis, EdgeWeight, GeneroEdge, GeneroGraph},
    traits::{HgBasis, HyperGraph},
    EdgeDirection,
};

/// A directed, weighted hypergraph utilizing a binary encoding of subsets.
/// Should be advantageous for dense hypergraphs over a small number of nodes.
#[derive(Debug, Clone)]
pub struct BGraph {
    pub name: String,
    num_nodes: usize,
    graph: GeneroGraph<BitBasis>,
}

impl BGraph {
    pub fn new(num_nodes: usize) -> Self {
        BGraph {
            name: "".to_string(),
            num_nodes: num_nodes,
            graph: GeneroGraph::new(),
        }
    }

    pub fn create_edge(
        &mut self,
        inputs: &[usize],
        outputs: &[usize],
        weight: EdgeWeight,
        direction: EdgeDirection,
    ) -> u128 {
        let mut input_basis = BitBasis::from(self.num_nodes, inputs.iter().cloned().collect());
        let mut output_basis = BitBasis::from(self.num_nodes, outputs.iter().cloned().collect());
        if direction == EdgeDirection::Loop || direction == EdgeDirection::Undirected {
            input_basis.union_with(&output_basis);
            output_basis = BitBasis::new(0);
        }
        let e = GeneroEdge::from(input_basis, output_basis, weight, direction);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn remove_edge(&mut self, edge_id: u128) -> bool {
        let id = Uuid::from_u128(edge_id);
        let e = self.graph.remove_edge(&id);
        e.is_some()
    }

    /// Takes a step from the given subset of nodes, returning an output `Vec`
    /// consisting of tuples of node subsets and their corresponding weights.
    pub fn step(&self, nodes: &[usize]) -> Vec<(HashSet<usize>, EdgeWeight)> {
        let start_basis = BitBasis::from(self.num_nodes, nodes.iter().cloned().collect());
        let out_vector = self.graph.map_basis(&start_basis);
        out_vector
            .to_tuples()
            .into_iter()
            .map(|(b, w)| (b.to_usize_set(), w))
            .collect()
    }
}

impl HyperGraph for BGraph {
    type Basis = BitBasis;

    fn edges(&self) -> Vec<crate::structs::EdgeID> {
        self.graph.clone_edges()
    }

    fn get_outbound_edges(&self, node: &Self::Basis) -> Vec<crate::structs::EdgeID> {
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

mod tests {
    use super::BGraph;

    #[test]
    fn basic_step_test() {
        let mut bg = BGraph::new(5);
        bg.create_edge(
            &[0, 1, 2, 3],
            &[1, 2, 3, 4],
            1.,
            crate::EdgeDirection::Directed,
        );
        let output = bg.step(&[0, 1, 2, 3]);
        println!("output:\n{:?}", output);

        dbg!(bg);
    }
}
