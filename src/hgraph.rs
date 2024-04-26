use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::traits::HgNode;
use crate::EdgeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<NodeData, EdgeID: HgNode> {
    pub containing_edges: HashSet<EdgeID>,
    pub data: NodeData,
}

impl<NodeData, EdgeID: HgNode> Node<NodeData, EdgeID> {
    ///
    /// ```rust
    /// let n = Node::new();
    /// ```
    pub fn new(data: NodeData) -> Self {
        Node {
            containing_edges: HashSet::new(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge<N: HgNode, EdgeData> {
    pub nodes: EdgeSet<N>,
    pub data: EdgeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defaults to undirected hypergraph, user has to declare it a simplex.
pub struct HGraph<NodeData, EdgeData, NodeID: HgNode = u32, EdgeID: HgNode = u64> {
    next_node_id: NodeID,
    next_edge_id: EdgeID,
    pub edges: HashMap<EdgeID, Edge<NodeID, EdgeData>>,
    pub nodes: HashMap<NodeID, Node<NodeData, EdgeID>>,
    is_simplex: bool,
}

impl<NodeData, EdgeData, NodeID: HgNode, EdgeID: HgNode>
    HGraph<NodeData, EdgeData, NodeID, EdgeID>
{
    pub fn new() -> Self {
        Self {
            next_node_id: NodeID::zero(),
            next_edge_id: EdgeID::zero(),
            edges: HashMap::new(),
            nodes: HashMap::new(),
            is_simplex: false,
        }
    }

    /// Does not allow for duplicate edges. Returns an error if all nodes in the
    /// provided edge are not in the graph. Will `panic!` if you run out of
    /// EdgeIDs to  use.
    pub fn add_edge<E>(&mut self, edge: E, data: EdgeData) -> Result<EdgeID, EdgeData>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let edge_set: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
        if self.find_id(edge_set.node_vec()).is_some() {
            return Err(data);
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
                return Err(data);
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
        Ok(id)
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
        let edge_set: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
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

    /// Returns the new id if a node can be added, returns `None` if the graph
    /// is out of space to add new nodes.
    pub fn add_node(&mut self, node: NodeData) -> Option<NodeID> {
        let node_id = self.next_node_id;
        if self.next_node_id == NodeID::max_number() {
            return None;
        }
        self.next_node_id.plus_one();

        let new_node = Node {
            containing_edges: HashSet::new(),
            data: node,
        };
        let insert = self.nodes.insert(node_id, new_node);
        if insert.is_some() {
            panic!("For some reason we encountered the same node_id twice.")
        }
        Some(node_id)
    }

    /// Solely for use by KVGraph, which needs to generate Uuids for each entry.
    /// Returns existing `NodeData` if it was there.
    pub(crate) fn add_node_with_id(&mut self, node: NodeData, id: NodeID) -> Option<NodeData> {
        let new_node = Node {
            containing_edges: HashSet::new(),
            data: node,
        };
        self.nodes
            .insert(id, new_node)
            .map(|old_node| old_node.data)
    }

    /// This will remove the node from the graph and any edges containing it.
    /// The node will not be reused in the future. Also if this leaves an edge
    /// empty the edge will be removed from the graph.
    pub fn remove_node(&mut self, node: NodeID) -> NodeData {
        let removed_node = self.nodes.remove(&node).expect("Node not found.");
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
        removed_node.data
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
    pub fn borrow_node(&self, node: &NodeID) -> Option<&NodeData> {
        self.nodes.get(node).map(|big_node| &big_node.data)
    }

    /// Borrows the data mutably of the provided node.
    pub fn borrow_node_mut(&mut self, node: &NodeID) -> Option<&mut NodeData> {
        self.nodes.get_mut(node).map(|big_node| &mut big_node.data)
    }

    /// Borrows the data of the provided edge.
    pub fn borrow_edge(&self, edge: &EdgeID) -> Option<&EdgeData> {
        self.edges.get(edge).map(|big_edge| &big_edge.data)
    }

    /// Borrows the data mutably of the provided edge.
    pub fn borrow_edge_mut(&mut self, edge: &EdgeID) -> Option<&mut EdgeData> {
        self.edges.get_mut(edge).map(|big_edge| &mut big_edge.data)
    }

    /// In case you forget :)
    pub fn find_id<E>(&self, edge: E) -> Option<EdgeID>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let e: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
        if e.is_empty() {
            return None;
        }
        let first = e.get_first_node().unwrap();
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
            if candidate.nodes == e {
                return Some(candidate_id.clone());
            }
        }
        None
    }

    /// Finds all edges that contain all of the provided input nodes. Note that if the nodes match an existing edge then that edge will be in the output `Vec`.
    /// ```rust
    /// use mhgl::HGraph;
    /// let mut hg = HGraph<u8, u8>::new();
    /// let data = vec![1_u8, 20, 13, 4];
    /// let nodes = data.into_iter().filter_map(|x| hg.add_node(x)).collect();
    ///
    /// let e1 = hg.add_edge(&nodes[0..=1]);
    /// let e2 = hg.add_edge(&nodes[0..=2]);
    /// let e3 = hg.add_edge(&nodes[0..=3]);
    ///
    /// let test_1 = hg.edges_containing_nodes([0, 1, 2]);
    /// let test_2 = hg.edges_containing_nodes([10]);
    /// assert_eq!(test_1, vec![e1, e2]);
    /// assert_eq!(test_2, vec![]);
    /// ```
    pub fn edges_containing_nodes<N>(&self, nodes: N) -> Vec<EdgeID>
    where
        N: AsRef<[NodeID]>,
    {
        let nodes_set: EdgeSet<NodeID> = nodes.into();
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

    /// Finds all edges that contain the provided input edge. As duplicate
    /// edges are not allowed this only returns edges that strictly contain the
    /// given edge. Note that an input edge that is maximal, meaning it has no edges containing it, will yield an empty `Vec`.
    pub fn edges_containing_edge(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        if self.edges.contains_key(edge_id) == false {
            return Vec::new();
        }
        let edge = self.edges.get(edge_id).unwrap();
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

    /// Under construction for planned simplicial complex.
    fn make_simplex(&mut self) {
        if self.is_simplex {
            return;
        }
        let new_edges = self
            .edges
            .drain()
            .map(|(id, e)| {
                let new_edge = Edge {
                    nodes: e.nodes.to_simplex(),
                    data: e.data,
                };
                (id, new_edge)
            })
            .collect();
        self.edges = new_edges;
        self.is_simplex = true;
    }

    /// Under construction for planned simplicial complex.
    fn make_undirected(&mut self) {
        if !self.is_simplex {
            return;
        }
        let new_edges = self
            .edges
            .drain()
            .map(|(id, e)| {
                let new_edge = Edge {
                    nodes: e.nodes.make_undirected(),
                    data: e.data,
                };
                (id, new_edge)
            })
            .collect();
        self.edges = new_edges;
        self.is_simplex = false;
    }

    pub fn remove_edge(&mut self, edge_id: EdgeID) -> Option<EdgeData> {
        if let Some(e) = self.edges.remove(&edge_id) {
            for node in e.nodes.nodes_ref() {
                let containing_edges = self.nodes.get_mut(node).expect("Why is edge not in here.");
                containing_edges.containing_edges.remove(&edge_id);
            }
            Some(e.data)
        } else {
            None
        }
    }

    pub fn link(&self, edge_id: &EdgeID) -> Vec<(EdgeID, Vec<NodeID>)> {
        if self.edges.contains_key(edge_id) == false {
            return Vec::new();
        }
        let containing_edges = self.edges_containing_edge(edge_id);
        let edge = self.edges.get(edge_id).unwrap();
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

    pub fn link_of_nodes<E>(&self, nodes: E) -> Vec<(EdgeID, Vec<NodeID>)>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let edge: EdgeSet<NodeID> = nodes.into();
        let containing_edges = self.edges_containing_nodes(edge.node_vec());
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

    pub fn maximal_edges_containing_edge(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        let containing_edges = self.edges_containing_edge(edge_id);
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

    pub fn maximal_edges_containing_nodes<E>(&self, nodes: E) -> Vec<EdgeID>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        todo!()
    }

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    pub fn edges_of_size(&self, card: usize) -> Vec<EdgeID> {
        self.edges
            .iter()
            .filter(|(id, e)| e.nodes.len() == card)
            .map(|(id, e)| id)
            .cloned()
            .collect()
    }

    /// Returns the set of edge of size less than or equal to `k`,
    /// inclusive. Also note that `k` refers to the cardinality of the
    /// provided sets, not the dimension (An edge {1, 2} would be included in a k-skeleton with k >= 2.)
    pub fn k_skeleton(&self, k: usize) -> HashSet<EdgeID> {
        self.edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= k)
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
    pub fn to_disk(&self, path: &Path) {
        let s = serde_json::to_string(self).expect("could not serialize NEGraph");
        let mut file = File::create(path).expect("Cannot create File.");
        file.write_all(s.as_bytes()).expect("Cannot write");
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        if path.is_file() == false {
            return None;
        }
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let out = serde_json::from_reader(reader);
            if out.is_ok() {
                Some(out.unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}

mod tests {
    use crate::EdgeSet;

    use super::HGraph;

    #[test]
    fn test_simple_tasks() {
        let mut g = HGraph::<(), (), u8, u8>::new();
        g.make_simplex();

        let nodes: Vec<_> = (0..10).map(|_| g.add_node(())).collect();
        assert_eq!(nodes.len(), 9);
        let e1 = g.add_edge(&[1_u8, 2, 3][..], ()).unwrap();
        let e2 = g.add_edge(vec![1, 2, 4], ()).unwrap();
        let e3 = g.add_edge([5_u8, 6, 7], ()).unwrap();
        assert!(g.find_id([1_u8, 2, 3]).is_some());
        // is simplex so this should work
        assert!(g.find_id([1_u8, 2]).is_some());
        assert!(g.find_id(&[0][..]).is_none());
        let containing_edges = g.edges_containing_nodes([1_u8, 2]);
        assert_eq!(containing_edges.len(), 2);
        assert!(containing_edges.contains(&e1));
        assert!(containing_edges.contains(&e2));
        let affected_edges = g.edges_containing_nodes([2]);
        g.remove_node(2);
        assert!(affected_edges.contains(&e1));
        assert!(affected_edges.contains(&e2));
        assert!(g.find_id([1_u8, 3]).is_some());
        assert!(g.find_id([1_u8, 2, 3]).is_none());
        let _: Vec<_> = (5..=7).map(|x| g.remove_node(x)).collect();
        assert!(g.find_id([5, 6, 7]).is_none());
    }

    #[test]
    fn test_link_and_star() {
        let mut core = HGraph::<(), (), u8, u8>::new();
        for ix in 0..10 {
            core.add_node(());
        }
        let e1 = core.add_edge(vec![0, 1], ()).unwrap();
        let e2 = core.add_edge(vec![0, 2], ()).unwrap();
        let e3 = core.add_edge(vec![0, 3], ()).unwrap();
        let e4 = core.add_edge(vec![0, 1, 4], ()).unwrap();
        let e5 = core.add_edge(vec![0, 1, 4, 5], ()).unwrap();
        let e6 = core.add_edge(vec![0, 2, 6], ()).unwrap();
        let containers = core.edges_containing_nodes([0]);
        let mut maximal_edges = core.maximal_edges_containing_nodes([0]);
        maximal_edges.sort();
        let mut expected = vec![e3, e5, e6];
        expected.sort();
        assert_eq!(maximal_edges, expected);

        let mut link = core.link_of_nodes([0, 1]);
        link.sort();
        let mut expected_link = vec![(e4.clone(), vec![4_u8]), (e5.clone(), vec![4_u8, 5])];
        expected_link.sort();
        assert_eq!(link, expected_link);
    }
}
