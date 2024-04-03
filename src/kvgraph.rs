use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::HGraphCore;



/// A hypergraph structure that can store key-value pairs
/// for each node and edge. Utilizes Uuid's for nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVGraph {
    hgraph: HGraphCore<Uuid>,
    kv_store: HashMap<Uuid, HashMap<String, String>>,
}

