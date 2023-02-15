use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::structs::{EdgeID, EdgeWeight, NodeID, SparseGraph};

use crate::traits::*;

#[derive(Debug, Clone)]
struct HGraph {
    // TODO: Move storage of nodes from underlying graph structure to container structures.
    pub name: String,
    nodes: HashSet<NodeID>,
    graph: SparseGraph<NodeID>,
    id_to_label: HashMap<Uuid, String>,
    label_to_id: HashMap<String, Uuid>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            name: String::new(),
            nodes: HashSet::new(),
            graph: SparseGraph::new(),
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

mod test {
    use uuid::Uuid;

    use super::HGraph;

    #[test]
    fn test_hgraph_trait_ergonomics() {
        let h = HGraph::new();
        println!("bytes? {:?}", b"testing");
        println!("{:#?}", h);
    }
}
