use std::collections::HashMap;

use uuid::Uuid;

use crate::structs::{nodes::NodeID, EdgeID, EdgeWeight, NodeUUID, SparseGraph};

use super::SparseVector;

#[derive(Debug, Clone)]
struct EZGraph<N: NodeID> {
    pub name: String,
    graph: SparseGraph<N>,
    id_to_label: HashMap<Uuid, String>,
    label_to_id: HashMap<String, Uuid>,
}

impl<N: NodeID> EZGraph<N> {
    pub fn new() -> EZGraph<N> {
        EZGraph {
            graph: SparseGraph::new(),
            name: String::new(),
            id_to_label: HashMap::new(),
            label_to_id: HashMap::new(),
        }
    }

    // pub fn create_node(&mut self, label: String) {
    //     let id = self.graph.create_node();
    //     self.id_to_label.insert(id.clone(), label.clone());
    //     self.label_to_id.insert(label, id);
    // }
}

pub trait HyperGraph {
    type Node: NodeID;
    /// The underlying basis representation, currently we have binary encoding
    /// of power sets and sparse representation where each node is saved as an
    /// unsized integer, u128's are generated with Uuid crate.
    type Basis;
    // type HVector<Basis>;
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

mod test {
    use uuid::Uuid;

    use crate::structs::hgraph::EZGraph;

    #[test]
    fn test_hgraph_trait_ergonomics() {
        let h = EZGraph::<Uuid>::new();
        println!("bytes? {:?}", b"testing");
        println!("{:#?}", h);
    }
}
