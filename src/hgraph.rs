use std::collections::{HashSet, VecDeque};
use std::fmt::Display;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::{EdgeWeight, GeneroEdge, GeneroGraph, SparseBasis};

use crate::traits::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
/// let mut hm: HashMap<Uuid, NodeType> = HashMap::new();
/// let node_data: Vec<NodeType> = data_set.load();
/// let node_ids: Vec<Uuid> = HGraph::add_nodes(node_data.len());
/// for ix in node_data.into_iter() {
///     hm.insert(node_ids[ix], node_data[ix])
/// }
/// ```
/// Then data can be accessed by querying `hm[id]`.
pub struct HGraph {
    nodes: HashSet<u32>,
    next_usable_node: u32,
    reusable_nodes: VecDeque<u32>,
    graph: GeneroGraph<SparseBasis<u32>>,
    edge_query_set: HashSet<Vec<u32>>,
}

impl HGraph {
    pub fn new() -> HGraph {
        HGraph {
            nodes: HashSet::new(),
            next_usable_node: 0,
            reusable_nodes: VecDeque::new(),
            graph: GeneroGraph::new(),
            edge_query_set: HashSet::new(),
        }
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        // check if path is a given file
        if path.is_file() == false {
            return None;
        }
        if let Ok(hg_json) = fs::read_to_string(path) {
            if let Ok(serde_out) = serde_json::from_str::<HGraph>(&hg_json) {
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
        if self.next_usable_node < u32::MAX {
            let ret = self.next_usable_node;
            self.next_usable_node += 1;
            self.nodes.insert(ret);
            ret
        } else if self.reusable_nodes.len() > 0 {
            self.reusable_nodes.pop_front().expect("No nodes left.")
        } else {
            panic!("No nodes remaining to be added.")
        }
    }

    /// Adds `num_nodes` nodes to the graph, returning a vector containing
    /// the nodes created. The number of nodes returned may be less than
    /// the number of nodes requested due to the use of u32 to store nodes.
    /// Nodes that get deleted are reused in a First In First Out (FIFO) format.
    // TODO: This should panic if it cannot offer the right amount of nodes.
    // Or return a Ret<Ok, Err> type. That would be the best option.
    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<u32> {
        // TODO: Should the user control what nodes are present? We don't
        // really care what numbers are used to store nodes, so why go through
        // all this hassle
        let mut ret = Vec::with_capacity(num_nodes);
        let mut counter = self.next_usable_node;
        let mut nodes_available = counter < u32::max_number() || self.reusable_nodes.len() > 0;
        while nodes_available && ret.len() < num_nodes {
            // Prefer adding never before seen nodes.
            if counter < u32::max_number() {
                if self.nodes.contains(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.nodes.insert(counter);
                    ret.push(counter);
                }
                counter += 1;
            } else {
                // If the counter has reached the max, then we start reusing nodes
                // TODO: This is rather inefficient, can just cache a boolean
                // if we already added the max value or not.
                if self.nodes.contains(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.nodes.insert(counter);
                    ret.push(counter);
                } else {
                    if let Some(old_node) = self.reusable_nodes.pop_front() {
                        if self.nodes.contains(&old_node) == false {
                            self.nodes.insert(old_node);
                            ret.push(old_node);
                        }
                    }
                }
            }
            nodes_available = counter < u32::max_number() || self.reusable_nodes.len() > 0;
        }
        self.next_usable_node = counter;
        ret
    }

    /// Removes a node from the node set. The deleted node will be added to a
    /// dequeue to be reused later once all possible nodes have been created.
    pub fn remove_node(&mut self, node: u32) {
        if self.nodes.contains(&node) == false {
            return;
        }
        let node_basis = SparseBasis::from(HashSet::from([node]));
        let edges = self.graph.get_containing_edges(&node_basis);
        for edge in edges {
            if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                old_edge.remove_node(&node_basis);
                self.graph.add_edge(old_edge);
            }
        }
        self.nodes.remove(&node);
        self.reusable_nodes.push_back(node);
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: Vec<u32>) {
        for node in nodes {
            self.remove_node(node);
        }
    }

    pub fn nodes(&self) -> Vec<u32> {
        self.nodes.clone().into_iter().collect()
    }

    /// Creates an undirected edge among the given nodes. Duplicate inputs are removed. Allows for duplicate edges. Returns the Uuid of the created edge.
    // TODO: rename to add_edge
    pub fn create_edge(&mut self, nodes: &[u32]) -> Uuid {
        // TODO: This can be made much faster for HGraph if we
        // take a memory hit by storing a HashSet of each
        // subset/edge we have seen.
        let input_basis = SparseBasis::from_slice(nodes);
        let mut query_vec = Vec::from(nodes);
        query_vec.sort();
        let e: GeneroEdge<SparseBasis<u32>> = input_basis.into();
        let id = e.id.clone();
        self.graph.add_edge(e);
        self.edge_query_set.insert(query_vec);
        id
    }

    /// Creates an edge if none exists, but does not create a duplicate edge.
    pub fn create_edge_no_dups(&mut self, nodes: &[u32]) {
        if self.query_edge(nodes) == false {
            self.create_edge(nodes);
        }
    }

    pub fn remove_edge(&mut self, nodes: &[u32]) {
        let input_basis = SparseBasis::from_slice(nodes);
        let mut query_vec = Vec::from(nodes);
        query_vec.sort();
        let e = self.graph.query_undirected(&input_basis);
        if let Some(id) = e.first() {
            self.graph.remove_edge(id);
        }
        self.edge_query_set.remove(&query_vec);
    }

    /// Returns true if the provided nodes form an existing edge in
    /// the graph, false if they do not.
    pub fn query_edge(&self, nodes: &[u32]) -> bool {
        // let input_basis = SparseBasis::from_slice(nodes);
        // self.graph.query_undirected(&input_basis).len() > 0
        let mut query_vec = Vec::from(nodes);
        query_vec.sort();
        self.edge_query_set.contains(&query_vec)
    }

    pub fn query_edge_id(&self, id: &Uuid) -> Option<Vec<u32>> {
        if let Some(e) = self.graph.query_edge(id) {
            Some(e.in_nodes.to_node_vec())
        } else {
            None
        }
    }

    pub fn get_edge_id(&self, nodes: &[u32]) -> Option<Uuid> {
        let e = self.graph.query_undirected(&SparseBasis::from_slice(nodes));
        e.first().copied()
    }

    /// Computes the link of the provided nodes in the HyperGraph but returns a
    /// list of sets as opposed to a new HyperGraph.
    pub fn link_as_vec(&self, nodes: &[u32]) -> Vec<(HashSet<u32>, EdgeWeight)> {
        let start_basis = SparseBasis::from(nodes);
        let out_vector = self.graph.map_basis(&start_basis);
        out_vector
            .to_tuples()
            .into_iter()
            .map(|(b, w)| (b.to_node_set(), w))
            .collect()
    }

    pub fn edges_of_size(&self, card: usize) -> Vec<Uuid> {
        self.graph.edges_of_size(card).into_iter().collect()
    }

    pub fn get_containing_edges(&self, nodes: &[u32]) -> Vec<Uuid> {
        self.graph
            .get_containing_edges(&SparseBasis::from_slice(nodes))
            .into_iter()
            .collect()
    }

    /// Returns the hyperedges that contain the provided edge.
    /// Ex: Edges = [{a, b, c}, {a,b,c,d}, {a,b}, {a,b,c,d,e}]
    /// star({a,b,c}) = [{a,b,c,d}, {a,b,c,d,e}]
    pub fn star_id(&self, edge_id: &Uuid) -> Vec<Uuid> {
        self.graph
            .get_containing_edges_id(edge_id)
            .into_iter()
            .collect()
    }

    /// Returns a list of all edges in the graph.
    pub fn get_edges(&self) -> Vec<HashSet<u32>> {
        let edge_ids = self.graph.clone_edges();
        edge_ids
            .into_iter()
            .filter_map(|id| {
                self.graph
                    .query_edge(&id)
                    .map(|edge| edge.in_nodes.to_node_set())
            })
            .collect()
    }

    ///
    pub fn get_edges_with_ids(&self) -> Vec<(HashSet<u32>, Uuid)> {
        let edge_ids = self.graph.clone_edges();
        edge_ids
            .into_iter()
            .filter_map(|id| {
                self.graph
                    .query_edge(&id)
                    .map(|edge| (edge.in_nodes.to_node_set(), id))
            })
            .collect()
    }

    pub fn walk(&self, _start: &[u32]) {}

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
    pub fn cut<ToSet>(&self, cut_nodes: ToSet) -> usize
    where
        ToSet: Into<SparseBasis<u32>>,
    {
        let mut counted_edges: HashSet<Uuid> = HashSet::new();
        let cut_basis: SparseBasis<u32> = cut_nodes.into();
        dbg!(&cut_basis);
        for node in cut_basis.nodes() {
            let out_edges: Vec<Uuid> = self
                .graph
                .get_outbound_edges(&node)
                .into_iter()
                .filter(|e_id| counted_edges.contains(e_id) == false)
                .collect();
            for edge_id in out_edges {
                if let Some(e) = self.graph.edges.get(&edge_id) {
                    if cut_basis.covers_basis(&e.in_nodes) {
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
    pub fn link(&self, face: HashSet<u32>) -> HGraph {
        let v: Vec<u32> = face.clone().into_iter().collect();
        let face_basis = SparseBasis::from_slice(&v[..]);
        let out = self.graph.map_basis(&face_basis);
        let mut link = HGraph {
            nodes: self.nodes.clone(),
            next_usable_node: self.next_usable_node,
            reusable_nodes: self.reusable_nodes.clone(),
            graph: GeneroGraph::new(),
            edge_query_set: HashSet::new(),
        };
        for (b, _) in out.to_tuples() {
            link.create_edge(&b.node_vec()[..]);
        }
        link
    }

    /// Computes the k-skeleton of this hypergraph and returns the
    /// information as a new `HGraph`.
    /// The k-skeleton is defined as all undirected hyperedges of cardinality
    /// less than or equal to `k`.
    /// To mutate a given `HGraph` use `HGraph::project_onto`.
    pub fn k_skeleton(&self, k: usize) -> HGraph {
        let mut ret = HGraph::new();
        let mut new_graph = self.graph.clone();
        let mut new_edge_query_set = HashSet::new();
        new_graph.edges = new_graph
            .edges
            .into_iter()
            .filter(|(_, e)| {
                let mut query_vec = e.clone_input_nodes().to_node_vec();
                query_vec.sort();
                new_edge_query_set.insert(query_vec);
                e.input_cardinality() <= k + 1
            })
            .collect();
        ret.nodes = self.nodes.clone();
        ret.next_usable_node = self.next_usable_node;
        ret.reusable_nodes = self.reusable_nodes.clone();
        ret.graph = new_graph;
        ret.edge_query_set = new_edge_query_set;
        ret
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
        if self.nodes.len() == 0 {
            println!("Graph is empty. Add nodes for more fun.");
            return Ok(());
        }
        let mut s = String::new();
        s.push_str("nodes: [");
        let x: Vec<String> = self
            .nodes
            .clone()
            .into_iter()
            .map(|n| n.to_string())
            .collect();
        for ix in 0..x.len() - 1 {
            s.push_str(&x[ix]);
            s.push_str(", ");
        }
        s.push_str(x.last().unwrap());
        s.push_str("]\n");
        s.push_str("edges:\n");
        for e in self.graph.clone_edges() {
            let e = self.graph.query_edge(&e).unwrap();
            s.push_str(&e.in_nodes.to_string());
            s.push_str("\n");
        }
        f.write_str(&s)
    }
}

mod test {

    use std::collections::HashSet;

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
        // WARNING: The below was performed once to verify accuracy, do not
        // uncomment as this test will take forever.
        // hg.add_nodes((u32::MAX - 101_u32) as usize );
        // let get_removed = hg.add_nodes(1);
        // assert_eq!(get_removed[0], removed);
    }

    #[test]
    fn test_edge_creation_removal() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[0..5]);
        hg.create_edge(&nodes[0..6]);
        hg.remove_edge(&nodes[0..5]);
        assert!(hg.query_edge(&[nodes[4], nodes[3], nodes[2], nodes[1], nodes[0]]) == false);
        assert!(hg.query_edge(&nodes[0..6]))
    }

    #[test]
    fn test_serialization() {
        let mut hg = HGraph::new();
        hg.add_nodes(10);
        hg.create_edge(&[0, 1]);
        println!("hg:\n{:}", hg);
        let g = hg.graph.clone();
        dbg!(serde_json::to_string(&g).unwrap());
        dbg!(&hg.graph);
        let s2 = serde_json::to_string(&hg.nodes).expect("could not serialize nodes");
        let s3 = serde_json::to_string(&hg.next_usable_node)
            .expect("could not serialize next_usable_node");
        let s4 =
            serde_json::to_string(&hg.reusable_nodes).expect("could not serialize reusable_nodes");
        let s5 = serde_json::to_string(&hg.graph).expect("could not serialize graph");

        dbg!(s2);
        dbg!(s3);
        dbg!(s4);
        dbg!(s5);
    }

    #[test]
    fn test_deserialization() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[0..2]);
        hg.create_edge(&nodes[0..3]);
        hg.create_edge(&nodes[0..4]);
        hg.create_edge(&nodes[0..5]);
        let mut s = String::new();
        s = serde_json::to_string(&hg).unwrap();
        println!("s: {:}", s);
        let hg2: HGraph = serde_json::from_str(&s[..]).unwrap();
        println!("hg2:{:}", hg2);
    }

    #[test]
    fn test_link() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[0..=5]);
        hg.create_edge(&nodes[5..]);
        let link = hg.link(HashSet::from([nodes[5], nodes[4]]));
        println!("hg\n{:}", hg);
        println!("link\n{:}", link);
    }

    #[test]
    fn test_skeleton() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        for size in 0..8 {
            hg.create_edge(&nodes[0..=size]);
        }
        for size in 1..10 {
            println!("{:}-skeleton", size);
            println!("{:}", hg.k_skeleton(size));
        }
    }

    fn simple_test_hg() -> HGraph {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[0..=5]);
        hg.create_edge(&nodes[5..]);
        hg
    }
    #[test]
    fn test_cut_with_traits() {
        let mut hg = HGraph::new();
        let nodes = hg.add_nodes(10);
        hg.create_edge(&nodes[..2]);
        hg.create_edge(&nodes[..3]);
        hg.create_edge(&nodes[..4]);
        println!("hg\n{:}", hg);
        assert_eq!(hg.cut(&nodes[..2]), 2);
        assert_eq!(hg.cut(&nodes[..3]), 1);
        assert_eq!(hg.cut(&nodes[..4]), 0);
    }
}
