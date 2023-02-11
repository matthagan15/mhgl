use std::collections::HashMap;

use uuid::Uuid;

use crate::structs::{EdgeID, EdgeWeight, NodeUUID, SparseGraph};

use super::SparseVector;
use crate::traits::*;

#[derive(Debug, Clone)]
struct EZGraph<N: HgNode> {
    pub name: String,
    graph: SparseGraph<N>,
    id_to_label: HashMap<Uuid, String>,
    label_to_id: HashMap<String, Uuid>,
}

impl<N: HgNode> EZGraph<N> {
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
