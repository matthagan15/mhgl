use crate::structs::SparseVector;
use crate::traits::*;

pub trait HyperGraph {
    type Node: HgNode;
    /// The underlying basis representation, currently we have binary encoding
    /// of power sets and sparse representation where each node is saved as an
    /// unsized integer, u128's are generated with Uuid crate.
    type Basis;
    type HVector: HgVector;
    fn new() -> Self;
    // fn new_with_num_nodes(num_nodes: usize) -> Self;
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
    fn map_basis(&self, basis: &Self::Basis) -> SparseVector<Self::Node>;
    fn random_basis(&self) -> SparseVector<Self::Node>;
}
