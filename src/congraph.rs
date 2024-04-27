use std::collections::HashSet;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::HGraph;

use crate::{EdgeSet, HgNode};

type EdgeID = u64;

/// A connectivity only hypergraph object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConGraph {
    core: HGraph<(), ()>,
}

impl ConGraph {
    pub fn new() -> ConGraph {
        ConGraph {
            core: HGraph::new(),
        }
    }

    pub fn to_disk(&self, path: &Path) {
        let s = self.to_string();
        let mut file = File::create(path).expect("Cannot create File.");
        file.write_all(s.as_bytes()).expect("Cannot write");
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        if path.is_file() == false {
            return None;
        }
        if let Ok(hg_string) = fs::read_to_string(path) {
            if let Ok(serde_out) = ConGraph::from_str(&hg_string) {
                Some(serde_out)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Panics if new node cannot be added.
    pub fn add_node(&mut self) -> u32 {
        self.core.add_node(()).unwrap()
    }

    /// Adds `num_nodes` nodes to the graph, returning a vector containing
    /// the nodes created. The number of nodes returned may be less than
    /// the number of nodes requested due to the use of u32 to store nodes.
    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<u32> {
        (0..num_nodes)
            .map(|_| self.core.add_node(()).unwrap())
            .collect()
    }

    /// Note this will delete any edges that are empty, but singleton edges are
    /// allowed. For example, if I have the edge {1, 2} and then I remove 2 from the graph I will still have the edge {1} so that way further nodes can
    /// be added back to the edge but if I then remove 1 I will also delete the
    /// edge.
    pub fn remove_node(&mut self, node: u32) {
        self.core.remove_node(node);
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: Vec<u32>) {
        nodes
            .into_iter()
            .map(|node| self.core.remove_node(node))
            .collect()
    }

    pub fn nodes(&self) -> Vec<u32> {
        self.core.nodes.keys().cloned().collect()
    }

    /// Creates an undirected edge among the given nodes. Duplicate inputs are removed. Does not allow for duplicate edges at the moment.
    pub fn add_edge<E>(&mut self, nodes: E) -> EdgeID
    where
        E: AsRef<[u32]>,
    {
        let id = self.core.add_edge(nodes, ());
        id.expect("Graph did not return a valid ID.")
    }

    pub fn remove_edge(&mut self, edge_id: EdgeID) {
        self.core.remove_edge(edge_id);
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn query_edge_id(&self, edge_id: &EdgeID) -> Option<Vec<u32>> {
        self.core.edges.get(edge_id).map(|e| e.nodes.node_vec())
    }

    /// In case you forget it.
    pub fn find_id<E>(&self, nodes: E) -> Option<EdgeID>
    where
        E: AsRef<[u32]>,
    {
        self.core.find_id(nodes.as_ref())
    }

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    pub fn edges_of_size(&self, card: usize) -> Vec<EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(id, e)| e.nodes.len() == card)
            .map(|(id, e)| id)
            .cloned()
            .collect()
    }

    /// Returns all edges that strictly contain the provided nodes AND are not contained in another edge.
    pub fn maximal_containing_edges<E>(&self, nodes: E) -> Vec<EdgeID>
    where
        E: AsRef<[u32]>,
    {
        self.core.maximal_edges_containing_nodes(nodes)
    }

    /// Returns IDs of all edges that strictly contain the provided nodes.
    pub fn edges_containing_edge(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        self.core.edges_containing_edge(edge_id)
    }

    pub fn edges(&self) -> Vec<EdgeID> {
        self.core.edges.keys().cloned().collect()
    }

    /// Computes the number of edges that have one vertex in the
    /// provided `cut_nodes` and one in the remaining set. For example,
    /// an edge with only support on the `cut_nodes` would not count. Neither
    /// would an edge without any nodes in `cut_nodes`.
    /// The type `ToSet` is any collection that can be converted to a sparse
    /// set representation.
    pub fn cut<E>(&self, cut_nodes: E) -> usize
    where
        E: Into<EdgeSet<u32>>,
    {
        let cut_as_edge: EdgeSet<u32> = cut_nodes.into();
        let mut counted_edges: HashSet<EdgeID> = HashSet::new();
        for node in cut_as_edge.0.iter() {
            let out_edges: Vec<EdgeID> = self
                .core
                .edges_containing_nodes([*node])
                .into_iter()
                .filter(|e_id| counted_edges.contains(e_id) == false)
                .collect();
            for edge_id in out_edges {
                if let Some(e) = self.core.edges.get(&edge_id) {
                    let intersection = cut_as_edge.intersection(&e.nodes);
                    if intersection.len() > 0 && intersection.len() < e.nodes.len() {
                        counted_edges.insert(edge_id);
                    }
                }
            }
        }
        counted_edges.len()
    }

    /// Computes the link of the provided set. The link of a single
    /// hyperedge is computed using the complement, so a hyperedge
    /// of nodes {a, b, c, d} and a provided `face` of {a, b} would
    /// yield a link of {c, d}. The link of the graph is then the
    /// union of all the links of each hyperedge.
    pub fn link<E>(&self, nodes: E) -> Vec<(u64, Vec<u32>)>
    where
        E: AsRef<[u32]>,
    {
        self.core.link_of_nodes(nodes).into_iter().collect()
    }

    /// Returns the edges that have cardinality less than or equal to the input `cardinality`.
    pub fn skeleton(&self, cardinality: usize) -> Vec<EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= cardinality)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Display for ConGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.core.nodes.len() == 0 {
            println!("Graph is empty. Add nodes for more fun.");
            return Ok(());
        }
        let mut s = String::new();
        s.push_str("nodes:\n[");
        let x: Vec<String> = self.core.nodes.keys().map(|n| n.to_string()).collect();
        for ix in 0..x.len() - 1 {
            s.push_str(&x[ix]);
            s.push_str(", ");
        }
        s.push_str(x.last().unwrap());
        s.push_str("]\n");
        s.push_str("edges:\n");
        for e in self.core.edges.values() {
            s.push_str(&e.nodes.to_string());
            s.push_str("\n");
        }
        f.write_str(&s)
    }
}

impl FromStr for ConGraph {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Convert this to regex
        let lines: Vec<&str> = s.lines().collect();
        let mut nodes_ix = 0;
        let mut edges_start_ix = 0;
        for ix in 0..lines.len() {
            if lines[ix] == "nodes:" {
                nodes_ix = ix + 1;
            }
            if lines[ix] == "edges:" {
                edges_start_ix = ix + 1;
            }
        }
        let mut node_string = lines[nodes_ix].to_string();
        if node_string.starts_with('[') {
            node_string.remove(0);
        }
        if node_string.ends_with(']') {
            node_string.pop();
        }
        let mut nodes = HashSet::new();
        for node in node_string.split(',') {
            nodes.insert(node.trim().parse::<u32>().expect("node parse error."));
        }
        let mut edges = Vec::new();
        for edge_ix in edges_start_ix..lines.len() {
            let mut edge_string = lines[edge_ix].to_string();
            if edge_string.starts_with('{') || edge_string.starts_with('[') {
                edge_string.remove(0);
            }
            if edge_string.ends_with('}') || edge_string.ends_with(']') {
                edge_string.pop();
            }
            let mut node_set = Vec::new();
            for node_str in edge_string.split(',') {
                println!("node_str: {:}", node_str);
                node_set.push(node_str.trim().parse::<u32>().expect("node parse error."));
            }
            edges.push(node_set);
        }
        let max_seen_node = nodes.iter().fold(0_u32, |acc, e| acc.max(*e)) + 1;
        let mut core = HGraph::<(), (), u32, u64>::new();
        let _: Vec<_> = (0..=max_seen_node).map(|_| core.add_node(())).collect();
        for ix in 0..=max_seen_node {
            if nodes.contains(&ix) == false {
                core.remove_node(ix);
            }
        }
        for edge in edges.into_iter() {
            core.add_edge(edge, ())
                .expect("All nodes should be present while deserializing.");
        }
        Ok(ConGraph { core })
    }
}

mod test {

    use std::{collections::HashSet, path::Path, str::FromStr};

    use crate::{congraph::ConGraph, EdgeSet};

    #[test]
    fn test_creating_and_deleting_nodes() {
        let mut hg = ConGraph::new();
        let first_100 = hg.add_nodes(100);
        assert_eq!(first_100, (0_u32..100_u32).collect::<Vec<u32>>());
        let removed = 99_u32;
        hg.remove_node(removed);
        let one_hundred = hg.add_nodes(1);
        assert_eq!(one_hundred[0], 100_u32);
    }

    #[test]
    fn test_edge_creation_removal() {
        let mut hg = ConGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..5]);
        let e1 = hg.add_edge(&nodes[0..6]);
        hg.remove_edge(e1);
        assert!(hg
            .find_id(&[nodes[4], nodes[3], nodes[2], nodes[1], nodes[0]])
            .is_some());
        assert!(hg.find_id(&nodes[0..6]).is_none());
    }

    #[test]
    fn serialization() {
        let mut hg = ConGraph::new();
        hg.add_nodes(10);
        hg.add_edge(&[0, 1]);
        hg.add_edge(&[0, 1, 3, 5]);
        println!("hg:\n{:}", hg);
        let s = hg.to_string();
        let hg_parsed = ConGraph::from_str(&s).expect("no parsing?");
        println!("hg_parsed:\n{:}", hg_parsed);
        dbg!(&hg.core);
        let s3 = serde_json::to_string(&hg).expect("could not serialize next_usable_node");

        dbg!(s3);
    }

    #[test]
    fn link_and_skeleton() {
        let mut hg = ConGraph::new();
        let nodes = hg.add_nodes(10);
        let mut edges = Vec::new();
        for d in 1..=9 {
            edges.push(hg.add_edge(&nodes[0..=d]));
        }
        let mut small_skeleton: Vec<_> = hg.skeleton(4).into_iter().collect();
        small_skeleton.sort();
        let mut expected_skeleton = vec![edges[0], edges[1], edges[2]];
        expected_skeleton.sort();
        assert_eq!(small_skeleton, expected_skeleton);
        let mut small_link = hg.link(&nodes[0..=6]);
        small_link.sort_by(|a, b| a.1.len().partial_cmp(&b.1.len()).unwrap());
        let expected_link = vec![
            EdgeSet::from(&nodes[7..=7]),
            EdgeSet::from(&nodes[7..=8]),
            EdgeSet::from(&nodes[7..=9]),
        ];
        dbg!(small_link);
    }

    #[test]
    fn test_skeleton() {
        let mut hg = ConGraph::new();
        let nodes = hg.add_nodes(10);
        for size in 0..8 {
            hg.add_edge(&nodes[0..=size]);
        }
        for size in 1..10 {
            println!("{:}-skeleton", size);
            println!("{:?}", hg.skeleton(size));
        }
    }

    #[test]
    fn test_cut_with_traits() {
        let mut hg = ConGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[..2]);
        hg.add_edge(&nodes[..3]);
        hg.add_edge(&nodes[..4]);
        println!("hg\n{:}", hg);
        assert_eq!(hg.cut(&nodes[..2]), 2);
        assert_eq!(hg.cut(&nodes[..3]), 1);
        assert_eq!(hg.cut(&nodes[..4]), 0);
    }

    #[test]
    fn test_node_as_edge() {
        let mut hg = ConGraph::new();
        let nodes = hg.add_nodes(3);
        let e0 = hg.add_edge(&[0]);
        let e1 = hg.add_edge(&[0, 1]);
        let e2 = hg.add_edge(&[0, 1, 2]);
        let star = hg.edges_containing_edge(&e0);
        dbg!(star);
        dbg!(&hg);
    }
}
