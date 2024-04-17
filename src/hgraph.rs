use std::collections::{HashSet, VecDeque};
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::structs::{EdgeID, HGraphCore};

use crate::{traits::*, EdgeSet};

/// The simplest to use hypergraph structure. An Undirected and unweighted variant
/// that utilizes u32's for nodes. The directed variant of `HGraph` is
/// `DGraph`. For smaller memory footprints, use
/// `UGraph<N>` for undirected graphs or `PGraph<N>` for directed variants.
/// Duplicate edges are disallowed.
/// ## Example Usage
/// ```
/// let hg = HGraph::new();
/// let nodes = hg.create_nodes(10);
/// hg.create_directed_edge(&nodes[0..3], &nodes[0..=1], 1.2);
/// assert_eq!(hg.step(&nodes[0..3]), vec![(HashSet::from(&nodes[0..=1]), 1.2)]);
/// ```
///
/// Currently do not support labeling nodes.
/// Here is how to store labeled data
/// ```
/// let mut hg = HGraph::new();
/// let mut hm: HashMap<NodeID, NodeType> = HashMap::new();
/// let node_data: Vec<NodeType> = data_set.load();
/// let node_ids: Vec<NodeID> = HGraph::add_nodes(node_data.len());
/// for ix in node_data.into_iter() {
///     hm.insert(node_ids[ix], node_data[ix])
/// }
/// ```
/// Then data can be accessed by querying `hm[id]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HGraph {
    core: HGraphCore<u32, (), ()>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            core: HGraphCore::new(),
        }
    }

    /// Gives the number of edges containing the provided node, where
    /// each edge counts equally regardless of it's cardinality.
    pub fn degree(&self, node: u32) -> usize {
        self.get_containing_edges(&[node]).len()
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
            if let Ok(serde_out) = HGraph::from_str(&hg_string) {
                Some(serde_out)
            } else {
                None
            }
        } else {
            None
        }
    }

    // TODO: Need to overhaul the add_nodes api to panic if new nodes
    // cannot be added. I also do not like the idea of reusing nodes.
    pub fn add_node(&mut self) -> u32 {
        self.core.add_node(())
    }

    /// Adds `num_nodes` nodes to the graph, returning a vector containing
    /// the nodes created. The number of nodes returned may be less than
    /// the number of nodes requested due to the use of u32 to store nodes.
    /// Nodes that get deleted are reused in a First In First Out (FIFO) format.
    // TODO: This should panic if it cannot offer the right amount of nodes.
    // Or return a Ret<Ok, Err> type. That would be the best option.
    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<u32> {
        self.core.add_nodes((0..num_nodes).map(|_| ()).collect())
    }

    /// Removes a node from the node set. The deleted node will be added to a
    /// dequeue to be reused later once all possible nodes have been created.
    pub fn remove_node(&mut self, node: u32) {
        self.core.remove_node(node);
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: Vec<u32>) {
        self.core.remove_nodes(nodes);
    }

    pub fn nodes(&self) -> Vec<u32> {
        self.core.nodes.keys().cloned().collect()
    }

    /// Creates an undirected edge among the given nodes. Duplicate inputs are removed. Does not allow for duplicate edges at the moment.
    pub fn add_edge(&mut self, nodes: &[u32]) -> EdgeID {
        let id = self.core.add_edge(nodes, ());
        id.expect("Graph busted")
    }

    pub fn remove_edge(&mut self, nodes: &[u32]) {
        let e = self.core.query_id(nodes);
        if let Some(id) = e {
            self.core.remove_edge(id);
        }
    }

    pub fn remove_edge_id(&mut self, edge_id: EdgeID) {
        self.core.remove_edge(edge_id);
    }

    /// Returns true if the provided nodes form an existing edge in
    /// the graph, false if they do not.
    pub fn query_edge(&self, nodes: &[u32]) -> bool {
        self.core.query(nodes)
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn query_edge_id(&self, edge_id: &EdgeID) -> Option<Vec<u32>> {
        self.core.edges.get(edge_id).map(|e| e.nodes.node_vec())
    }

    pub fn get_edge_id(&self, nodes: &[u32]) -> Option<EdgeID> {
        self.core.query_id(nodes)
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

    pub fn get_containing_edges(&self, nodes: &[u32]) -> Vec<EdgeID> {
        self.core
            .get_containing_edges_strict(nodes)
            .into_iter()
            .collect()
    }

    /// Returns the hyperedges that contain the provided edge, not
    /// including the provided edge.
    /// Ex: Edges = [{a, b, c}, {a,b,c,d}, {a,b}, {a,b,c,d,e}]
    /// star({a,b,c}) = [{a,b,c,d}, {a,b,c,d,e}]
    pub fn get_containing_edges_id(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        self.core
            .get_containing_edges_strict_id(edge_id)
            .into_iter()
            .collect()
    }

    /// Returns a list of all edges in the graph.
    pub fn get_edges(&self) -> Vec<EdgeID> {
        self.core.edges.keys().cloned().collect()
    }

    /// Computes the number of edges that have one vertex in the
    /// provided `cut_nodes` and one in the remaining set. For example,
    /// an edge with only support on the `cut_nodes` would not count. Neither
    /// would an edge without any nodes in `cut_nodes`.
    /// The type `ToSet` is any collection that can be converted to a sparse
    /// set representation.
    ///
    /// Example
    /// ```
    /// let mut hg = HGraph::new();
    /// let nodes = hg.add_nodes(10);
    /// hg.create_edge(&nodes[..2]);
    /// hg.create_edge(&nodes[..3]);
    /// hg.create_edge(&nodes[..4]);
    /// assert_eq!(hg.cut(&nodes[..2]), 2);
    /// assert_eq!(hg.cut(&nodes[..3]), 1);
    /// assert_eq!(hg.cut(&nodes[..4]), 0);
    /// ```
    pub fn cut(&self, cut_nodes: &[u32]) -> usize {
        let cut_as_edge = EdgeSet::from(cut_nodes.clone());
        let mut counted_edges: HashSet<EdgeID> = HashSet::new();
        for node in cut_nodes {
            let out_edges: Vec<EdgeID> = self
                .core
                .get_containing_edges([*node])
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
    pub fn link<E>(&self, nodes: E) -> Vec<EdgeSet<u32>>
    where
        E: Into<EdgeSet<u32>>,
    {
        self.core
            .link(nodes)
            .into_iter()
            .map(|(_, edge)| edge)
            .collect()
    }

    /// Returns the set of edge of size less than or equal to `k`,
    /// inclusive. Also note that `k` refers to the cardinality of the
    /// provided sets, not the dimension.
    pub fn k_skeleton(&self, k: usize) -> HashSet<EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= k)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

// impl std::ops::Index<dyn Into<SparseBasis<u32>>> for HGraph {
//   type Output = bool;
//
//   fn index(&self, index: dyn Into<SparseBasis<u32>>) -> &Self::Output {
//     todo!()
// }
// }

impl Display for HGraph {
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

impl FromStr for HGraph {
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
            if edge_string.starts_with('{') {
                edge_string.remove(0);
            }
            if edge_string.ends_with('}') {
                edge_string.pop();
            }
            let mut node_set = Vec::new();
            for node_str in edge_string.split(',') {
                node_set.push(node_str.trim().parse::<u32>().expect("node parse error."));
            }
            edges.push(node_set);
        }
        let max_seen_node = nodes.iter().fold(0_u32, |acc, e| acc.max(*e)) + 1;
        let mut core = HGraphCore::<u32, (), ()>::new();
        core.add_nodes((0..=max_seen_node).map(|_| ()).collect());
        for ix in 0..=max_seen_node {
            if nodes.contains(&ix) == false {
                core.remove_node(ix);
            }
        }
        for edge in edges.into_iter() {
            core.add_edge(edge, ());
        }
        Ok(HGraph { core })
    }
}

mod test {

    use std::{collections::HashSet, path::Path, str::FromStr};

    use crate::HGraph;

    #[test]
    fn test_creating_and_deleting_nodes() {
        let mut hg = HGraph::new();
        let first_100 = hg.add_nodes(100);
        assert_eq!(first_100, (0_u32..100_u32).collect::<Vec<u32>>());
        let removed = 99_u32;
        hg.remove_node(removed);
        let one_hundred = hg.add_nodes(1);
        assert_eq!(one_hundred[0], 100_u32);
    }

    #[test]
    fn test_edge_creation_removal() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..5]);
        hg.add_edge(&nodes[0..6]);
        hg.remove_edge(&nodes[0..5]);
        assert!(hg.query_edge(&[nodes[4], nodes[3], nodes[2], nodes[1], nodes[0]]) == false);
        assert!(hg.query_edge(&nodes[0..6]))
    }

    #[test]
    fn test_serialization() {
        let mut hg = HGraph::new();
        hg.add_nodes(10);
        hg.add_edge(&[0, 1]);
        hg.add_edge(&[0, 1, 3, 5]);
        println!("hg:\n{:}", hg);
        let s = hg.to_string();
        let hg_parsed = HGraph::from_str(&s).expect("no parsing?");
        println!("hg_parsed:\n{:}", hg_parsed);
        dbg!(&hg.core);
        let s3 = serde_json::to_string(&hg).expect("could not serialize next_usable_node");

        dbg!(s3);
    }

    #[test]
    fn test_link() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..=5]);
        hg.add_edge(&nodes[5..]);
        let link = hg.link(HashSet::from([nodes[5], nodes[4]]));
        println!("hg\n{:}", hg);
        dbg!(link);
    }

    #[test]
    fn test_skeleton() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        for size in 0..8 {
            hg.add_edge(&nodes[0..=size]);
        }
        for size in 1..10 {
            println!("{:}-skeleton", size);
            println!("{:?}", hg.k_skeleton(size));
        }
    }

    fn simple_test_hg() -> HGraph {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.add_edge(&nodes[0..=5]);
        hg.add_edge(&nodes[5..]);
        hg
    }
    #[test]
    fn test_cut_with_traits() {
        let mut hg = HGraph::new();
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
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(3);
        let e0 = hg.add_edge(&[0]);
        let e1 = hg.add_edge(&[0, 1]);
        let e2 = hg.add_edge(&[0, 1, 2]);
        let star = hg.get_containing_edges_id(&e0);
        dbg!(star);
        dbg!(&hg);
        println!(
            "degrees in order: {:}, {:}, {:}.",
            hg.degree(0),
            hg.degree(1),
            hg.degree(2)
        )
    }
}
