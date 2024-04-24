use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use indexmap::IndexMap;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{structs::HGraphCore, EdgeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    String,
}

impl FromStr for DataType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "bool" => Ok(DataType::Bool),
            "uint8" => Ok(DataType::UInt8),
            "uint16" => Ok(DataType::UInt16),
            "uint32" => Ok(DataType::UInt32),
            "uint64" => Ok(DataType::UInt64),
            "int8" => Ok(DataType::Int8),
            "int16" => Ok(DataType::Int16),
            "int32" => Ok(DataType::Int32),
            "int64" => Ok(DataType::Int64),
            "float32" => Ok(DataType::Float32),
            "float64" => Ok(DataType::Float64),
            "string" => Ok(DataType::String),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
}

impl Value {
    pub fn dtype(&self) -> String {
        match self {
            Value::Bool(_) => "Bool".to_string(),
            Value::UInt8(_) => "UInt8".to_string(),
            Value::UInt16(_) => "UInt16".to_string(),
            Value::UInt32(_) => "UInt32".to_string(),
            Value::UInt64(_) => "UInt64".to_string(),
            Value::Int8(_) => "Int8".to_string(),
            Value::Int16(_) => "Int16".to_string(),
            Value::Int32(_) => "Int32".to_string(),
            Value::Int64(_) => "Int64".to_string(),
            Value::Float32(_) => "Float32".to_string(),
            Value::Float64(_) => "Float64".to_string(),
            Value::String(_) => "String".to_string(),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}
impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::UInt8(value)
    }
}
impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::UInt16(value)
    }
}
impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::UInt32(value)
    }
}
impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::UInt64(value)
    }
}
impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::Int8(value)
    }
}
impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Int16(value)
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int32(value)
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int64(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float32(value)
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float64(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl<'a> From<Value> for AnyValue<'a> {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(v) => AnyValue::Boolean(v),
            Value::UInt8(v) => v.into(),
            Value::UInt16(v) => v.into(),
            Value::UInt32(v) => v.into(),
            Value::UInt64(v) => v.into(),
            Value::Int8(v) => v.into(),
            Value::Int16(v) => v.into(),
            Value::Int32(v) => v.into(),
            Value::Int64(v) => v.into(),
            Value::Float32(v) => v.into(),
            Value::Float64(v) => v.into(),
            Value::String(v) => AnyValue::StringOwned(v.into()),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        if let Value::Bool(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for u8 {
    fn from(value: Value) -> Self {
        if let Value::UInt8(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for u16 {
    fn from(value: Value) -> Self {
        if let Value::UInt16(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for u32 {
    fn from(value: Value) -> Self {
        if let Value::UInt32(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for u64 {
    fn from(value: Value) -> Self {
        if let Value::UInt64(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for i8 {
    fn from(value: Value) -> Self {
        if let Value::Int8(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for i16 {
    fn from(value: Value) -> Self {
        if let Value::Int16(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for i32 {
    fn from(value: Value) -> Self {
        if let Value::Int32(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for i64 {
    fn from(value: Value) -> Self {
        if let Value::Int64(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for f32 {
    fn from(value: Value) -> Self {
        if let Value::Float32(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}
impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        if let Value::Float64(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        if let Value::String(v) = value {
            return v;
        } else {
            panic!("Improper From conversion Value => bool.")
        }
    }
}

pub struct KVGraph {
    core: HGraphCore<HashMap<String, Value>, HashMap<String, Value>, u128, u128>,
    schema: IndexMap<String, String>,
}

impl KVGraph {
    pub fn new() -> Self {
        Self {
            core: HGraphCore::new(),
            schema: IndexMap::from([
                ("label".to_string(), "String".to_string()),
                ("id".to_string(), "String".to_string()),
                ("nodes".to_string(), "String".to_string()),
                ("labelled_nodes".to_string(), "String".to_string()),
            ]),
        }
    }
    pub fn add_node(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.core
            .add_node_with_id(HashMap::new(), id.clone().as_u128());
        self.insert(&id, "label", "".to_string()).unwrap();
        id
    }

    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<Uuid> {
        (0..num_nodes).map(|_| self.add_node()).collect()
    }

    pub fn add_node_with_label<S>(&mut self, label: S) -> Uuid
    where
        S: ToString,
    {
        let id = Uuid::new_v4();
        self.core
            .add_node_with_id(HashMap::new(), id.clone().as_u128());
        self.insert(&id, "label", label.to_string()).unwrap();
        id
    }

    /// Removes a node from the node set. The deleted node will be added to a
    /// dequeue to be reused later once all possible nodes have been created.
    pub fn remove_node(&mut self, node: Uuid) {
        self.core.remove_node(node.as_u128());
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: Vec<Uuid>) {
        let _: Vec<_> = nodes.into_iter().map(|id| self.remove_node(id)).collect();
    }
    /// Creates an undirected edge among the given nodes. Duplicate input nodes are removed.
    /// Returns `None` if an edge among those nodes already exists (Duplicate edges not allowed) or
    /// if less than 2 nodes are provided.
    pub fn add_edge_with_label<E>(&mut self, nodes: E, label: &str) -> Option<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes.as_ref().iter().map(|id| id.as_u128()).collect();
        let edge: EdgeSet<u128> = EdgeSet::from(node_vec);
        if edge.len() == 1 {
            return None;
        }
        let id = Uuid::new_v4();
        self.core
            .add_edge_with_id(edge, HashMap::new(), id.clone().as_u128());
        self.insert(&id, "label", label.to_string()).unwrap();
        Some(id)
    }

    /// Creates an undirected edge among the given nodes. Duplicate input nodes are removed.
    /// Returns `None` if an edge among those nodes already exists (Duplicate edges not allowed) or
    /// if less than 2 nodes are provided.
    pub fn add_edge<E>(&mut self, nodes: E) -> Option<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        self.add_edge_with_label(nodes, "")
    }
    pub fn remove_edge<E>(&mut self, nodes: E)
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes.as_ref().iter().map(|id| id.as_u128()).collect();
        let e = self.core.find_id(node_vec);
        if let Some(id) = e {
            self.core.remove_edge(id);
        }
    }

    pub fn remove_edge_id(&mut self, edge_id: Uuid) {
        self.core.remove_edge(edge_id.as_u128());
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn get_nodes_of_edge_id(&self, edge_id: &Uuid) -> Option<Vec<Uuid>> {
        self.core.edges.get(&edge_id.as_u128()).map(|e| {
            e.nodes
                .node_vec()
                .into_iter()
                .map(|id| Uuid::from_u128(id))
                .collect()
        })
    }

    pub fn find_id<E>(&self, nodes: E) -> Option<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes.as_ref().iter().map(|id| id.as_u128()).collect();
        self.core.find_id(node_vec).map(|id| Uuid::from_u128(id))
    }

    /// Computes the link of the provided set. The link of a single
    /// hyperedge is computed using the complement, so a hyperedge
    /// of nodes {a, b, c, d} and a provided `face` of {a, b} would
    /// yield a link of {c, d}. The link of the graph is then the
    /// union of all the links of each hyperedge.
    pub fn link<E>(&self, nodes: E) -> Vec<Vec<Uuid>>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes.as_ref().iter().map(|id| id.as_u128()).collect();
        self.core
            .link(node_vec)
            .into_iter()
            .map(|(_, edge)| {
                let nodes = edge.to_node_vec();
                nodes.into_iter().map(|id| Uuid::from_u128(id)).collect()
            })
            .collect()
    }

    /// Returns the set of edge of size less than or equal to `k`,
    /// inclusive. Also note that `k` refers to the cardinality of the
    /// provided sets, not the dimension.
    pub fn k_skeleton(&self, k: usize) -> HashSet<Uuid> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= k)
            .map(|(id, _)| Uuid::from_u128(*id))
            .collect()
    }

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    pub fn edges_of_size(&self, card: usize) -> Vec<Uuid> {
        self.core
            .edges
            .iter()
            .filter(|(id, e)| e.nodes.len() == card)
            .map(|(id, e)| Uuid::from_u128(*id))
            .collect()
    }
    pub fn get_containing_edges<E>(&self, nodes: E) -> Vec<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .get_containing_edges_strict(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }
    pub fn get_maximal_containing_edges<E>(&self, nodes: E) -> Vec<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .maximal_containing_edges(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    /// Adds a `key`-`value` pair to the provided `id`, whether `id` correspond to
    /// a node or edge. The provided pair must match the schema associated with the
    /// hypergraph, if the `key` has not been seen before it automatically creates
    /// a new schema in the structure.
    pub fn insert<S, V>(&mut self, id: &Uuid, key: S, value: V) -> Result<(), ()>
    where
        S: ToString,
        V: Into<Value>,
    {
        let id = id.as_u128();
        let key_string = key.to_string();
        let val: Value = value.into();
        let unchangeables = vec![
            "id".to_string(),
            "nodes".to_string(),
            "labelled_nodes".to_string(),
        ];
        if unchangeables.contains(&key_string) {
            return Err(());
        }
        if self.schema.contains_key(&key_string) == false {
            self.schema.insert(key_string.clone(), val.dtype());
        } else {
            if *self.schema.get(&key_string).unwrap() != val.dtype() {
                return Err(());
            }
        }
        if self.core.nodes.contains_key(&id) {
            self.core
                .borrow_node_mut(&id)
                .unwrap()
                .insert(key_string, val);
            Ok(())
        } else if self.core.edges.contains_key(&id) {
            self.core
                .borrow_edge_mut(&id)
                .unwrap()
                .insert(key_string, val);
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn get(&self, id: &Uuid, key: &str) -> Option<&Value> {
        let id = id.as_u128();
        if self.core.nodes.contains_key(&id) {
            let query = key.to_string();
            self.core.borrow_node(&id).unwrap().get(&query)
        } else if self.core.nodes.contains_key(&id) {
            let query = key.to_string();
            self.core.borrow_edge(&id).unwrap().get(&query)
        } else {
            None
        }
    }

    /// A shorthand for `self.insert(id, "label", label)`.
    pub fn label<S>(&mut self, id: &Uuid, label: S) -> Result<(), ()>
    where
        S: ToString,
    {
        self.insert(id, "label", label.to_string())
    }

    /// Returns a copy of the given schema being used
    pub fn get_schema(&self) -> Vec<(String, String)> {
        self.schema.clone().into_iter().collect()
    }

    /// Collects all key-value pairs of the given `KVGraph` schema for all nodes
    /// with `null`'s where given nodes do not contain the key. Rows are given
    /// in arbitrary ordering.
    pub fn get_node_dataframe(&self) -> DataFrame {
        let mut df = DataFrame::default();
        for (node_id, node_struct) in self.core.nodes.iter() {
            let node_kv_store = &node_struct.data;
            let mut node_df = DataFrame::default();
            let id_string = Uuid::from_u128(*node_id).to_string();
            // todo: change this to a List type of AnyValue.
            for (key, dtype) in self.schema.iter() {
                if &key[..] == "id" {
                    node_df
                        .with_column(Series::new("id", [id_string.clone()]))
                        .expect("couldn't add column.");
                } else if &key[..] == "nodes" {
                    let mut node_string = String::from("[");
                    node_string.push_str(&id_string[..]);
                    node_string.push(']');
                    node_df
                        .with_column(Series::new("nodes", [node_string]))
                        .expect("What error");
                } else if &key[..] == "labelled_nodes" {
                    let mut node_string = String::from("[");

                    let label = node_kv_store
                        .get(&"label".to_string())
                        .map(|val| Into::<String>::into(val.clone()))
                        .expect("Could not get label as string.");
                    node_string.push_str(&label[..]);
                    node_string.push(']');
                    node_df
                        .with_column(Series::new("labelled_nodes", [node_string]))
                        .expect("What error");
                } else {
                    let true_dtype =
                        DataType::from_str(&dtype[..]).expect("could not parse dtype.");
                    match true_dtype {
                        DataType::Bool => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<bool>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt8 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<u8>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt16 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<u16>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt32 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<u32>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt64 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<u64>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int8 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<i8>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int16 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<i16>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int32 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<i32>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int64 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<i64>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Float32 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<f32>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Float64 => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<f64>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::String => {
                            let s = Series::new(
                                &key[..],
                                [node_kv_store
                                    .get(key)
                                    .map(|val| Into::<String>::into(val.clone()))],
                            );
                            node_df.with_column(s).expect("Couldn't add column.");
                        }
                    };
                }
            }
            df.vstack_mut(&node_df).expect("Could not vstack");
        }
        df
    }
    pub fn get_edge_dataframe(&self) -> DataFrame {
        let mut df = DataFrame::default();
        for (edge_id, edge_struct) in self.core.edges.iter() {
            let edge_kv_store = &edge_struct.data;
            let mut edge_df = DataFrame::default();
            let id_string = Uuid::from_u128(*edge_id).to_string();
            // todo: change this to a List type of AnyValue.
            for (key, dtype) in self.schema.iter() {
                if &key[..] == "id" {
                    edge_df
                        .with_column(Series::new("id", [id_string.clone()]))
                        .expect("couldn't add column.");
                } else if &key[..] == "nodes" {
                    let mut node_labels_string = String::from("[");
                    let node_labels: Vec<_> = edge_struct
                        .nodes
                        .node_vec()
                        .into_iter()
                        .map(|id| Uuid::from_u128(id).to_string())
                        .collect();
                    for label in node_labels {
                        node_labels_string.push_str(&label[..]);
                        node_labels_string.push_str(", ");
                    }
                    node_labels_string.pop();
                    node_labels_string.pop();
                    node_labels_string.push(']');
                    edge_df
                        .with_column(Series::new("nodes", [node_labels_string]))
                        .expect("What error");
                } else if &key[..] == "labelled_nodes" {
                    let mut node_labels_string = String::from("[");
                    let node_labels: Vec<_> = edge_struct
                        .nodes
                        .node_vec()
                        .into_iter()
                        .map(|id| {
                            let node_kv_store = self
                                .core
                                .nodes
                                .get(&id)
                                .expect("Could not find data for node in given edge.");
                            let node_label = node_kv_store
                                .data
                                .get("label")
                                .expect("Could not find node label.");
                            Into::<String>::into(node_label.clone())
                        })
                        .collect();
                    for label in node_labels {
                        node_labels_string.push_str(&label[..]);
                        node_labels_string.push_str(", ");
                    }
                    node_labels_string.pop();
                    node_labels_string.pop();
                    node_labels_string.push(']');
                    edge_df
                        .with_column(Series::new("labelled_nodes", [node_labels_string]))
                        .expect("What error");
                } else {
                    let true_dtype =
                        DataType::from_str(&dtype[..]).expect("could not parse dtype.");
                    match true_dtype {
                        DataType::Bool => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<bool>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt8 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<u8>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt16 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<u16>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt32 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<u32>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::UInt64 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<u64>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int8 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<i8>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int16 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<i16>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int32 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<i32>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Int64 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<i64>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Float32 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<f32>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::Float64 => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<f64>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                        DataType::String => {
                            let s = Series::new(
                                &key[..],
                                [edge_kv_store
                                    .get(key)
                                    .map(|val| Into::<String>::into(val.clone()))],
                            );
                            edge_df.with_column(s).expect("Couldn't add column.");
                        }
                    };
                }
            }
            df.vstack_mut(&edge_df).expect("Could not vstack");
        }
        df
    }

    /// Computes the dataframe of both nodes and edges, starting with nodes at
    /// the top followed by the edges. Just a vertical stack of
    /// `get_node_dataframe` and `get_edge_dataframe`.
    pub fn get_dataframe(&self) -> DataFrame {
        let node_df = self.get_node_dataframe();
        let edge_df = self.get_edge_dataframe();
        node_df
            .vstack(&edge_df)
            .expect("Cannot stack node and edge dataframes")
    }

    /// Clones the given id's key-value pairs.
    /// The result is wrapped in an option to help the user distinguish between an empty `id`
    /// with no key-value pairs or the id is incorrect.
    pub fn get_all_kv_pairs(&self, id: &Uuid) -> Option<Vec<(String, Value)>> {
        let id = id.as_u128();
        if self.core.nodes.contains_key(&id) {
            Some(
                self.core
                    .borrow_node(&id)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect(),
            )
        } else if self.core.edges.contains_key(&id) {
            Some(
                self.core
                    .borrow_edge(&id)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect(),
            )
        } else {
            None
        }
    }
}

mod tests {
    use polars::datatypes::AnyValue;

    use crate::KVGraph;

    #[test]
    fn create_read_update_delete() {
        let mut hg = KVGraph::new();
        let n1 = hg.add_node();
        let n2 = hg.add_node();
        let n3 = hg.add_node_with_label("node 3");
        hg.label(&n1, "node 1");
        hg.label(&n2, "node 2");
        hg.insert(&n1, "test", "failure".to_string()).unwrap();
        hg.insert(&n1, "weight", 1.0_f32).unwrap();
        hg.insert(&n1, "booty", true).unwrap();
        hg.insert(&n2, "weight", 2.2_f32);
        hg.insert(&n2, "booty", false);
        hg.insert(&n2, "defense", 0_u8);
        let nodes = vec![n1, n2, n3];
        let e1 = hg.add_edge(&[n1, n2]).unwrap();
        let e2 = hg.add_edge(&[nodes[0], nodes[2]]).unwrap();
        hg.insert(&e1, "defense", 3_u8);

        dbg!(hg.get(&n1, "test"));
        println!("{:}", hg.get_node_dataframe());
        println!("{:}", hg.get_edge_dataframe());
        println!("{:}", hg.get_dataframe());
    }
}
