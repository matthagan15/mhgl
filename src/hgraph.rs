use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use fxhash::{FxHashMap, FxHashSet};
use hashbrown::HashTable;
use serde::{Deserialize, Serialize};

use crate::{ConGraph, EdgeID, NodeID};
use crate::{EdgeSet, HyperGraph};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Node<NodeData> {
    pub containing_edges: FxHashSet<EdgeID>,
    pub data: NodeData,
}

#[allow(dead_code)]
impl<NodeData> Node<NodeData> {
    pub fn new(data: NodeData) -> Self {
        Node {
            containing_edges: FxHashSet::default(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Edge<EdgeData> {
    pub nodes: EdgeSet,
    pub data: EdgeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// An undirected hypergraph structure that is generic over structs stored
/// for nodes and edges, as well as the ID types used for both (with defaults of `u32` and `u64`). Does not allow for duplicate edges and panics if the data type used for either type of IDs runs out of options. IDs are simple counters and IDs cannot be reused if the node or edge is deleted.
///
/// Nodes are added with `add_node(data)` and edges with `add_edge(node_slice, data)` and removed similarly. Data of a node or edge can be accessed with the
/// `borrow_node`, `borrow_edge` functions and their mutable variants. If you forget the id associated with a collection of nodes you can query the `HGraph`
/// with `find_id(node_slice)` to retrieve the edge's id, if one exists.
///
/// Currently this structure just uses `HashMap`s and edge lists to organize
/// everything, as that was the easiest path to a working structure. This may
/// change to a trie-type structure called a Simplex Tree used in projects such
/// as Gudhi. On my first evaluation it did not seem particularly beneficial
/// asymptotically for computing links, but it may be worth investigating.
#[serde(from = "StoredHGraph<NodeData, EdgeData>")]
pub struct HGraph<NodeData, EdgeData> {
    next_node_id: NodeID,
    next_edge_id: EdgeID,
    pub(crate) edges: FxHashMap<EdgeID, Edge<EdgeData>>,
    pub(crate) nodes: FxHashMap<NodeID, Node<NodeData>>,
    /// Every edge's id, keyed by the hash of its node set: `find_id` is one
    /// lookup. Only ids are stored; equality is checked against `edges`.
    #[serde(skip)]
    pub(crate) edge_ids: HashTable<EdgeID>,
}

/// The fields an `HGraph` is serialized with. It deserializes through this,
/// so that `edge_ids`, which is not stored, is rebuilt from the edges.
#[derive(Deserialize)]
struct StoredHGraph<NodeData, EdgeData> {
    next_node_id: NodeID,
    next_edge_id: EdgeID,
    edges: FxHashMap<EdgeID, Edge<EdgeData>>,
    nodes: FxHashMap<NodeID, Node<NodeData>>,
}

impl<NodeData, EdgeData> From<StoredHGraph<NodeData, EdgeData>> for HGraph<NodeData, EdgeData> {
    fn from(stored: StoredHGraph<NodeData, EdgeData>) -> Self {
        Self {
            next_node_id: stored.next_node_id,
            next_edge_id: stored.next_edge_id,
            edge_ids: build_edge_ids(&stored.edges),
            edges: stored.edges,
            nodes: stored.nodes,
        }
    }
}

/// The hash `edge_ids` is keyed by: a sorted, deduplicated node slice.
fn hash_nodes(nodes: &[NodeID]) -> u64 {
    fxhash::hash64(nodes)
}

/// An `edge_ids` table holding every edge of `edges`.
fn build_edge_ids<EdgeData>(edges: &FxHashMap<EdgeID, Edge<EdgeData>>) -> HashTable<EdgeID> {
    let rehash = |id: &EdgeID| hash_nodes(&edges[id].nodes.0);
    let mut edge_ids = HashTable::with_capacity(edges.len());
    for (id, edge) in edges.iter() {
        edge_ids.insert_unique(hash_nodes(&edge.nodes.0), *id, rehash);
    }
    edge_ids
}

/// Adds `id` to `edge_ids`. Its edge, with its final node set, must already be
/// in `edges`, and no other edge may have that node set.
fn insert_edge_id<EdgeData>(
    edge_ids: &mut HashTable<EdgeID>,
    edges: &FxHashMap<EdgeID, Edge<EdgeData>>,
    id: EdgeID,
) {
    let rehash = |id: &EdgeID| hash_nodes(&edges[id].nodes.0);
    edge_ids.insert_unique(hash_nodes(&edges[&id].nodes.0), id, rehash);
}

/// Removes `id` from `edge_ids`, where it is stored under `nodes`; does nothing
/// if it is not there. Matches on the id, so an edge with the same nodes stays.
fn remove_edge_id(edge_ids: &mut HashTable<EdgeID>, nodes: &[NodeID], id: EdgeID) {
    if let Ok(entry) = edge_ids.find_entry(hash_nodes(nodes), |other| *other == id) {
        entry.remove();
    }
}

impl<NodeData, EdgeData> HGraph<NodeData, EdgeData> {
    /// If you have a `ConGraph` and data for each node and edge you can
    /// build a `HGraph`.
    pub fn from_congraph<NodeFn, EdgeFn>(
        cgraph: ConGraph,
        node_data: NodeFn,
        edge_data: EdgeFn,
    ) -> Self
    where
        NodeFn: Fn(&NodeID) -> NodeData,
        EdgeFn: Fn(&EdgeID) -> EdgeData,
    {
        let next_node_id = cgraph.core.next_node_id;
        let next_edge_id = cgraph.core.next_edge_id;
        let nodes = cgraph
            .core
            .nodes
            .into_iter()
            .map(|(id, node)| {
                (
                    id,
                    Node {
                        containing_edges: node.containing_edges,
                        data: node_data(&id),
                    },
                )
            })
            .collect();
        let edges = cgraph
            .core
            .edges
            .into_iter()
            .map(|(id, edge)| {
                (
                    id,
                    Edge {
                        nodes: edge.nodes,
                        data: edge_data(&id),
                    },
                )
            })
            .collect();
        Self {
            next_node_id,
            next_edge_id,
            edges,
            nodes,
            edge_ids: cgraph.core.edge_ids,
        }
    }
}

impl<N, E> HGraph<N, E>
where
    N: Default,
    E: Default,
{
    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<NodeID> {
        (0..num_nodes)
            .map(|_| self.add_node(N::default()))
            .collect()
    }
}
impl<NodeData, EdgeData> HGraph<NodeData, EdgeData> {
    pub fn new() -> Self {
        Self {
            next_node_id: 0,
            next_edge_id: 0,
            edges: FxHashMap::default(),
            nodes: FxHashMap::default(),
            edge_ids: HashTable::new(),
        }
    }

    /// Returns the new id if a node can be added, `panic`s if the graph
    /// is out of space to add new nodes.
    pub fn add_node(&mut self, node: NodeData) -> NodeID {
        let node_id = self.next_node_id;
        if self.next_node_id == NodeID::MAX {
            panic!("The storage type for NodeIDs ran out of space.")
        }
        self.next_node_id += 1;

        let new_node = Node {
            containing_edges: FxHashSet::default(),
            data: node,
        };
        let insert = self.nodes.insert(node_id, new_node);
        if insert.is_some() {
            panic!("For some reason we encountered the same node_id twice.")
        }
        node_id
    }

    /// Maps `node1` onto `node2` and adjusts edges correspondingly.
    /// maps all edges e1 = {node1, ...} to e2 = {node2, ...} and clobbers
    /// the node1 data with the node2 data.
    /// returns if both nodes are not present
    /// If an edge {e1, e2} exists the edge is removed.
    pub fn concatenate_nodes(&mut self, node1: &NodeID, node2: &NodeID) {
        if self.nodes.contains_key(node1) == false
            || self.nodes.contains_key(node2) == false
            || node1 == node2
        {
            return;
        }
        let mut node1_d = self.nodes.remove(node1).unwrap();
        let mut new_edges = Vec::new();
        for edge in node1_d.containing_edges.drain() {
            let e = self.edges.get_mut(&edge).unwrap();
            remove_edge_id(&mut self.edge_ids, &e.nodes.0, edge);
            e.nodes.remove_node(node1);
            e.nodes.add_node(*node2);
            new_edges.push(edge);
        }
        let node2_ref = self.nodes.get_mut(node2).unwrap();
        for e in new_edges.iter() {
            node2_ref.containing_edges.insert(*e);
        }
        // A rewritten edge that is now a single node, or a duplicate of an edge
        // already in the table, is removed; so an edge that was already there
        // wins over a rewritten one.
        for edge in new_edges {
            let nodes = &self.edges[&edge].nodes.0;
            let drop_edge = nodes.len() == 1 || self.find_id_sorted_nodes(nodes).is_some();
            if drop_edge {
                self.remove_edge(edge);
            } else {
                insert_edge_id(&mut self.edge_ids, &self.edges, edge);
            }
        }
    }

    /// Creates an edge in the hypergraph, if the edge already exists it will
    /// delete the old data and replace it with the newly provided data.
    /// ### `panic`s
    /// - If all nodes are not present in the hypergraph
    /// - If you create more edges than allowable by the `EdgeID` storage type
    pub fn add_edge(&mut self, edge: impl AsRef<[NodeID]>, data: EdgeData) -> EdgeID {
        let edge_set: EdgeSet = edge.into();
        if let Some(id) = self.find_id_sorted_nodes(&edge_set.0) {
            let e = self.edges.remove(&id).unwrap();
            self.edges.insert(
                id,
                Edge {
                    nodes: e.nodes,
                    data: data,
                },
            );
            return id;
        }

        let id = self.next_edge_id;
        // Note this technically means we can't use all possible edges
        // but missing 1 out of the 2^64 - 1 possibilities ain't bad.
        if self.next_edge_id == EdgeID::MAX {
            panic!("Ran out of edges, need to use a bigger EdgeID representation.")
        }
        self.next_edge_id += 1;

        let nodes = edge_set.node_vec();
        for node in nodes.iter() {
            if self.nodes.contains_key(&node) == false {
                panic!("Adding edge but a provided node is not present in the hypergraph.")
            }
        }
        for node in nodes.iter() {
            let node_link = self
                .nodes
                .get_mut(node)
                .expect("Node should already be present, I just added it.");
            node_link.containing_edges.insert(id.clone());
        }
        let edge = Edge {
            nodes: edge_set,
            data,
        };
        self.edges.insert(id.clone(), edge);
        insert_edge_id(&mut self.edge_ids, &self.edges, id);
        id
    }

    /// Solely for use by KVGraph, which needs to generate Uuids for each entry.
    /// Returns existing `NodeData` if it was there.
    #[allow(dead_code)]
    pub(crate) fn add_node_with_id(&mut self, node: NodeData, id: NodeID) -> Option<NodeData> {
        let new_node = Node {
            containing_edges: FxHashSet::default(),
            data: node,
        };
        self.nodes
            .insert(id, new_node)
            .map(|old_node| old_node.data)
    }

    /// For `KVGraph` only.
    #[allow(dead_code)]
    pub(crate) fn add_edge_with_id<E>(
        &mut self,
        edge: E,
        data: EdgeData,
        id: EdgeID,
    ) -> Option<EdgeData>
    where
        E: Into<EdgeSet>,
    {
        let edge_set: EdgeSet = edge.into();
        if self.find_id_sorted_nodes(&edge_set.0).is_some() {
            return None;
        }

        let nodes = edge_set.node_vec();
        for node in nodes.iter() {
            if self.nodes.contains_key(&node) == false {
                return None;
            }
        }
        for node in nodes.iter() {
            let node_link = self
                .nodes
                .get_mut(node)
                .expect("Node should already be present, I just added it.");
            node_link.containing_edges.insert(id.clone());
        }
        let edge = Edge {
            nodes: edge_set,
            data,
        };
        let old_edge = self.edges.insert(id.clone(), edge);
        if let Some(old_edge) = &old_edge {
            remove_edge_id(&mut self.edge_ids, &old_edge.nodes.0, id);
        }
        insert_edge_id(&mut self.edge_ids, &self.edges, id);
        old_edge.map(|edge_struct| edge_struct.data)
    }

    /// This will remove the node from the graph and any edges containing it.
    /// The node will not be reused in the future. If this leaves an edge
    /// empty the edge will be removed from the graph.
    ///
    /// **Warning:** if removing the node shrinks an edge onto the nodes of an
    /// edge that already exists, the shrunk edge is removed, data included, and
    /// the existing edge is kept. This is the policy of
    /// [`concatenate_nodes`](Self::concatenate_nodes), and it keeps the graph
    /// free of duplicate edges.
    pub fn remove_node(&mut self, node: NodeID) -> Option<NodeData> {
        if self.nodes.contains_key(&node) == false {
            return None;
        }
        let removed_node = self.nodes.remove(&node).unwrap();
        for effected_edge_id in removed_node.containing_edges.iter() {
            let effected_edge = self
                .edges
                .get_mut(&effected_edge_id)
                .expect("Effected edge not found.");
            remove_edge_id(
                &mut self.edge_ids,
                &effected_edge.nodes.0,
                *effected_edge_id,
            );
            effected_edge.nodes.remove_node(&node);
        }
        // Two shrunk edges cannot collide: both held `node` and differed elsewhere.
        for effected_edge_id in removed_node.containing_edges.iter() {
            let nodes = &self.edges[effected_edge_id].nodes.0;
            let drop_edge = nodes.is_empty() || self.find_id_sorted_nodes(nodes).is_some();
            if drop_edge {
                self.remove_edge(*effected_edge_id);
            } else {
                insert_edge_id(&mut self.edge_ids, &self.edges, *effected_edge_id);
            }
        }
        Some(removed_node.data)
    }

    /// Returns the `EdgeData` of the associated edge if it existed and `None`
    /// if an incorrect edge was provided.
    pub fn remove_edge(&mut self, edge_id: EdgeID) -> Option<EdgeData> {
        if let Some(e) = self.edges.remove(&edge_id) {
            remove_edge_id(&mut self.edge_ids, &e.nodes.0, edge_id);
            for node in e.nodes.0.iter() {
                let containing_edges = self.nodes.get_mut(node).expect("Why is edge not in here.");
                containing_edges.containing_edges.remove(&edge_id);
            }
            Some(e.data)
        } else {
            None
        }
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn nodes(&self) -> Vec<NodeID> {
        self.nodes.keys().cloned().collect()
    }

    pub fn edges(&self) -> Vec<EdgeID> {
        self.edges.keys().cloned().collect()
    }

    /// Returns the previously existing data of the provided node, returns
    /// `None` if the node does not exist.
    pub fn insert_node_data(&mut self, node: &NodeID, new_data: NodeData) -> Option<NodeData> {
        if let Some(old_node) = self.nodes.remove(node) {
            let new_node = Node {
                containing_edges: old_node.containing_edges,
                data: new_data,
            };
            self.nodes.insert(node.clone(), new_node);
            Some(old_node.data)
        } else {
            None
        }
    }

    /// Returns the previously existing data of the provided edge, returns
    /// `None` if the edge does not exist.
    pub fn insert_edge_data(&mut self, edge_id: &EdgeID, new_data: EdgeData) -> Option<EdgeData> {
        if let Some(old_edge) = self.edges.remove(edge_id) {
            let new_edge = Edge {
                nodes: old_edge.nodes,
                data: new_data,
            };
            self.edges.insert(edge_id.clone(), new_edge);
            Some(old_edge.data)
        } else {
            None
        }
    }

    /// Borrows the data of the provided node.
    pub fn get_node(&self, node: &NodeID) -> Option<&NodeData> {
        self.nodes.get(node).map(|big_node| &big_node.data)
    }

    /// Borrows the data mutably of the provided node.
    pub fn get_node_mut(&mut self, node: &NodeID) -> Option<&mut NodeData> {
        self.nodes.get_mut(node).map(|big_node| &mut big_node.data)
    }

    /// Borrows the data of the provided edge.
    pub fn get_edge(&self, edge: &EdgeID) -> Option<&EdgeData> {
        self.edges.get(edge).map(|big_edge| &big_edge.data)
    }

    /// Borrows the data mutably of the provided edge.
    pub fn get_edge_mut(&mut self, edge: &EdgeID) -> Option<&mut EdgeData> {
        self.edges.get_mut(edge).map(|big_edge| &mut big_edge.data)
    }

    /// Applies the provided closure to each edge.
    pub fn apply_edge_data_map<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut EdgeData),
    {
        for edge in self.edges.values_mut() {
            f(&mut edge.data)
        }
    }

    /// Applies the provided closure to each node.
    pub fn apply_node_data_map<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut NodeData),
    {
        for node in self.nodes.values_mut() {
            f(&mut node.data)
        }
    }

    /// In case you forget :)
    pub fn find_id(&self, nodes: impl AsRef<[NodeID]>) -> Option<EdgeID> {
        // check if sorted
        let nodes_ref = nodes.as_ref();
        if nodes_ref.len() == 0 {
            return None;
        }
        if nodes_ref.len() == 1 {
            return self.find_id_sorted_nodes(nodes_ref);
        }
        if nodes_ref.is_sorted() {
            let no_duplicates: bool = (0..nodes_ref.len() - 1)
                .map(|ix| nodes_ref[ix] != nodes_ref[ix + 1])
                .reduce(|acc, x| acc && x)
                .unwrap();
            if no_duplicates {
                return self.find_id_sorted_nodes(nodes_ref);
            }
        }
        let edge_set = EdgeSet::from(nodes);
        self.find_id_sorted_nodes(&edge_set.0)
    }

    fn find_id_sorted_nodes(&self, nodes: &[NodeID]) -> Option<EdgeID> {
        if nodes.is_empty() {
            return None;
        }
        self.edge_ids
            .find(hash_nodes(nodes), |edge_id| {
                if let Some(edge) = self.edges.get(edge_id) {
                    edge.nodes.0 == nodes
                } else {
                    false
                }
            })
            .cloned()
    }
}

impl<NodeData, EdgeData> HGraph<NodeData, EdgeData>
where
    NodeData: Clone,
    EdgeData: Clone,
{
    /// Returns a new HGraph with the edges that pass the filter, along with all the nodes
    /// needed to support each edge.
    pub fn filter_by_edge<F>(&self, filter: F) -> HGraph<NodeData, EdgeData>
    where
        F: Fn(EdgeID) -> bool,
    {
        let new_edges: FxHashMap<EdgeID, Edge<EdgeData>> = self
            .edges
            .iter()
            .filter_map(|x| {
                if filter(*x.0) {
                    Some((x.0.clone(), x.1.clone()))
                } else {
                    None
                }
            })
            .collect();
        let mut nodes_contained_in_edge = HashSet::new();
        for edge in new_edges.iter() {
            for node in edge.1.nodes.0.iter() {
                nodes_contained_in_edge.insert(*node);
            }
        }
        let new_nodes: FxHashMap<NodeID, Node<NodeData>> = nodes_contained_in_edge
            .into_iter()
            .map(|node| {
                let mut new_node = self.nodes.get(&node).cloned().unwrap();
                let new_node_edges = new_node
                    .containing_edges
                    .iter()
                    .filter(|edge_id| filter(**edge_id))
                    .cloned()
                    .collect();
                new_node.containing_edges = new_node_edges;
                (node, new_node)
            })
            .collect();
        HGraph {
            next_node_id: self.next_node_id,
            next_edge_id: self.next_edge_id,
            edge_ids: build_edge_ids(&new_edges),
            edges: new_edges,
            nodes: new_nodes,
        }
    }

    pub fn filter_nodes<F>(&self, filter: F) -> Vec<NodeID>
    where
        F: Fn(NodeID) -> bool,
    {
        self.nodes
            .keys()
            .filter(|node| filter(**node))
            .cloned()
            .collect()
    }

    pub fn star(&self, nodes: impl AsRef<[NodeID]>) -> HGraph<NodeData, EdgeData> {
        let mut star = HashSet::new();
        let nodes_set = EdgeSet::from(nodes.as_ref());
        for node in nodes.as_ref().iter() {
            for edge in self.containing_edges_of_nodes([*node]) {
                let edge_nodes =
                    EdgeSet::from(self.query_edge(&edge).expect("Faulty edge detected."));
                if nodes_set.contains(&edge_nodes) {
                    star.insert(edge);
                } else if edge_nodes.contains_strict(&nodes_set) {
                    star.insert(edge);
                }
            }
        }
        let filter = |edge_id| star.contains(&edge_id);
        self.filter_by_edge(filter)
    }

    pub fn edges_containing_node(&self, node: &NodeID) -> Option<impl Iterator<Item = &EdgeID>> {
        self.nodes
            .get(node)
            .map(|node| node.containing_edges.iter())
    }
}

impl<NData, EData> HyperGraph for HGraph<NData, EData> {
    fn query_edge(&self, edge: &EdgeID) -> Option<Vec<NodeID>> {
        self.edges
            .get(edge)
            .map(|big_edge| big_edge.nodes.node_vec())
    }

    fn containing_edges_of_nodes(&self, nodes: impl AsRef<[NodeID]>) -> Vec<EdgeID> {
        let nodes_set: EdgeSet = nodes.into();
        let first = nodes_set.get_first_node().unwrap();
        if self.nodes.contains_key(&first) == false {
            return vec![];
        }
        let candidate_ids = self.nodes.get(&first).unwrap();
        let mut ret = Vec::new();
        for candidate_id in candidate_ids.containing_edges.iter() {
            let candidate = self
                .edges
                .get(candidate_id)
                .expect("Edge invariant violated.");
            if candidate.nodes.contains_strict(&nodes_set) {
                ret.push(candidate_id.clone());
            }
        }
        ret
    }

    fn containing_edges(&self, edge: &EdgeID) -> Vec<EdgeID> {
        if self.edges.contains_key(edge) == false {
            return Vec::new();
        }
        let edge = self.edges.get(edge).unwrap();
        let first = edge.nodes.get_first_node().unwrap();
        let candidate_ids = self.nodes.get(&first).unwrap();
        let mut ret = Vec::new();
        for candidate_id in candidate_ids.containing_edges.iter() {
            let candidate = self
                .edges
                .get(candidate_id)
                .expect("Edge invariant violated.");
            if candidate.nodes.contains_strict(&edge.nodes) {
                ret.push(candidate_id.clone());
            }
        }
        ret
    }

    fn link(&self, edge: &EdgeID) -> Vec<(EdgeID, Vec<NodeID>)> {
        if self.edges.contains_key(edge) == false {
            return Vec::new();
        }
        let containing_edges = self.containing_edges(edge);
        let edge = self.edges.get(edge).unwrap();
        containing_edges
            .into_iter()
            .filter_map(|id| {
                if let Some(local_link) = self
                    .edges
                    .get(&id)
                    .expect("Broken edge invariant found in link.")
                    .nodes
                    .link(&edge.nodes)
                {
                    Some((id, local_link.to_node_vec()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn link_of_nodes(&self, nodes: impl AsRef<[NodeID]>) -> Vec<(EdgeID, Vec<NodeID>)> {
        let edge: EdgeSet = nodes.into();
        let containing_edges = self.containing_edges_of_nodes(edge.node_vec());
        containing_edges
            .into_iter()
            .filter_map(|id| {
                if let Some(local_link) = self
                    .edges
                    .get(&id)
                    .expect("Broken edge invariant found in link.")
                    .nodes
                    .link(&edge)
                {
                    Some((id, local_link.to_node_vec()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn maximal_edges(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        let containing_edges = self.containing_edges(edge_id);
        if containing_edges.is_empty() {
            return Vec::new();
        }
        let mut submaximal_edges = HashSet::new();
        for ix in 0..containing_edges.len() {
            if submaximal_edges.contains(&containing_edges[ix]) {
                continue;
            }
            let edge_ix = self
                .edges
                .get(&containing_edges[ix])
                .expect("Edge invariant broken.");
            for jx in 0..containing_edges.len() {
                if ix == jx {
                    continue;
                }
                let edge_jx = self
                    .edges
                    .get(&containing_edges[jx])
                    .expect("Edge invariant broken.");
                if edge_jx.nodes.contains_strict(&edge_ix.nodes) {
                    submaximal_edges.insert(containing_edges[ix].clone());
                } else if edge_ix.nodes.contains_strict(&edge_jx.nodes) {
                    submaximal_edges.insert(containing_edges[jx].clone());
                }
            }
        }
        containing_edges
            .into_iter()
            .filter(|id| submaximal_edges.contains(id) == false)
            .collect()
    }

    fn maximal_edges_of_nodes(&self, nodes: impl AsRef<[NodeID]>) -> Vec<EdgeID> {
        let containing_edges = self.containing_edges_of_nodes(nodes);
        if containing_edges.is_empty() {
            return Vec::new();
        }
        let mut submaximal_edges = HashSet::new();
        for ix in 0..containing_edges.len() {
            if submaximal_edges.contains(&containing_edges[ix]) {
                continue;
            }
            let edge_ix = self
                .edges
                .get(&containing_edges[ix])
                .expect("Edge invariant broken.");
            let mut is_edge_ix_maximal = true;
            for jx in 0..containing_edges.len() {
                if ix == jx {
                    continue;
                }
                let edge_jx = self
                    .edges
                    .get(&containing_edges[jx])
                    .expect("Edge invariant broken.");
                if edge_jx.nodes.contains_strict(&edge_ix.nodes) {
                    is_edge_ix_maximal = false;
                    break;
                }
            }
            if is_edge_ix_maximal {
                submaximal_edges.insert(containing_edges[ix]);
            }
        }
        submaximal_edges.into_iter().collect()
    }

    fn edges_of_size(&self, card: usize) -> Vec<EdgeID> {
        self.edges
            .iter()
            .filter(|(_, e)| e.nodes.len() == card)
            .map(|(id, _)| id)
            .cloned()
            .collect()
    }

    fn boundary_up(&self, edge_id: &EdgeID) -> Vec<Vec<NodeID>> {
        let containing_edges = self.containing_edges(edge_id);
        if containing_edges.is_empty() {
            return Vec::new();
        }
        let given_edge_len = self
            .edges
            .get(edge_id)
            .expect("Should have checked for edge_id being proper in containing_edges")
            .nodes
            .len();
        let mut boundary = Vec::new();
        for id in containing_edges {
            let containing_edge = self
                .edges
                .get(&id)
                .expect("Containing edges broken from boundary_up");
            if containing_edge.nodes.len() == given_edge_len + 1 {
                boundary.push(self.edges.get(&id).unwrap().nodes.node_vec());
            }
        }
        boundary
    }

    fn boundary_down(&self, edge_id: &EdgeID) -> Vec<Vec<NodeID>> {
        if self.edges.contains_key(edge_id) == false {
            return Vec::new();
        }
        let edge_set = &self.edges.get(edge_id).unwrap().nodes;
        if edge_set.len() == 1 {
            return Vec::new();
        } else if edge_set.len() == 2 {
            return edge_set
                .node_vec()
                .into_iter()
                .map(|node| vec![node])
                .collect();
        }
        let mut boundary = Vec::new();
        for ix in 0..edge_set.len() {
            let mut possible = edge_set.node_vec();
            possible.remove(ix);
            if let Some(id) = self.find_id(possible) {
                boundary.push(self.edges.get(&id).unwrap().nodes.node_vec());
            }
        }
        boundary
    }

    fn boundary_up_of_nodes(&self, nodes: impl AsRef<[NodeID]>) -> Vec<Vec<NodeID>> {
        let nodes_ref = nodes.as_ref();
        let given_nodes_len = nodes_ref.len();
        let containing_edges = self.containing_edges_of_nodes(nodes);
        if containing_edges.is_empty() {
            return Vec::new();
        }
        let mut boundary = Vec::new();
        for id in containing_edges {
            let containing_edge = self
                .edges
                .get(&id)
                .expect("Containing edges broken from boundary_up");
            if containing_edge.nodes.len() == given_nodes_len + 1 {
                boundary.push(self.edges.get(&id).unwrap().nodes.node_vec());
            }
        }
        boundary
    }

    fn boundary_down_of_nodes(&self, nodes: impl AsRef<[NodeID]>) -> Vec<Vec<NodeID>> {
        let edge_set: EdgeSet = nodes.into();
        if edge_set.len() == 1 {
            return Vec::new();
        } else if edge_set.len() == 2 {
            return edge_set
                .node_vec()
                .into_iter()
                .map(|node| vec![node])
                .collect();
        }
        let mut boundary = Vec::new();
        for ix in 0..edge_set.len() {
            let mut possible = edge_set.node_vec();
            possible.remove(ix);
            if let Some(id) = self.find_id(possible) {
                boundary.push(self.edges.get(&id).unwrap().nodes.node_vec());
            }
        }
        boundary
    }

    fn skeleton(&self, cardinality: usize) -> Vec<EdgeID> {
        self.edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= cardinality)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl<NodeData, EdgeData> HGraph<NodeData, EdgeData>
where
    NodeData: Serialize + for<'a> Deserialize<'a>,
    EdgeData: Serialize + for<'a> Deserialize<'a>,
{
    /// Serializes the struct using `serde_json` and writes it to disk. `panic`s if anything fails.
    pub fn to_disk(&self, path: &Path) {
        let s = serde_json::to_string(self).expect("could not serialize NEGraph");
        let mut file = File::create(path).expect("Cannot create File.");
        file.write_all(s.as_bytes()).expect("Cannot write");
    }

    /// Attempts to deserialize using `serde_json` from the input file.
    pub fn from_file(file: &Path) -> Option<Self> {
        if file.is_file() == false {
            println!("File is not a file?");
            return None;
        }
        if let Ok(file) = File::open(file) {
            let reader = BufReader::new(file);
            let out = serde_json::from_reader(reader);
            if out.is_ok() {
                Some(out.unwrap())
            } else {
                println!("serde_json failed.");
                println!("output: {:?}", out.err());
                None
            }
        } else {
            println!("File opening failed.");
            None
        }
    }
}

impl<NodeData, EdgeData> Display for HGraph<NodeData, EdgeData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.nodes.len() == 0 {
            println!("Graph is empty. Add nodes for more fun.");
            return Ok(());
        }
        let mut s = String::new();
        s.push_str("nodes:\n[");
        let x: Vec<String> = self.nodes.keys().map(|n| n.to_string()).collect();
        for ix in 0..x.len() - 1 {
            s.push_str(&x[ix]);
            s.push_str(", ");
        }
        s.push_str(x.last().unwrap());
        s.push_str("]\n");
        s.push_str("edges:\n");
        for (id, e) in self.edges.iter() {
            s.push_str(&id.to_string()[..]);
            s.push_str(" = ");
            s.push_str(&e.nodes.to_string());
            s.push_str("\n");
        }
        f.write_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::{ConGraph, HyperGraph, NodeID};

    use super::HGraph;

    /// `edge_ids` holds exactly one entry per edge, no two edges share a node
    /// set, and every edge is found from its nodes in any order and with repeats.
    fn assert_edge_ids_consistent<N, E>(hg: &HGraph<N, E>) {
        assert_eq!(hg.edge_ids.len(), hg.edges.len());
        let mut node_sets: Vec<&Vec<NodeID>> = hg.edges.values().map(|e| &e.nodes.0).collect();
        node_sets.sort();
        node_sets.dedup();
        assert_eq!(
            node_sets.len(),
            hg.edges.len(),
            "two edges share a node set"
        );
        for (id, edge) in hg.edges.iter() {
            assert_eq!(hg.find_id(&edge.nodes.0), Some(*id));
            let mut shuffled = edge.nodes.0.clone();
            shuffled.reverse();
            shuffled.push(shuffled[0]);
            assert_eq!(hg.find_id(&shuffled), Some(*id));
        }
    }

    #[test]
    fn syntax() {
        let mut hg = HGraph::<i32, String>::new();
        let a = hg.add_node(-1);
        let b = hg.add_node(3);
        let c = hg.add_node(5);
        let d = hg.add_node(7);

        let e1 = hg.add_edge([a, b], "one".to_string());
        let e2 = hg.add_edge([a, b, c], "two".to_string());
        let e3 = hg.add_edge([a, b, c, d], "three".to_string());

        let found_id = hg.find_id([a, b]);
        let found_string = found_id.map(|id| hg.get_edge(&id)).flatten();
        assert_eq!(found_string, Some(&String::from("one")));
        assert_eq!(None, hg.find_id([a]));

        let mut containing_edges = hg.containing_edges(&e1);
        containing_edges.sort();
        assert_eq!(containing_edges, vec![e2, e3]);

        let mut containing_edges = hg.containing_edges_of_nodes([a]);
        containing_edges.sort();
        assert_eq!(containing_edges, vec![e1, e2, e3]);

        let max_edge1 = hg.maximal_edges(&e1);
        let max_edge2 = hg.maximal_edges(&e2);
        assert_eq!(max_edge1.first(), Some(&e3));
        assert_eq!(max_edge2.first(), Some(&e3));

        let mut link = hg.link(&e1);
        link.sort_by_key(|x| x.0);
        assert_eq!(link, vec![(e2, vec![c]), (e3, vec![c, d])]);

        // Boundaries currently only work on "basis sets", or
        // single edges in the graph. They do not function as
        // linear operators acting on linear combinations of
        // edges yet.
        let boundary_down = hg.boundary_down_of_nodes([a, b, c]);
        let boundary_down_id = hg.find_id(&boundary_down[0][..]);
        assert_eq!(boundary_down_id, Some(e1));

        let mut boundary_single_nodes = hg.boundary_down_of_nodes([a, b]);
        boundary_single_nodes.sort();
        assert_eq!(boundary_single_nodes, vec![vec![a], vec![b]]);

        let boundary_up = hg.boundary_up_of_nodes([a, b, c]);
        let boundary_up_id = hg.find_id(&boundary_up[0][..]);
        assert_eq!(boundary_up_id, Some(e3));
    }

    #[test]
    fn simple_tasks() {
        let mut g = HGraph::<(), ()>::new();

        let nodes: Vec<_> = (0..10).map(|_| g.add_node(())).collect();
        assert_eq!(nodes.len(), 10);
        let e1 = g.add_edge(&[1, 2, 3][..], ());
        let e2 = g.add_edge(vec![1, 2, 4], ());
        g.add_edge([5, 6, 7], ());
        assert!(g.find_id([1, 2, 3]).is_some());
        // is simplex so this should work
        assert!(g.find_id(&[0][..]).is_none());
        let containing_edges = g.containing_edges_of_nodes([1, 2]);
        assert_eq!(containing_edges.len(), 2);
        assert!(containing_edges.contains(&e1));
        assert!(containing_edges.contains(&e2));
        let affected_edges = g.containing_edges_of_nodes([2]);
        g.remove_node(2);
        assert!(affected_edges.contains(&e1));
        assert!(affected_edges.contains(&e2));
        assert!(g.find_id([1, 3]).is_some());
        assert!(g.find_id([1, 2, 3]).is_none());
        let _: Vec<_> = (5..=7).map(|x| g.remove_node(x)).collect();
        assert!(g.find_id([5, 6, 7]).is_none());
    }

    #[test]
    fn link_and_maximal() {
        let mut core = HGraph::<(), ()>::new();
        for _ in 0..7 {
            core.add_node(());
        }
        let e1 = core.add_edge(vec![0, 1], ());
        let _e2 = core.add_edge(vec![0, 6], ());
        let e3 = core.add_edge(vec![0, 3], ());
        let e4 = core.add_edge(vec![0, 1, 4], ());
        let e5 = core.add_edge(vec![0, 1, 4, 5], ());
        let e6 = core.add_edge(vec![0, 2, 6], ());
        let mut maximal_edges = core.maximal_edges_of_nodes([0]);
        maximal_edges.sort();
        let mut expected = vec![e3, e5, e6];
        expected.sort();
        assert_eq!(maximal_edges, expected);

        let mut link = core.link_of_nodes([0, 1]);
        link.sort();
        for ix in 0..link.len() {
            link[ix].1.sort();
        }
        let mut expected_link = vec![(e4.clone(), vec![4]), (e5.clone(), vec![4, 5])];
        expected_link.sort();
        assert_eq!(link, expected_link);

        let star = core.star([1]);
        let mut star_nodes = star.nodes();
        star_nodes.sort();
        assert_eq!(vec![0, 1, 4, 5], star_nodes);
        let mut star_edges = star.edges();
        star_edges.sort();
        assert_eq!(vec![e1, e4, e5], star_edges);
    }

    #[test]
    fn gluing_nodes() {
        let mut hg = HGraph::<(), ()>::new();
        let _nodes: Vec<_> = (0..6).map(|_| hg.add_node(())).collect();
        let e4 = hg.add_edge([0, 1, 2], ());
        hg.add_edge([3, 4, 5], ());
        let e1 = hg.add_edge([0, 1], ());
        let e2 = hg.add_edge([2, 1], ());
        hg.add_edge([0, 2], ());
        let e3 = hg.add_edge([3, 4], ());
        hg.add_edge([5, 4], ());
        hg.add_edge([5, 3], ());
        hg.add_edge([2, 3], ());
        println!("Before Glueing.");
        println!("{:}", hg);
        hg.concatenate_nodes(&2, &3);
        println!("First Glue. 2 -> 3");
        println!("{:}", hg);
        hg.concatenate_nodes(&1, &4);
        println!("Post Glue. 1 -> 4");
        println!("{:}", hg);
        assert!(hg.get_node(&1).is_none());
        assert!(hg.get_node(&2).is_none());
        assert!(hg.get_node(&3).is_some());
        assert!(hg.get_node(&4).is_some());

        let e1_nodes = hg.query_edge(&e1);
        assert!(e1_nodes.is_some());
        let e1_nodes = e1_nodes.unwrap();
        assert!(e1_nodes.len() == 2);
        assert!(e1_nodes.contains(&0));
        assert!(e1_nodes.contains(&4));

        let e4_nodes = hg.query_edge(&e4);
        assert!(e4_nodes.is_some());
        let e4_nodes = e4_nodes.unwrap();
        assert_eq!(e4_nodes.len(), 3);
        assert!(e4_nodes.contains(&0));
        assert!(e4_nodes.contains(&3));
        assert!(e4_nodes.contains(&4));

        assert_eq!(hg.find_id([3, 4]), Some(e3));
        assert!(hg.query_edge(&e2).is_none());
    }

    #[test]
    fn boundaries() {
        let mut hg = HGraph::<u8, u8>::new();
        let _: Vec<_> = (0..10).map(|x| hg.add_node(x)).collect();
        let e1 = hg.add_edge(vec![0, 1], 1);
        let _e2 = hg.add_edge(vec![0, 1, 2], 2);
        let _e3 = hg.add_edge(vec![0, 1, 3], 3);
        let e4 = hg.add_edge(vec![0, 1, 2, 3], 4);
        hg.add_edge(vec![1, 2, 5], 19);

        let expected = vec![vec![0, 1, 2], vec![0, 1, 3]];
        let mut test_1 = hg.boundary_up(&e1);
        test_1.sort();
        assert_eq!(test_1, expected);

        let mut test_2 = hg.boundary_down(&e4);
        test_2.sort();
        assert_eq!(test_2, expected);

        let expected_3 = vec![vec![0, 1, 2, 3]];
        let test_3 = hg.boundary_up_of_nodes(vec![0, 1, 2]);
        assert_eq!(test_3, expected_3);

        let expected_4 = vec![vec![0, 1]];
        let test_4 = hg.boundary_down_of_nodes(vec![0, 1, 3]);
        assert_eq!(test_4, expected_4);
    }
    #[test]
    fn find_id_accepts_any_order_and_repeats() {
        let mut hg = HGraph::<(), ()>::new();
        let n: Vec<_> = (0..6).map(|_| hg.add_node(())).collect();
        let edge = hg.add_edge([n[1], n[3], n[5]], ());
        assert_eq!(hg.find_id([n[1], n[3], n[5]]), Some(edge));
        assert_eq!(hg.find_id([n[5], n[1], n[3]]), Some(edge));
        assert_eq!(hg.find_id([n[3], n[3], n[1], n[5], n[5]]), Some(edge));
        assert_eq!(hg.find_id([n[1], n[3]]), None);
        assert_eq!(hg.find_id([n[1], n[3], n[5], n[0]]), None);
        assert_eq!(hg.find_id(&[] as &[NodeID]), None);
        assert_eq!(hg.find_id([100]), None);
    }

    /// Removing a node can shrink an edge onto one that already exists: the
    /// shrunk edge and its data go, the existing edge stays.
    #[test]
    fn remove_node_drops_the_shrunk_duplicate() {
        let mut hg = HGraph::<(), i32>::new();
        let n: Vec<_> = (0..3).map(|_| hg.add_node(())).collect();
        let existing = hg.add_edge([n[0], n[1]], 1);
        let shrunk = hg.add_edge([n[0], n[1], n[2]], 2);
        hg.remove_node(n[2]);
        assert_eq!(hg.get_edge(&existing), Some(&1));
        assert_eq!(hg.get_edge(&shrunk), None);
        assert_eq!(hg.find_id([n[0], n[1]]), Some(existing));
        assert_eq!(hg.num_edges(), 1);
        assert_edge_ids_consistent(&hg);
    }

    /// Gluing can rewrite an edge onto one that already exists: the rewritten
    /// edge goes and the existing one keeps its data.
    #[test]
    fn concatenate_nodes_keeps_the_existing_edge() {
        let mut hg = HGraph::<(), i32>::new();
        let n: Vec<_> = (0..4).map(|_| hg.add_node(())).collect();
        let existing = hg.add_edge([n[2], n[3]], 10);
        let rewritten = hg.add_edge([n[1], n[3]], 20);
        hg.concatenate_nodes(&n[1], &n[2]);
        assert_eq!(hg.get_edge(&existing), Some(&10));
        assert_eq!(hg.get_edge(&rewritten), None);
        assert_eq!(hg.find_id([n[3], n[2]]), Some(existing));
        assert_edge_ids_consistent(&hg);
    }

    /// `edge_ids` is not serialized; loading rebuilds it, and the format is unchanged.
    #[test]
    fn serde_round_trip_rebuilds_edge_ids() {
        let mut hg = HGraph::<u8, i32>::new();
        let n: Vec<_> = (0..5).map(|i| hg.add_node(i)).collect();
        let e1 = hg.add_edge([n[0], n[1]], 1);
        let e2 = hg.add_edge([n[3], n[1], n[4]], 2);
        let json = serde_json::to_string(&hg).unwrap();
        assert!(!json.contains("edge_ids"));
        let loaded: HGraph<u8, i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.find_id([n[1], n[0]]), Some(e1));
        assert_eq!(loaded.find_id([n[4], n[3], n[1]]), Some(e2));
        assert_edge_ids_consistent(&loaded);
        let reserialized: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&loaded).unwrap()).unwrap();
        assert_eq!(
            reserialized,
            serde_json::from_str::<serde_json::Value>(&json).unwrap()
        );

        let mut cg = ConGraph::new();
        let m = cg.add_nodes(4);
        let f = cg.add_edge(&[m[2], m[0], m[3]]);
        let loaded: ConGraph = serde_json::from_str(&serde_json::to_string(&cg).unwrap()).unwrap();
        assert_eq!(loaded.find_id(&[m[0], m[2], m[3]]), Some(f));
    }

    /// Random adds, edge removals, node removals and gluings keep `edge_ids`
    /// in step with the edges.
    #[test]
    fn edge_ids_stay_consistent_under_random_operations() {
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..200 {
            let mut hg = HGraph::<(), ()>::new();
            let mut nodes: Vec<NodeID> = (0..8).map(|_| hg.add_node(())).collect();
            for _ in 0..40 {
                match rng.gen_range(0..10) {
                    0..=5 => {
                        let k = rng.gen_range(1..=nodes.len().min(4));
                        let edge: Vec<NodeID> = (0..k)
                            .map(|_| nodes[rng.gen_range(0..nodes.len())])
                            .collect();
                        hg.add_edge(&edge, ());
                    }
                    6 => {
                        let edges = hg.edges();
                        if !edges.is_empty() {
                            hg.remove_edge(edges[rng.gen_range(0..edges.len())]);
                        }
                    }
                    7 if nodes.len() > 2 => {
                        let node = nodes.swap_remove(rng.gen_range(0..nodes.len()));
                        hg.remove_node(node);
                    }
                    8 | 9 if nodes.len() > 2 => {
                        let from = nodes.swap_remove(rng.gen_range(0..nodes.len()));
                        let onto = nodes[rng.gen_range(0..nodes.len())];
                        hg.concatenate_nodes(&from, &onto);
                    }
                    _ => {}
                }
                assert_edge_ids_consistent(&hg);
            }
        }
    }
}
