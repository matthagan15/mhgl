use crate::structs::SparseVector;
use crate::traits::*;

pub trait HyperGraph {
    type Node: HgNode;
    /// The underlying basis representation, currently we have binary encoding
    /// of power sets and sparse representation where each node is saved as an
    /// unsized integer, u128's are generated with Uuid crate.
    type Basis: HgBasis;
    type HVector: HgVector;
    // fn add_node(node: Self::Node); // What if node is already present?
    // fn add_nodes(nodes: Vec<Self::Node>); // What if a single node is already present?
    // fn create_node(&mut self);
    // fn create_nodes(&mut self, num_nodes: usize);
    // fn remove_node(&mut self, node: Self::Node);
    // fn remove_nodes(&mut self, nodes: Vec<Self::Node>);
    // fn has_node(&self, node: &Self::Node);
    // fn has_nodes(&self, nodes: Vec<&Self::Node>);
    // fn edges(&self) -> Vec<EdgeID>;
    // fn get_outbound_edges(&self, node: &Self::Basis) -> HashMap<Self::Basis, EdgeWeight>;
    // fn edges_with_input_cardinality(&self, cardinality: usize) -> Vec<EdgeID>;
    // fn edges_with_output_cardinality(&self, cardinality: usize) -> Vec<EdgeID>;
    // fn contains_edge(&self, input: Self::Basis, output: Self::Basis) -> bool;
    // fn get_weight_of_all_edges(&self, input: Self::Basis, output: Self::Basis) -> EdgeWeight;
    // fn map_basis(&self, basis: &Self::Basis) -> SparseVector<Self::Node>;
    // fn random_basis(&self) -> SparseVector<Self::Node>;
    // fn random_step(&self, start: &mut Self::HVector);
    // fn random_basis_step(&self, start: &Self::Basis) -> Self::Basis;

    // TODO: Thoughts on a closure type hypergraph? One in which you give it a function
    // from basis elements to basis elements to the EdgeWeight type. Then when you need to
    // take a step or actually traverse the hypergraph you just call the closure. Would be
    // useful for complete hypergraphs on more than ~ 17 nodes, you essentially are trading off
    // memory for time, because if you store the hypergraph then you are fucked past 17 ish
    // nodes, similar to number of qubits.
}
