use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use fxhash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::{ConGraph, HgNode};
use crate::{EdgeSet, HyperGraph};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Node<NodeData, EdgeID: HgNode> {
    pub containing_edges: FxHashSet<EdgeID>,
    pub data: NodeData,
}

#[allow(dead_code)]
impl<NodeData, EdgeID: HgNode> Node<NodeData, EdgeID> {
    pub fn new(data: NodeData) -> Self {
        Node {
            containing_edges: FxHashSet::default(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Edge<N: HgNode, EdgeData> {
    pub nodes: EdgeSet<N>,
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
pub struct HGraph<NodeData, EdgeData, NodeID: HgNode = u32, EdgeID: HgNode = u64> {
    next_node_id: NodeID,
    next_edge_id: EdgeID,
    pub(crate) edges: FxHashMap<EdgeID, Edge<NodeID, EdgeData>>,
    pub(crate) nodes: FxHashMap<NodeID, Node<NodeData, EdgeID>>,
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
        NodeFn: Fn(&u32) -> NodeData,
        EdgeFn: Fn(&u64) -> EdgeData,
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
        }
    }
}

impl<N, E, NodeID: HgNode, EdgeID: HgNode> HGraph<N, E, NodeID, EdgeID>
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
impl<NodeData, EdgeData, NodeID: HgNode, EdgeID: HgNode>
    HGraph<NodeData, EdgeData, NodeID, EdgeID>
{
    pub fn new() -> Self {
        Self {
            next_node_id: NodeID::zero(),
            next_edge_id: EdgeID::zero(),
            edges: FxHashMap::default(),
            nodes: FxHashMap::default(),
        }
    }

    /// Returns the new id if a node can be added, `panic`s if the graph
    /// is out of space to add new nodes.
    pub fn add_node(&mut self, node: NodeData) -> NodeID {
        let node_id = self.next_node_id;
        if self.next_node_id == NodeID::max_number() {
            panic!("The storage type for NodeIDs ran out of space.")
        }
        self.next_node_id.plus_one();

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
        let mut new_edges = HashSet::new();
        let mut edge_to_remove = Vec::new();
        for edge in node1_d.containing_edges.drain() {
            let e = self.edges.get_mut(&edge).unwrap();
            e.nodes.remove_node(node1);
            e.nodes.add_node(*node2);
            if e.nodes.len() == 1 {
                edge_to_remove.push(edge);
            }
            new_edges.insert(edge);
        }
        let node2_ref = self.nodes.get_mut(node2).unwrap();
        for e in new_edges.iter() {
            node2_ref.containing_edges.insert(*e);
        }
        for e in edge_to_remove {
            self.remove_edge(e);
        }
        // need to dedup
        let mut duplicate_edges = Vec::new();
        let node2_edges: Vec<_> = self
            .nodes
            .get(node2)
            .unwrap()
            .containing_edges
            .clone()
            .into_iter()
            .collect();
        for ix in 0..node2_edges.len() {
            for jx in (ix + 1)..node2_edges.len() {
                let ix_edge = self.edges.get(&node2_edges[ix]).unwrap();
                let jx_edge = self.edges.get(&node2_edges[jx]).unwrap();
                if ix_edge.nodes == jx_edge.nodes {
                    if new_edges.contains(&node2_edges[ix]) {
                        duplicate_edges.push(node2_edges[ix]);
                    } else {
                        duplicate_edges.push(node2_edges[jx]);
                    }
                }
            }
        }
        for edge in duplicate_edges {
            self.remove_edge(edge);
        }
    }

    /// Creates an edge in the hypergraph, if the edge already exists it will
    /// delete the old data and replace it with the newly provided data.
    /// ### `panic`s
    /// - If all nodes are not present in the hypergraph
    /// - If you create more edges than allowable by the `EdgeID` storage type
    pub fn add_edge(&mut self, edge: impl AsRef<[NodeID]>, data: EdgeData) -> EdgeID {
        let edge_set: EdgeSet<NodeID> = edge.into();
        if let Some(id) = self.find_id(edge_set.node_vec()) {
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
        if self.next_edge_id == EdgeID::max_number() {
            panic!("Ran out of edges, need to use a bigger EdgeID representation.")
        }
        self.next_edge_id.plus_one();

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
        id
    }

    /// Solely for use by KVGraph, which needs to generate Uuids for each entry.
    /// Returns existing `NodeData` if it was there.
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
    pub(crate) fn add_edge_with_id<E>(
        &mut self,
        edge: E,
        data: EdgeData,
        id: EdgeID,
    ) -> Option<EdgeData>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let edge_set: EdgeSet<NodeID> = edge.into();
        if self.find_id(edge_set.node_vec()).is_some() {
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
        self.edges
            .insert(id.clone(), edge)
            .map(|edge_struct| edge_struct.data)
    }

    /// This will remove the node from the graph and any edges containing it.
    /// The node will not be reused in the future. If this leaves an edge
    /// empty the edge will be removed from the graph.
    pub fn remove_node(&mut self, node: NodeID) -> Option<NodeData> {
        if self.nodes.contains_key(&node) == false {
            return None;
        }
        let removed_node = self.nodes.remove(&node).unwrap();
        let mut edges_to_be_removed = Vec::new();
        for effected_edge_id in removed_node.containing_edges.iter() {
            let effected_edge = self
                .edges
                .get_mut(&effected_edge_id)
                .expect("Effected edge not found.");
            effected_edge.nodes.remove_node(&node);
            if effected_edge.nodes.len() == 0 {
                edges_to_be_removed.push(effected_edge_id.clone());
            }
        }
        for edge_id in edges_to_be_removed {
            self.remove_edge(edge_id);
        }
        Some(removed_node.data)
    }

    /// Returns the `EdgeData` of the associated edge if it existed and `None`
    /// if an incorrect edge was provided.
    pub fn remove_edge(&mut self, edge_id: EdgeID) -> Option<EdgeData> {
        if let Some(e) = self.edges.remove(&edge_id) {
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

    /// In case you forget :)
    pub fn find_id(&self, nodes: impl AsRef<[NodeID]>) -> Option<EdgeID> {
        let nodes_ref = nodes.as_ref();
        if nodes_ref.len() == 0 {
            return None;
        }
        let nodes_as_edge: EdgeSet<NodeID> = nodes_ref.into();
        let first = nodes_ref[0];
        if self.nodes.contains_key(&first) == false {
            return None;
        }
        let candidate_ids = self.nodes.get(&first).unwrap();
        for candidate_id in candidate_ids.containing_edges.iter() {
            let candidate = self
                .edges
                .get(candidate_id)
                .expect("Edge invariant violated.");
            // This is where the "no duplicate edges" is enforced, otherwise
            // we will just return the arbitrary first edge that matches
            if candidate.nodes == nodes_as_edge {
                return Some(candidate_id.clone());
            }
        }
        None
    }
}

impl<NodeData, EdgeData, NodeID, EdgeID> HGraph<NodeData, EdgeData, NodeID, EdgeID>
where
    NodeID: HgNode,
    EdgeID: HgNode,
    NodeData: Clone,
    EdgeData: Clone,
{
    /// Returns a new HGraph with the edges that pass the filter, along with all the nodes
    /// needed to support each edge.
    pub fn filter_by_edge<F>(&self, filter: F) -> HGraph<NodeData, EdgeData, NodeID, EdgeID>
    where
        F: Fn(EdgeID) -> bool,
    {
        let new_edges: FxHashMap<EdgeID, Edge<NodeID, EdgeData>> = self
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
        let new_nodes: FxHashMap<NodeID, Node<NodeData, EdgeID>> = nodes_contained_in_edge
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
        let mut next_node_id = *new_nodes.keys().max().unwrap();
        next_node_id.plus_one();
        let mut next_edge_id = *new_edges.keys().max().unwrap();
        next_edge_id.plus_one();
        HGraph {
            next_node_id,
            next_edge_id,
            edges: new_edges,
            nodes: new_nodes,
        }
    }

    pub fn star(&self, nodes: impl AsRef<[NodeID]>) -> HGraph<NodeData, EdgeData, NodeID, EdgeID> {
        let link = self.link_of_nodes(nodes);
        let link_edges: HashSet<EdgeID> = link.iter().map(|(id, _nodes)| *id).collect();
        let filter = |edge_id| link_edges.contains(&edge_id);
        self.filter_by_edge(filter)
    }
}

impl<N, E, NData, EData> HyperGraph for HGraph<NData, EData, N, E>
where
    N: HgNode,
    E: HgNode,
{
    type NodeID = N;
    type EdgeID = E;

    fn query_edge(&self, edge: &Self::EdgeID) -> Option<Vec<Self::NodeID>> {
        self.edges
            .get(edge)
            .map(|big_edge| big_edge.nodes.node_vec())
    }

    fn containing_edges_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
        let nodes_set: EdgeSet<Self::NodeID> = nodes.into();
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

    fn containing_edges(&self, edge: &Self::EdgeID) -> Vec<Self::EdgeID> {
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

    fn link(&self, edge: &Self::EdgeID) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)> {
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

    fn link_of_nodes(
        &self,
        nodes: impl AsRef<[Self::NodeID]>,
    ) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)> {
        let edge: EdgeSet<Self::NodeID> = nodes.into();
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

    fn maximal_edges(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
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

    fn maximal_edges_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
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

    fn edges_of_size(&self, card: usize) -> Vec<Self::EdgeID> {
        self.edges
            .iter()
            .filter(|(_, e)| e.nodes.len() == card)
            .map(|(id, _)| id)
            .cloned()
            .collect()
    }

    fn boundary_up(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
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
                boundary.push(id.clone());
            }
        }
        boundary
    }

    fn boundary_down(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID> {
        if self.edges.contains_key(edge_id) == false {
            return Vec::new();
        }
        let edge_set = &self.edges.get(edge_id).unwrap().nodes;
        let mut boundary: Vec<Self::EdgeID> = Vec::new();
        for ix in 0..edge_set.len() {
            let mut possible = edge_set.node_vec();
            possible.remove(ix);
            if let Some(id) = self.find_id(possible) {
                boundary.push(id);
            }
        }
        boundary
    }

    fn boundary_up_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
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
                boundary.push(id.clone());
            }
        }
        boundary
    }

    fn boundary_down_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID> {
        let edge_set: EdgeSet<Self::NodeID> = nodes.into();
        let mut boundary: Vec<Self::EdgeID> = Vec::new();
        for ix in 0..edge_set.len() {
            let mut possible = edge_set.node_vec();
            possible.remove(ix);
            if let Some(id) = self.find_id(possible) {
                boundary.push(id);
            }
        }
        boundary
    }

    fn skeleton(&self, cardinality: usize) -> Vec<Self::EdgeID> {
        self.edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= cardinality)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl<NodeData, EdgeData, NodeID, EdgeID> HGraph<NodeData, EdgeData, NodeID, EdgeID>
where
    NodeID: HgNode + for<'a> Deserialize<'a>,
    EdgeID: HgNode + for<'a> Deserialize<'a>,
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

impl<NodeData, EdgeData, NodeID, EdgeID> Display for HGraph<NodeData, EdgeData, NodeID, EdgeID>
where
    NodeID: HgNode,
    EdgeID: HgNode,
{
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
    use crate::HyperGraph;

    use super::HGraph;

    #[test]
    fn simple_tasks() {
        let mut g = HGraph::<(), (), u8, u8>::new();

        let nodes: Vec<_> = (0..10).map(|_| g.add_node(())).collect();
        assert_eq!(nodes.len(), 10);
        let e1 = g.add_edge(&[1_u8, 2, 3][..], ());
        let e2 = g.add_edge(vec![1, 2, 4], ());
        g.add_edge([5_u8, 6, 7], ());
        assert!(g.find_id([1_u8, 2, 3]).is_some());
        // is simplex so this should work
        assert!(g.find_id(&[0][..]).is_none());
        let containing_edges = g.containing_edges_of_nodes([1_u8, 2]);
        assert_eq!(containing_edges.len(), 2);
        assert!(containing_edges.contains(&e1));
        assert!(containing_edges.contains(&e2));
        let affected_edges = g.containing_edges_of_nodes([2]);
        g.remove_node(2);
        assert!(affected_edges.contains(&e1));
        assert!(affected_edges.contains(&e2));
        assert!(g.find_id([1_u8, 3]).is_some());
        assert!(g.find_id([1_u8, 2, 3]).is_none());
        let _: Vec<_> = (5..=7).map(|x| g.remove_node(x)).collect();
        assert!(g.find_id([5, 6, 7]).is_none());
    }

    #[test]
    fn link_and_maximal() {
        let mut core = HGraph::<(), (), u8, u8>::new();
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
        let mut expected_link = vec![(e4.clone(), vec![4_u8]), (e5.clone(), vec![4_u8, 5])];
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
        let e2 = hg.add_edge(vec![0, 1, 2], 2);
        let e3 = hg.add_edge(vec![0, 1, 3], 3);
        let e4 = hg.add_edge(vec![0, 1, 2, 3], 4);
        hg.add_edge(vec![1, 2, 5], 19);

        let expected = vec![e2, e3];
        let mut test_1 = hg.boundary_up(&e1);
        test_1.sort();
        assert_eq!(test_1, expected);

        let mut test_2 = hg.boundary_down(&e4);
        test_2.sort();
        assert_eq!(test_2, expected);

        let expected_3 = vec![e4];
        let test_3 = hg.boundary_up_of_nodes(vec![0, 1, 2]);
        assert_eq!(test_3, expected_3);

        let expected_4 = vec![e1];
        let test_4 = hg.boundary_down_of_nodes(vec![0, 1, 3]);
        assert_eq!(test_4, expected_4);
    }
}
