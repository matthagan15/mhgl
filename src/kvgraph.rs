#[cfg(feature = "polars")]
use polars::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{EdgeSet, HGraph, HyperGraph};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub fn dtype(&self) -> DataType {
        match self {
            Value::Bool(_) => DataType::Bool,
            Value::UInt8(_) => DataType::UInt8,
            Value::UInt16(_) => DataType::UInt16,
            Value::UInt32(_) => DataType::UInt32,
            Value::UInt64(_) => DataType::UInt64,
            Value::Int8(_) => DataType::Int8,
            Value::Int16(_) => DataType::Int16,
            Value::Int32(_) => DataType::Int32,
            Value::Int64(_) => DataType::Int64,
            Value::Float32(_) => DataType::Float32,
            Value::Float64(_) => DataType::Float64,
            Value::String(_) => DataType::String,
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

pub struct KVGraph {
    core: HGraph<HashMap<String, Value>, HashMap<String, Value>, u128, u128>,
    schema: IndexMap<String, DataType>,
}

impl KVGraph {
    pub fn new() -> Self {
        Self {
            core: HGraph::new(),
            schema: IndexMap::from([
                ("label".to_string(), DataType::String),
                ("id".to_string(), DataType::String),
                ("nodes".to_string(), DataType::String),
                ("labelled_nodes".to_string(), DataType::String),
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

    pub fn remove_edge(&mut self, edge_id: Uuid) {
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

    pub fn find_id<N>(&self, nodes: N) -> Option<Uuid>
    where
        N: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes.as_ref().iter().map(|id| id.as_u128()).collect();
        self.core.find_id(node_vec).map(|id| Uuid::from_u128(id))
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
    pub fn get_schema(&self) -> Vec<(String, DataType)> {
        self.schema.clone().into_iter().collect()
    }

    fn nodes_string(&self, id: &u128) -> Option<String> {
        let mut s = String::from("[");
        if self.core.nodes.contains_key(&id) {
            let node_id = Uuid::from_u128(*id);
            s.push_str(&node_id.to_string()[..]);
            s.push(']');
            Some(s)
        } else if self.core.edges.contains_key(&id) {
            let edge_set = &self.core.edges.get(&id).unwrap().nodes;
            for ix in 0..(edge_set.len() - 1) {
                let node_id = Uuid::from_u128(edge_set.0[ix]);
                s.push_str(&node_id.to_string()[..]);
                s.push(',');
            }
            let last_node_id = Uuid::from_u128(edge_set.0[edge_set.len() - 1]);
            s.push_str(&last_node_id.to_string()[..]);
            s.push(']');
            Some(s)
        } else {
            None
        }
    }

    fn labelled_nodes_string(&self, id: &u128) -> Option<String> {
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
    pub fn dataframe_of_ids<IDs>(&self, ids: IDs) -> DataFrame
    where
        IDs: AsRef<[Uuid]>,
    {
        let mut df = DataFrame::default();
        ids.as_ref()
            .into_iter()
            .map(|id| (id.as_u128(), id.to_string()))
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
                            DataType::Bool => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<bool>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::UInt8 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u8>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::UInt16 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u16>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::UInt32 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u32>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::UInt64 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<u64>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::Int8 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i8>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::Int16 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i16>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::Int32 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i32>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::Int64 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<i64>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::Float32 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<f32>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::Float64 => {
                                let s = Series::new(
                                    &key[..],
                                    [kv_store.get(key).map(|val| Into::<f64>::into(val.clone()))],
                                );
                                id_df.with_column(s).expect("Couldn't add column.");
                            }
                            DataType::String => {
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
        self.dataframe_of_ids(
            self.core
                .nodes
                .keys()
                .map(|id| Uuid::from_u128(*id))
                .collect::<Vec<_>>(),
        )
    }

    /// Collects the dataframe for all edges in the hypergraph. If a key is not
    /// present for an edge then 'null' is used in the dataframe.
    #[cfg(feature = "polars")]
    pub fn dataframe_of_edges(&self) -> DataFrame {
        self.dataframe_of_ids(
            self.core
                .edges
                .keys()
                .map(|id| Uuid::from_u128(*id))
                .collect::<Vec<_>>(),
        )
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
}

impl HyperGraph for KVGraph {
    type NodeID = Uuid;

    type EdgeID = Uuid;

    fn edges_containing_nodes<Nodes>(&self, nodes: Nodes) -> Vec<Self::EdgeID>
    where
        Nodes: AsRef<[Self::NodeID]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .edges_containing_nodes(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    fn edges_containing_edge(&self, edge: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core
            .edges_containing_edge(&edge.as_u128())
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    fn link(&self, edge: &Self::EdgeID) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)> {
        self.core
            .link(&edge.as_u128())
            .into_iter()
            .map(|(id, nodes)| {
                (
                    Uuid::from_u128(id),
                    nodes.into_iter().map(|id| Uuid::from_u128(id)).collect(),
                )
            })
            .collect()
    }

    fn link_of_nodes<Nodes>(&self, nodes: Nodes) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)>
    where
        Nodes: AsRef<[Self::NodeID]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .link_of_nodes(node_vec)
            .into_iter()
            .map(|(id, nodes)| {
                (
                    Uuid::from_u128(id),
                    nodes.into_iter().map(|id| Uuid::from_u128(id)).collect(),
                )
            })
            .collect()
    }

    fn maximal_edges_containing_edge(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core
            .maximal_edges_containing_edge(&edge_id.clone().as_u128())
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    fn maximal_edges_containing_nodes<Nodes>(&self, nodes: Nodes) -> Vec<Self::EdgeID>
    where
        Nodes: AsRef<[Self::NodeID]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .maximal_edges_containing_nodes(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    fn edges_of_size(&self, card: usize) -> Vec<Self::EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() == card)
            .map(|(id, _)| Uuid::from_u128(*id))
            .collect()
    }

    fn skeleton(&self, cardinality: usize) -> Vec<Self::EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= cardinality)
            .map(|(id, _)| Uuid::from_u128(*id))
            .collect()
    }

    fn boundary_up(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core
            .boundary_up(&edge_id.as_u128())
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    fn boundary_down(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
        self.core
            .boundary_down(&edge_id.as_u128())
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    fn boundary_up_nodes<Nodes>(&self, nodes: Nodes) -> Vec<Self::EdgeID>
    where
        Nodes: AsRef<[Self::NodeID]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .boundary_up_nodes(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    fn boundary_down_nodes<Nodes>(&self, nodes: Nodes) -> Vec<Self::EdgeID>
    where
        Nodes: AsRef<[Self::NodeID]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .boundary_down_nodes(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }
}

mod tests {
    #[cfg(feature = "polars")]
    use polars::prelude::*;

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
        let e1 = hg.add_edge(&[n1, n2]).unwrap();
        let e2 = hg.add_edge(&[nodes[0], nodes[2]]).unwrap();
        hg.insert(&e1, "defense", 3_u8).unwrap();

        // I'm not sure how to validate the output dataframes
        // other than manual inspection at the moment.
        dbg!(hg.get(&n1, "test"));
        println!("{:}", hg.dataframe_of_nodes());
        println!("{:}", hg.dataframe_of_edges());
        println!("{:}", hg.dataframe());
    }
}
