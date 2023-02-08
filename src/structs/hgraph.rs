use std::collections::HashMap;

use uuid::Uuid;

use super::{NodeUUID, SparseGraph};

struct HGraph {
    pub name: String,
    backend: SparseGraph<NodeUUID>,
    id_to_label: HashMap<Uuid, String>,
    label_to_id: HashMap<String, Uuid>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            backend: SparseGraph::new(),
            name: String::new(),
            id_to_label: HashMap::new(),
            label_to_id: HashMap::new(),
        }
    }
    
}
