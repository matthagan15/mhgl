use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::traits::HgNode;

use super::{edge_set::EdgeSet, EdgeID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<NodeData> {
    pub containing_edges: HashSet<EdgeID>,
    pub data: NodeData,
}

impl<NodeData> Node<NodeData> {
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
pub struct HGraphCore<NodeID: HgNode, NodeData, EdgeData> {
    next_node_id: NodeID,
    next_edge_id: EdgeID,
    pub edges: HashMap<EdgeID, Edge<NodeID, EdgeData>>,
    pub nodes: HashMap<NodeID, Node<NodeData>>,
    is_simplex: bool,
}

impl<NodeID: HgNode, NodeData, EdgeData> HGraphCore<NodeID, NodeData, EdgeData> {
    pub fn new() -> Self {
        Self {
            next_node_id: NodeID::zero(),
            next_edge_id: 0,
            edges: HashMap::new(),
            nodes: HashMap::new(),
            is_simplex: false,
        }
    }
    pub fn contains_node(&self, node: &NodeID) -> bool {
        self.nodes.contains_key(node)
    }

    pub fn change_node_data(&mut self, node: &NodeID, new_data: NodeData) -> Option<NodeData> {
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
    pub fn borrow_node(&self, node: &NodeID) -> Option<&NodeData> {
        self.nodes.get(node).map(|big_node| &big_node.data)
    }
    pub fn borrow_node_mut(&mut self, node: &NodeID) -> Option<&mut NodeData> {
        self.nodes.get_mut(node).map(|big_node| &mut big_node.data)
    }

    pub fn borrow_edge(&self, edge: &EdgeID) -> Option<&EdgeData> {
        self.edges.get(edge).map(|big_edge| &big_edge.data)
    }
    pub fn borrow_edge_mut(&mut self, edge: &EdgeID) -> Option<&mut EdgeData> {
        self.edges.get_mut(edge).map(|big_edge| &mut big_edge.data)
    }

    pub fn change_edge_data(&mut self, edge_id: &EdgeID, new_data: EdgeData) -> Option<EdgeData> {
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

    /// Returns `None` if the edge is not present. Otherwise retrieves the unique id associated with this edge (no duplicate edges allowed).
    ///
    /// Returns `nil` for empty query
    ///
    pub fn query_id<E>(&self, edge: E) -> Option<EdgeID>
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

    pub fn get_containing_edges_id(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        if let Some(nodes_of_edge) = self.edges.get(edge_id) {
            self.get_containing_edges(nodes_of_edge.nodes.node_vec())
        } else {
            vec![]
        }
    }

    pub fn get_containing_edges_strict_id(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        if let Some(nodes_of_edge) = self.edges.get(edge_id) {
            self.get_containing_edges_strict(nodes_of_edge.nodes.node_vec())
        } else {
            vec![]
        }
    }

    /// what is the right behavior for adding a sub-face of a simplex?
    /// Fails if not every node is contained in the provided edge.
    pub fn add_edge<E>(&mut self, edge: E, data: EdgeData) -> Option<EdgeID>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let edge_set: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
        if self.query_id(edge_set.node_vec()).is_some() {
            return None;
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
        self.edges.insert(id.clone(), edge);
        Some(id)
    }

    /// Returns true if the node was added correctly, false if
    /// the node was not added because it was already present.
    /// Does not overwrite existing nodes.
    pub fn add_node(&mut self, node: NodeData) -> NodeID {
        let node_id = self.next_node_id;
        if self.next_node_id == NodeID::max_number() {
            panic!("Too small of a NodeID representation used, you ran out of nodes.")
        }
        self.next_node_id.plus_one();

        let new_node = Node {
            containing_edges: HashSet::new(),
            data: node,
        };
        self.nodes.insert(node_id, new_node);
        node_id
    }

    /// Adds the provided nodes to the graph with no connective structure.
    /// Returns `None` if all nodes are new and added correctly, otherwise
    /// returns the list of nodes that were already present. Does not overwrite
    /// any existing nodes.
    pub fn add_nodes(&mut self, nodes: Vec<NodeData>) -> Vec<NodeID> {
        nodes.into_iter().map(|data| self.add_node(data)).collect()
    }

    /// Removes the node, returning the edge ids of the
    /// effected edges. Deletes all data associated with
    /// the node atm.
    /// Does **NOTHING** if a node is not
    /// present in the graph.
    /// what to do if:
    /// - an edge is just modified
    /// - an edge is completely removed
    /// - a node is not present.
    pub fn remove_nodes(&mut self, nodes: Vec<NodeID>) -> Vec<NodeData> {
        nodes.into_iter().map(|n| self.remove_node(n)).collect()
    }

    /// Note: keeps potentially empty edges around!
    pub fn remove_node(&mut self, node: NodeID) -> NodeData {
        let removed_node = self.nodes.remove(&node).expect("Node not found.");
        for effected_edge_id in removed_node.containing_edges.iter() {
            let effected_edge = self
                .edges
                .get_mut(&effected_edge_id)
                .expect("Effected edge not found.");
            effected_edge.nodes.remove_node(&node);
        }
        removed_node.data
    }

    /// Returns `true` if the provided edge is supported in
    /// the graph, `false` otherwise. and `true` for provided edges that are **covered** by
    /// another if  the graph is a simplicial complex.
    ///
    /// ```rust
    /// let hg = HGraph::new();
    /// assert!(hg.query(&[]))
    /// ```
    pub fn query<E>(&self, edge: E) -> bool
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let e: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
        if e.is_empty() {
            return true;
        }
        let first = e.get_first_node().unwrap();
        if self.nodes.contains_key(&first) == false {
            return false;
        }
        let candidate_ids = self.nodes.get(&first).unwrap();
        for candidate_id in candidate_ids.containing_edges.iter() {
            let candidate = self
                .edges
                .get(candidate_id)
                .expect("Edge invariant violated.");
            let candidite_is_good = if self.is_simplex {
                candidate.nodes.contains(&e)
            } else {
                candidate.nodes == e
            };
            if candidite_is_good {
                return true;
            }
        }
        false
    }

    /// Returns all edges such that the provided edge is contained
    /// within the returned edge. For example a graph with these edges
    /// [{1, 2, 3}, {1, 2, 3, 4}, {1,2}, {0, 1, 2}] would yield
    /// g.get_containing_edges([1, 2]) = [{1, 2, 3}, {1, 2, 3, 4}, {1,2}]
    /// If you want **strictly** containing edges use `
    /// get_containing_edges_strict`. If the provided edge is empty, or the
    /// edge is not supported on the graph, we return nothing.
    pub fn get_containing_edges<E>(&self, edge: E) -> Vec<EdgeID>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        self.get_containing_edges_internal(edge, false)
    }

    /// Same as `get_containing_edges` but only returns edges that are
    /// strictly supersets of the provided edge.
    pub fn get_containing_edges_strict<E>(&self, edge: E) -> Vec<EdgeID>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        self.get_containing_edges_internal(edge, false)
    }
    fn get_containing_edges_internal<E>(&self, edge: E, strict: bool) -> Vec<EdgeID>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let e: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
        if e.is_empty() {
            return vec![];
        }
        let first = e.get_first_node().unwrap();
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
            if strict {
                if candidate.nodes.contains_strict(&e) {
                    ret.push(candidate_id.clone());
                }
            } else {
                if candidate.nodes.contains(&e) {
                    ret.push(candidate_id.clone());
                }
            }
        }
        ret
    }

    pub fn make_simplex(&mut self) {
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

    pub fn make_undirected(&mut self) {
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

    pub fn remove_edge(&mut self, edge_id: EdgeID) -> Option<Edge<NodeID, EdgeData>> {
        if let Some(e) = self.edges.remove(&edge_id) {
            for node in e.nodes.nodes_ref() {
                let containing_edges = self.nodes.get_mut(node).expect("Why is edge not in here.");
                containing_edges.containing_edges.remove(&edge_id);
            }
            Some(e)
        } else {
            None
        }
    }

    /// If this is a simplex it should only return the link associated with the
    /// maximal id? No, it should return any associated edge_ids and the link within the edge. doesn't matter if its a sub-maximal edge or not. up to the user
    /// to decide.
    /// ```rust
    /// let core = HGraphCore::<u32, (), ()>::new();
    /// let nodes: Vec<_> =(0..=10).into_iter().map(|x| {
    /// (x, ())
    /// }).collect();
    /// core.add_nodes(nodes);
    ///
    /// ```
    pub fn link<E>(&self, edge: E) -> Vec<(EdgeID, EdgeSet<NodeID>)>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let e: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
        let containing_edges = self.get_containing_edges(e.node_vec());
        containing_edges
            .into_iter()
            .filter_map(|id| {
                if let Some(local_link) = self
                    .edges
                    .get(&id)
                    .expect("Broken edge invariant found in link.")
                    .nodes
                    .link(&e)
                {
                    Some((id, local_link))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn maximal_containing_edges<E>(&self, edge: E) -> Vec<EdgeID>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let e: EdgeSet<NodeID> = if self.is_simplex {
            edge.into().to_simplex()
        } else {
            edge.into()
        };
        let containing_edges = self.get_containing_edges(e.node_vec());
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

    /// Returns the edges containing the provided node set that are
    /// of the max size viewable by the edge set.
    pub fn maximal_size_containing_edges<E>(&self, edge: E) -> Option<Vec<EdgeID>>
    where
        E: Into<EdgeSet<NodeID>>,
    {
        let containing_edges = self.get_containing_edges(edge);
        if containing_edges.is_empty() {
            return None;
        }
        let mut edges_with_len: Vec<_> = containing_edges
            .into_iter()
            .map(|id| (id, self.edges.get(&id).unwrap().nodes.len()))
            .collect();
        edges_with_len.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
        let max_size = edges_with_len[0].1;
        Some(
            edges_with_len
                .into_iter()
                .filter_map(|(id, size)| if size == max_size { Some(id) } else { None })
                .collect(),
        )
    }

    pub fn maximal_size_containing_edges_id(&self, edge_id: &EdgeID) -> Option<Vec<EdgeID>> {
        if let Some(edge) = self.edges.get(edge_id) {
            self.maximal_size_containing_edges(edge.nodes.node_vec())
        } else {
            None
        }
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

impl<NodeID, NodeData, EdgeData> HGraphCore<NodeID, NodeData, EdgeData>
where
    NodeID: HgNode + for<'a> Deserialize<'a>,
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

    use super::HGraphCore;

    #[test]
    fn test_simple_tasks() {
        let mut g = HGraphCore::<u8, (), ()>::new();
        g.make_simplex();
        let nodes = g.add_nodes((0..10).map(|_| ()).collect());
        assert_eq!(nodes.len(), 9);
        let e1 = g.add_edge(&[1_u8, 2, 3][..], ()).unwrap();
        let e2 = g.add_edge(vec![1, 2, 4], ()).unwrap();
        let e3 = g.add_edge([5_u8, 6, 7], ()).unwrap();
        assert!(g.query([1_u8, 2, 3]));
        // is simplex so this should work
        assert!(g.query([1_u8, 2]));
        assert!(!g.query(&[0][..]));
        let containing_edges = g.get_containing_edges([1_u8, 2]);
        assert_eq!(containing_edges.len(), 2);
        assert!(containing_edges.contains(&e1));
        assert!(containing_edges.contains(&e2));
        let affected_edges = g.get_containing_edges([2]);
        g.remove_node(2);
        assert!(affected_edges.contains(&e1));
        assert!(affected_edges.contains(&e2));
        assert!(g.query([1_u8, 3]));
        assert!(!g.query([1_u8, 2, 3]));
        g.remove_nodes(vec![5, 6, 7]);
        assert!(!g.query([5, 6, 7]));
    }

    #[test]
    fn test_link_and_star() {
        let mut core = HGraphCore::<u8, (), ()>::new();
        for ix in 0..10 {
            core.add_node(());
        }
        let e1 = core.add_edge(vec![0, 1], ()).unwrap();
        let e2 = core.add_edge(vec![0, 2], ()).unwrap();
        let e3 = core.add_edge(vec![0, 3], ()).unwrap();
        let e4 = core.add_edge(vec![0, 1, 4], ()).unwrap();
        let e5 = core.add_edge(vec![0, 1, 4, 5], ()).unwrap();
        let e6 = core.add_edge(vec![0, 2, 6], ()).unwrap();
        let containers = core.get_containing_edges([0]);
        let mut maximal_edges = core.maximal_containing_edges([0]);
        maximal_edges.sort();
        let mut expected = vec![e3, e5, e6];
        expected.sort();
        assert_eq!(maximal_edges, expected);

        let mut link = core.link([0, 1]);
        link.sort();
        let mut expected_link = vec![
            (e4.clone(), EdgeSet::from([4_u8])),
            (e5.clone(), EdgeSet::from([4_u8, 5])),
        ];
        expected_link.sort();
        assert_eq!(link, expected_link);

        assert_eq!(core.maximal_size_containing_edges([0]), Some(vec![e5]));
    }
}
