#[cfg(feature = "polars")]
use polars::prelude::*;

use std::{collections::HashMap, path::PathBuf, str::FromStr};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{EdgeSet, HGraph, HyperGraph};

/// The data types of the possible values that can be stored in a `KVGraph`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValueTypes {
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

impl FromStr for ValueTypes {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "bool" => Ok(ValueTypes::Bool),
            "uint8" => Ok(ValueTypes::UInt8),
            "uint16" => Ok(ValueTypes::UInt16),
            "uint32" => Ok(ValueTypes::UInt32),
            "uint64" => Ok(ValueTypes::UInt64),
            "int8" => Ok(ValueTypes::Int8),
            "int16" => Ok(ValueTypes::Int16),
            "int32" => Ok(ValueTypes::Int32),
            "int64" => Ok(ValueTypes::Int64),
            "float32" => Ok(ValueTypes::Float32),
            "float64" => Ok(ValueTypes::Float64),
            "string" => Ok(ValueTypes::String),
            _ => Err(()),
        }
    }
}

/// The possible values that can be stored in a `KVGraph`, essentially a subset
/// of the polars `AnyValue<'a>` so that `KVGraph` can avoid generic lifetimes.
/// Helper conversions are implemented.
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
    pub fn dtype(&self) -> ValueTypes {
        match self {
            Value::Bool(_) => ValueTypes::Bool,
            Value::UInt8(_) => ValueTypes::UInt8,
            Value::UInt16(_) => ValueTypes::UInt16,
            Value::UInt32(_) => ValueTypes::UInt32,
            Value::UInt64(_) => ValueTypes::UInt64,
            Value::Int8(_) => ValueTypes::Int8,
            Value::Int16(_) => ValueTypes::Int16,
            Value::Int32(_) => ValueTypes::Int32,
            Value::Int64(_) => ValueTypes::Int64,
            Value::Float32(_) => ValueTypes::Float32,
            Value::Float64(_) => ValueTypes::Float64,
            Value::String(_) => ValueTypes::String,
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

#[cfg(feature = "polars")]
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

/// A hypergraph where you can store key-value pairs in the form of
/// `String`s for keys and a [`Value`] enum, a simple subset of the polars `AnyValue<'a>`. `Uuid`s are used for `NodeID` and `EdgeID` so that way
/// nodes and edges can be present in an output dataframe without confusion.
/// A schema is used to maintain consistency of data types across nodes and
/// edges for later collection into dataframes. See [`ValueTypes`] for which types can be stored. Note that the schema does not have to be set by the
/// user and is determined by the values passed in on `insert`. If a `key`
/// has never been seen before then the datatype is inferred from the `value` and the key - type pairing is added to the schema. Currently `KVGraph` does not keep track of when a specific key is completely removed from the `KVGraph`, so once a `key` is set to a specific type that can only be reset after calling [`KVGraph::remove_all_keys`]. The current schema can be retrieved as a `HashMap<String, ValueType>` with [`get_schema`](KVGraph::get_schema).
///
/// There are a few special keys, `id`, `nodes` and `labelled_nodes` cannot be
/// modified. `label` is used as an easy name for the user to visualize a
/// node or edge and can be modified with `graph.label(id, label)` as a shorthand for `graph.insert(id, "label", label)`, or set upon
/// node or edge creation. Currently it does not allow for the user to provide a `Uuid`
/// upon creation of a node or edge and new uuids are generated for each new
/// node or edge.
#[derive(Debug, Clone)]
pub struct KVGraph {
    pub(crate) core: HGraph<HashMap<String, Value>, HashMap<String, Value>, Uuid, Uuid>,
    schema: IndexMap<String, ValueTypes>,
}

impl Default for KVGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KVGraph {
    pub fn new() -> Self {
        Self {
            core: HGraph::new(),
            schema: IndexMap::from([
                ("label".to_string(), ValueTypes::String),
                ("id".to_string(), ValueTypes::String),
                ("nodes".to_string(), ValueTypes::String),
                ("labelled_nodes".to_string(), ValueTypes::String),
            ]),
        }
    }

    pub fn add_node(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.core.add_node_with_id(HashMap::new(), id.clone());
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
        self.core.add_node_with_id(HashMap::new(), id.clone());
        self.insert(&id, "label", label.to_string()).unwrap();
        id
    }

    /// Removes a node from the node set. The deleted node will be added to a
    /// dequeue to be reused later once all possible nodes have been created.
    /// The data stored will be dropped.
    pub fn remove_node(&mut self, node: Uuid) -> Option<HashMap<String, Value>> {
        self.core.remove_node(node)
    }

    /// Creates an undirected edge among the given nodes. Duplicate input nodes are removed.
    /// Returns `None` if an edge among those nodes already exists (Duplicate edges not allowed) or
    /// if less than 2 nodes are provided.
    pub fn add_edge_with_label(&mut self, nodes: impl AsRef<[Uuid]>, label: &str) -> Uuid {
        let edge: EdgeSet<Uuid> = EdgeSet::from(nodes.as_ref());
        if edge.len() == 1 {
            panic!("Cannot make an edge with only one node.")
        }
        let id = Uuid::new_v4();
        self.core.add_edge_with_id(edge, HashMap::new(), id.clone());
        self.insert(&id, "label", label.to_string()).unwrap();
        id
    }

    /// Creates an undirected edge among the given nodes. Duplicate input nodes are removed.
    /// Returns `None` if an edge among those nodes already exists (Duplicate edges not allowed) or
    /// if less than 2 nodes are provided.
    pub fn add_edge(&mut self, nodes: impl AsRef<[Uuid]>) -> Uuid {
        self.add_edge_with_label(nodes, "")
    }

    pub fn remove_edge(&mut self, edge_id: Uuid) -> Option<HashMap<String, Value>> {
        self.core.remove_edge(edge_id)
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn get_nodes_of_edge_id(&self, edge_id: &Uuid) -> Option<Vec<Uuid>> {
        self.core.edges.get(&edge_id).map(|e| e.nodes.node_vec())
    }

    pub fn find_id<N>(&self, nodes: impl AsRef<[Uuid]>) -> Option<Uuid> {
        self.core.find_id(nodes.as_ref())
    }

    /// Adds a `key`-`value` pair to the provided `id`, whether `id` correspond to
    /// a node or edge. The provided pair must match the schema associated with the
    /// hypergraph, if the `key` has not been seen before it automatically creates
    /// a new schema in the structure.
    pub fn insert(
        &mut self,
        id: &Uuid,
        key: impl ToString,
        value: impl Into<Value>,
    ) -> Result<Option<Value>, String> {
        let key_string = key.to_string();
        let val: Value = value.into();
        let unchangeables = vec![
            "id".to_string(),
            "nodes".to_string(),
            "labelled_nodes".to_string(),
        ];
        if unchangeables.contains(&key_string) {
            return Err(String::from("Cannot change this key."));
        }
        if self.schema.contains_key(&key_string) == false {
            self.schema.insert(key_string.clone(), val.dtype());
        } else {
            if *self.schema.get(&key_string).unwrap() != val.dtype() {
                return Err(String::from(
                    "Data type of Value provided does not match the schema.",
                ));
            }
        }
        if self.core.nodes.contains_key(&id) {
            Ok(self.core.get_node_mut(&id).unwrap().insert(key_string, val))
        } else if self.core.edges.contains_key(&id) {
            Ok(self.core.get_edge_mut(&id).unwrap().insert(key_string, val))
        } else {
            Err(String::from("KVGraph does not contain this ID."))
        }
    }

    /// Retrieve the value stored for the given `id` and `key`.
    pub fn get(&self, id: &Uuid, key: &str) -> Option<&Value> {
        if self.core.nodes.contains_key(&id) {
            let query = key.to_string();
            self.core.get_node(&id).unwrap().get(&query)
        } else if self.core.nodes.contains_key(&id) {
            let query = key.to_string();
            self.core.get_edge(&id).unwrap().get(&query)
        } else {
            None
        }
    }

    /// A shorthand for `self.insert(id, "label", label)`.
    pub fn label(&mut self, id: &Uuid, label: impl ToString) -> Result<Option<Value>, String> {
        self.insert(id, "label", label.to_string())
    }

    /// Returns a copy of the given schema being used
    pub fn get_schema(&self) -> HashMap<String, ValueTypes> {
        self.schema.clone().into_iter().collect()
    }

    /// Removes the input key from the schema for future change to a different
    /// data type. Has to traverse the entire graph so could take a while.
    pub fn remove_all_keys(&mut self, key: &str) -> Vec<(Uuid, Value)> {
        if self.schema.contains_key(key) == false {
            return Vec::new();
        }
        let key_string = key.to_string();
        let mut ret = Vec::new();
        for (node_id, node_data) in self.core.nodes.iter_mut() {
            let node_kv_store = &mut node_data.data;
            if node_kv_store.contains_key(&key_string) {
                ret.push((node_id.clone(), node_kv_store.remove(&key_string).unwrap()));
            }
        }
        for (edge_id, edge_data) in self.core.edges.iter_mut() {
            let edge_kv_store = &mut edge_data.data;
            if edge_kv_store.contains_key(&key_string) {
                ret.push((edge_id.clone(), edge_kv_store.remove(&key_string).unwrap()));
            }
        }
        self.schema.swap_remove(&key_string);
        ret
    }

    fn nodes_string(&self, id: &Uuid) -> Option<String> {
        let mut s = String::from("[");
        if self.core.nodes.contains_key(&id) {
            s.push_str(&id.to_string()[..]);
            s.push(']');
            Some(s)
        } else if self.core.edges.contains_key(&id) {
            let edge_set = &self.core.edges.get(&id).unwrap().nodes;
            for _ix in 0..(edge_set.len() - 1) {
                s.push_str(&id.to_string()[..]);
                s.push(',');
            }
            s.push_str(&edge_set.0[edge_set.len() - 1].to_string()[..]);
            s.push(']');
            Some(s)
        } else {
            None
        }
    }

    fn labelled_nodes_string(&self, id: &Uuid) -> Option<String> {
        let mut s = String::from("[");
        if self.core.nodes.contains_key(&id) {
            let kv_store = &self.core.nodes.get(&id).unwrap().data;
            let label = kv_store
                .get(&"label".to_string())
                .map(|val| Into::<String>::into(val.clone()))
                .expect("Could not get label as string.");
            s.push_str(&label[..]);
            s.push(']');
            Some(s)
        } else if self.core.edges.contains_key(&id) {
            let edge_set = &self.core.edges.get(&id).unwrap().nodes;
            for ix in 0..(edge_set.len() - 1) {
                let kv_store = &self
                    .core
                    .nodes
                    .get(&edge_set.0[ix])
                    .expect("Broken edge encountered when trying to get labels of neighbors.")
                    .data;
                let label = kv_store
                    .get(&"label".to_string())
                    .map(|val| Into::<String>::into(val.clone()))
                    .expect("Could not get label as string.");
                s.push_str(&label[..]);
                s.push(',');
            }
            let last_node_id = edge_set.0[edge_set.len() - 1];
            let kv_store = &self
                .core
                .nodes
                .get(&last_node_id)
                .expect("Broken edge encountered when trying to get labels of neighbors.")
                .data;
            let label = kv_store
                .get(&"label".to_string())
                .map(|val| Into::<String>::into(val.clone()))
                .expect("Could not get label as string.");
            s.push_str(&label[..]);
            s.push(']');
            Some(s)
        } else {
            None
        }
    }

    #[cfg(feature = "polars")]
    pub fn dataframe_of_ids(&self, ids: impl AsRef<[Uuid]>) -> DataFrame {
        let mut df = DataFrame::default();
        ids.as_ref()
            .into_iter()
            .map(|id| (id, id.to_string()))
            .filter(|(id, _)| self.core.nodes.contains_key(id) || self.core.edges.contains_key(id))
            .for_each(|(id, id_string)| {
                let mut id_df = DataFrame::default();
                let kv_store = if self.core.nodes.contains_key(&id) {
                    &self.core.nodes.get(&id).unwrap().data
                } else {
                    &self.core.edges.get(&id).unwrap().data
                };

                for (key, dtype) in self.schema.iter() {
                    if &key[..] == "id" {
                        id_df
                            .with_column(Series::new("id", [id_string.clone()]))
                            .expect("couldn't add column.");
                    } else if &key[..] == "nodes" {
                        let node_string = self
                            .nodes_string(&id)
                            .expect("ID was checked in previous filter.");
                        id_df
                            .with_column(Series::new("nodes", [node_string]))
                            .expect("What error");
                    } else if &key[..] == "labelled_nodes" {
                        let labelled_nodes = self
                            .labelled_nodes_string(&id)
                            .expect("ID was checked in previous filter.");
                        id_df
                            .with_column(Series::new("labelled_nodes", [labelled_nodes]))
                            .expect("What error");
                    } else {
                        match dtype {
                            ValueTypes::Bool => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<bool>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::UInt8 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u8>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::UInt16 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u16>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::UInt32 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u32>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::UInt64 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u64>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::Int8 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i8>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::Int16 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i16>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::Int32 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i32>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::Int64 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i64>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::Float32 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<f32>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::Float64 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<f64>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            ValueTypes::String => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store
                                        .get(key)
                                        .map(|val| Into::<String>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                        };
                    }
                }
                df.vstack_mut(&id_df).expect("Could not vstack");
            });
        df
    }
    /// Collects the dataframe for all nodes in the hypergraph. If a key is not
    /// present for a node then 'null' is used in the dataframe.
    #[cfg(feature = "polars")]
    pub fn dataframe_of_nodes(&self) -> DataFrame {
        self.dataframe_of_ids(self.core.nodes.keys().cloned().collect::<Vec<_>>())
    }

    /// Collects the dataframe for all edges in the hypergraph. If a key is not
    /// present for an edge then 'null' is used in the dataframe.
    #[cfg(feature = "polars")]
    pub fn dataframe_of_edges(&self) -> DataFrame {
        self.dataframe_of_ids(self.core.edges.keys().cloned().collect::<Vec<_>>())
    }

    /// Computes the dataframe of both nodes and edges, starting with nodes at
    /// the top followed by the edges. Just a vertical stack of
    /// `get_node_dataframe` and `get_edge_dataframe`.
    #[cfg(feature = "polars")]
    pub fn dataframe(&self) -> DataFrame {
        let node_df = self.dataframe_of_nodes();
        let edge_df = self.dataframe_of_edges();
        node_df
            .vstack(&edge_df)
            .expect("Cannot stack node and edge dataframes")
    }

    #[cfg(feature = "polars")]
    pub fn to_disk(&self, filename: PathBuf) -> Result<(), std::io::Error> {
        use std::io::Write;
        let mut df = self.dataframe();
        let csv_filename = filename.with_extension("csv");
        let mut csv_file = std::fs::File::create(csv_filename.as_path())?;
        CsvWriter::new(&mut csv_file)
            .finish(&mut df)
            .expect("Could not serialize dataframe");
        let hg_filename = filename.with_extension("hg");
        let mut hg_file = std::fs::File::create(hg_filename.as_path())?;
        write!(hg_file, "nodes\n")?;
        for node in self.core.nodes.keys() {
            write!(hg_file, "{:},", node.to_string())?;
        }
        write!(hg_file, "\nedges\n")?;
        for (edge_id, edge) in self.core.edges.iter() {
            let mut nodes_string = String::new();
            nodes_string.push_str("[");
            for node in edge.nodes.node_vec() {
                nodes_string.push_str(&node.to_string()[..]);
                nodes_string.push(',');
            }
            nodes_string.pop();
            nodes_string.push(']');
            write!(hg_file, "{:}={:}", edge_id.to_string(), nodes_string)?;
        }
        Ok(())
    }
}

impl HyperGraph for KVGraph {
    type NodeID = Uuid;

    type EdgeID = Uuid;

    fn query_edge(&self, edge: &Self::EdgeID) -> Option<Vec<Self::NodeID>> {
        self.core.query_edge(edge)
    }

    fn containing_edges_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
        self.core.containing_edges_of_nodes(nodes.as_ref())
    }

    fn containing_edges(&self, edge: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core.containing_edges(edge)
    }

    fn link(&self, edge: &Self::EdgeID) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)> {
        self.core.link(edge)
    }

    fn link_of_nodes(
        &self,
        nodes: impl AsRef<[Self::NodeID]>,
    ) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)> {
        self.core.link_of_nodes(nodes)
    }

    fn maximal_edges(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core.maximal_edges(edge_id)
    }

    fn maximal_edges_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
        self.core.maximal_edges_of_nodes(nodes)
    }

    fn edges_of_size(&self, card: usize) -> Vec<Self::EdgeID> {
        self.core.edges_of_size(card)
    }

    fn skeleton(&self, cardinality: usize) -> Vec<Self::EdgeID> {
        self.core.skeleton(cardinality)
    }

    fn boundary_up(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core.boundary_up(edge_id)
    }

    fn boundary_down(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core.boundary_down(edge_id)
    }

    fn boundary_up_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
        self.core.boundary_up_of_nodes(nodes)
    }

    fn boundary_down_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
        self.core.boundary_down_of_nodes(nodes)
    }
}

#[cfg(test)]
mod tests {

    use crate::KVGraph;

    #[test]
    #[cfg(feature = "polars")]
    fn create_read_update_delete() {
        let mut hg = KVGraph::new();
        let n1 = hg.add_node();
        let n2 = hg.add_node();
        let n3 = hg.add_node_with_label("node 3");
        hg.label(&n1, "node 1").unwrap();
        hg.label(&n2, "node 2").unwrap();
        hg.insert(&n1, "test", "failure".to_string()).unwrap();
        hg.insert(&n1, "weight", 1.0_f32).unwrap();
        hg.insert(&n1, "booty", true).unwrap();
        hg.insert(&n2, "weight", 2.2_f32).unwrap();
        hg.insert(&n2, "booty", false).unwrap();
        hg.insert(&n2, "defense", 0_u8).unwrap();
        let nodes = vec![n1, n2, n3];
        let e1 = hg.add_edge(&[n1, n2]);
        hg.add_edge(&[nodes[0], nodes[2]]);
        hg.insert(&e1, "defense", 3_u8).unwrap();

        // I'm not sure how to validate the output dataframes
        // other than manual inspection at the moment.
        dbg!(hg.get(&n1, "test"));
        println!("{:}", hg.dataframe_of_nodes());
        println!("{:}", hg.dataframe_of_edges());
        println!("{:}", hg.dataframe());
    }
}
