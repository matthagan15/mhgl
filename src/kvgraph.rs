use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::HGraphCore;

#[derive(Debug)]
struct KVStore {
    store: HashMap<String, Box<dyn std::fmt::Debug>>,
}

/// A hypergraph structure that can store key-value pairs
/// for each node and edge. Utilizes Uuid's for nodes and edges.
#[derive(Debug)]
pub struct KVGraph {
    hgraph: HGraphCore<Uuid, KVStore, KVStore>,
    kv_store: HashMap<Uuid, HashMap<String, String>>,
}
