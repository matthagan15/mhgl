use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::structs::{EdgeID, EdgeWeight, NodeID, SparseGraph, GeneroGraph, SparseBasis, GeneroEdge};

use crate::traits::*;

#[derive(Debug, Clone)]
struct HGraph {
    // TODO: Move storage of nodes from underlying graph structure to container structures.
    pub name: String,
    nodes: HashSet<NodeID>,
    graph: GeneroGraph<SparseBasis<NodeID>>,
    id_to_label: HashMap<Uuid, String>,
    label_to_id: HashMap<String, Uuid>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            name: String::new(),
            nodes: HashSet::new(),
            graph: GeneroGraph::new(),
            id_to_label: HashMap::new(),
            label_to_id: HashMap::new(),
        }
    }

    pub fn create_node<S: ToString>(&mut self, label: S) -> NodeID {
        let id = Uuid::new_v4();
        self.nodes.insert(id.clone());
        self.id_to_label.insert(id.clone(), label.to_string());
        self.label_to_id.insert(label.to_string(), id.clone());
        id
    }

    pub fn remove_node<S: ToString>(&mut self, label: S) {
        if let Some(id) = self.label_to_id.get(&label.to_string()) {
            let node_basis = SparseBasis::from(HashSet::from([id.clone()]));
            // TODO: WRONG! this currently gets edges that exactly match this basis
            // Need a function that gets containing edges, inbound and outbound.
            let edges = self.graph.get_containing_edges(&node_basis);
            for edge in edges {
                if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                    old_edge.remove_node(&node_basis);
                    self.graph.add_edge(old_edge);
                }
            }
            self.id_to_label.remove(id);
            self.nodes.remove(id);
        }
        self.label_to_id.remove(&label.to_string());
    }

    pub fn create_edge<const N: usize, const M: usize>(&mut self, inputs: [String; N], outputs: [String; M], weight:EdgeWeight) {
        let mut e = GeneroEdge::new();
        for input in inputs {
            if let Some(id) = self.label_to_id.get(&input) {
                let input_basis = SparseBasis::from(HashSet::from([id.clone()]));
                e.add_input_nodes(&input_basis);
            }
        }
        for output in outputs {
            if let Some(id) = self.label_to_id.get(&output) {
                let output_basis = SparseBasis::from(HashSet::from([id.clone()]));
                e.add_output_nodes(&output_basis);
            }
        }
        e.change_direction(crate::structs::EdgeDirection::Directed);
        e.change_weight(weight);
        self.graph.add_edge(e);
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
        hg.create_node("node1");
        hg.create_node(1);
        hg.create_node(1.1);
        hg.create_edge([1.to_string(), 1.1.to_string()], ["node1".to_string()], 1.);
        println!("before removal:\n{:#?}", hg);
        hg.remove_node(1);
        println!("post removal:\n{:#?}", hg);
    }
}
