use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use serde::{Deserialize, Serialize};


use uuid::Uuid;

use crate::traits::{HgBasis, HgNode};

use super::{
    generic_edge::{EdgeDirection, GeneroEdge}, generic_vec::GeneroVector, sparse_basis::Edge, EdgeID, EdgeWeight, GraphID
};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defaults to undirected hypergraph, user has to declare it a simplex.
pub struct HGraphCore<N: HgNode> {
    pub edges: HashMap<EdgeID, Edge<N>>,
    pub node_to_containing_edges: HashMap<N, HashSet<EdgeID>>,
    is_simplex: bool,
}

impl<N: HgNode> HGraphCore<N> {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            node_to_containing_edges: HashMap::new(),
            is_simplex: false,
        }
    }
    pub fn contains_node(&self, node: &N) -> bool {
        self.node_to_containing_edges.contains_key(node)
    }
    
    /// Returns `None` if the edge is not present. Otherwise retrieves the unique id associated with this edge (no duplicate edges allowed).
    /// 
    /// Returns `nil` for empty query
    /// 
    pub fn query_id<E>(&self, edge: E) -> Option<EdgeID>
        where E: Into<Edge<N>>
    {
        let e: Edge<N> = if self.is_simplex {
            edge.into().make_simplex() 
        } else {
            edge.into()
        };
        if e.is_empty() {
            return Some(Uuid::nil());
        }
        let first = e.get_first_node().unwrap();
        if self.node_to_containing_edges.contains_key(&first) == false {
            return None;
        }
        let candidate_ids = self.node_to_containing_edges.get(&first).unwrap();
        for candidate_id in candidate_ids {
            let candidate = self.edges.get(candidate_id).expect("Edge invariant violated.");
            if *candidate == e {
                return Some(candidate_id.clone());
            }
        }
        None
    }

    pub fn get_containing_edges_id(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        todo!()
    }

    /// what is the right behavior for adding a sub-face of a simplex? 
    pub fn add_edge<E>(&mut self, edge: E) -> Option<EdgeID>
    where E: Into<Edge<N>> {
        let edge: Edge<N> = if self.is_simplex {
            edge.into().make_simplex() 
        } else {
            edge.into()
        };

        let id = Uuid::new_v4();
        let nodes = edge.node_vec();
        self.add_nodes(nodes.clone());
        for node in nodes.iter() {
            let node_link = self.node_to_containing_edges.get_mut(node).expect("Node should already be present, I just added it.");
            node_link.insert(id.clone());
        }
        self.edges.insert(id.clone(), edge);
        Some(id)
    }

    /// Returns true if the node was added correctly, false if 
    /// the node was not added because it was already present.
    /// Does not overwrite existing nodes.
    pub fn add_node(&mut self, node: N) -> bool {
        if self.node_to_containing_edges.contains_key(&node) {
            false 
        } else {
            self.node_to_containing_edges.insert(node, HashSet::new()).is_none()
        }
    }

    /// Adds the provided nodes to the graph with no connective structure.
    /// Returns `None` if all nodes are new and added correctly, otherwise
    /// returns the list of nodes that were already present. Does not overwrite 
    /// any existing nodes.
    pub fn add_nodes(&mut self, nodes: Vec<N>) -> Option<Vec<N>> {
        let mut ret = Vec::new();
        for node in nodes.into_iter() {
            if !self.add_node(node) {
                ret.push(node);
            }
        }
        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }

    /// Removes the node, returning the edge ids of the
    /// effected edges. Does **NOTHING** if a node is not
    /// present in the graph.
    /// what to do if:
    /// - an edge is just modified
    /// - an edge is completely removed
    /// - a node is not present.
    pub fn remove_nodes(&mut self, nodes: &Vec<N>) -> Vec<EdgeID> {
        let mut effected_edges = HashSet::new();
        for node in nodes {
            let effected = self.node_to_containing_edges.remove(&node);
            if let Some(edge_id_set) = effected {
                for id in edge_id_set.into_iter() {
                    let edge = self.edges.get_mut(&id).expect("Edge invariant violated.");
                    edge.remove_node(node);
                    effected_edges.insert(id);
                }
            }
        }
        for e_id in effected_edges.iter() {
            let edge = self.edges.get(e_id).expect("Edge Invariant violated.");
            if edge.is_empty() {
                self.edges.remove(e_id);
            }
        }
        effected_edges.into_iter().collect()
    }

    pub fn remove_node(&mut self, node: N) -> Vec<EdgeID> {
        self.remove_nodes(&vec![node])
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
        where E: Into<Edge<N>>
    {
        let e: Edge<N> = if self.is_simplex {
            edge.into().make_simplex() 
        } else {
            edge.into()
        };
        if e.is_empty() {
            return true;
        }
        let first = e.get_first_node().unwrap();
        if self.node_to_containing_edges.contains_key(&first) == false {
            return false;
        }
        let candidate_ids = self.node_to_containing_edges.get(&first).unwrap();
        for candidate_id in candidate_ids {
            let candidate = self.edges.get(candidate_id).expect("Edge invariant violated.");
            let candidite_is_good = if self.is_simplex {
                candidate.contains(&e)
            } else {
                *candidate == e
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
        where E: Into<Edge<N>>
    {
        let e: Edge<N> = if self.is_simplex {
            edge.into().make_simplex() 
        } else {
            edge.into()
        };
        if e.is_empty() {
            return vec![];
        }
        let first = e.get_first_node().unwrap();
        if self.node_to_containing_edges.contains_key(&first) == false {
            return vec![];
        }
        let candidate_ids = self.node_to_containing_edges.get(&first).unwrap();
        let mut ret = Vec::new();
        for candidate_id in candidate_ids {
            let candidate = self.edges.get(candidate_id).expect("Edge invariant violated.");
            let candidite_is_good = if self.is_simplex {
                candidate.contains(&e)
            } else {
                *candidate == e
            };
            if candidite_is_good {
                ret.push(candidate_id.clone());
            }
        }
        ret
    }
    
    pub fn make_simplex(&mut self) {
        if self.is_simplex {
            return;
        }
        let new_edges = self.edges.drain().map(|(id, e)| (id, e.make_simplex())).collect();
        self.edges = new_edges;
        self.is_simplex = true;
    }

    pub fn make_undirected(&mut self) {
        if !self.is_simplex {
            return;
        }
        let new_edges = self.edges.drain().map(|(id, e)| (id, e.make_undirected())).collect();
        self.edges = new_edges;
        self.is_simplex = false;
    }

    pub fn remove_edge(&mut self, edge_id: EdgeID) -> Option<Edge<N>> {
        if let Some(e) = self.edges.remove(&edge_id) {
            for node in e.nodes_ref() {
                let containing_edges = self.node_to_containing_edges.get_mut(node).expect("Why is edge not in here.");
                containing_edges.remove(&edge_id);
            }
            Some(e)
        } else {
            None
        }
    }



    /// I need to put the IDs into the edge structs. 
    /// There is an API problem to solve - What type do you return? 
    /// If this is a simplex, I should return the maximal complements.
    /// If I'm a hypergraph I should return something else. I think this 
    /// should move up a layer, to things like HGraph or `SComplex`
    /// 
    /// The issue is that complements are not guaranteed to be contained
    /// in the hypergraph. So what should I return? An Edge with the proper 
    /// types. If there are multiple edges in the graph that contain the 
    /// provided edge we just return the complement, not multiple copies.
    /// 
    pub fn link<E: Into<Edge<N>> + ?Sized>(&self, edge: &E) -> Option<Vec<Edge<N>>> {
        todo!()
    }

    /// Do we include the given edges? AKA is this strict?
    /// ```
    /// let g = Graph::new();
    /// let e1 = g.add_edge([1_u8, 2, 3]);
    /// let maxes = g.maximal_containing_edges(&e1);
    /// assert_eq!(maxes, vec![e1])
    /// ```
    pub fn maximal_containing_edges(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        
        todo!()
    }

    /// This one can stay in here.
    pub fn star(&self, edge: &Edge<N>) -> Option<Vec<EdgeID>> {
        todo!()
    }

    pub fn star_id(&self, edge_id: &EdgeID) -> Vec<EdgeID> {
        todo!()
    }

}

mod tests {
    use super::HGraphCore;

    #[test]
    fn test_simple_tasks() {
        let mut g = HGraphCore::<u8>::new();
        g.make_simplex();
        let e1 = g.add_edge(&[1_u8, 2, 3][..]).unwrap();
        let e2 = g.add_edge(vec![1, 2, 4]).unwrap();
        let e3 = g.add_edge([5_u8,6,7]).unwrap();
        assert!(g.query([1_u8, 2, 3]));
        // is simplex so this should work
        assert!(g.query([1_u8, 2]));
        assert!(!g.query(&[0][..]));
        let containing_edges = g.get_containing_edges([1_u8, 2]);
        assert_eq!(containing_edges.len(), 2);
        assert!(containing_edges.contains(&e1));
        assert!(containing_edges.contains(&e2));
        let affected_edges = g.remove_node(2);
        assert!(affected_edges.contains(&e1));
        assert!(affected_edges.contains(&e2));
        assert!(g.query([1_u8, 3]));
        assert!(!g.query([1_u8, 2, 3]));
        assert_eq!(g.remove_nodes(&vec![5, 6, 7])[0], e3);
        assert!(!g.query([5, 6, 7]));

    }
}

/// The underlying structure for the directed graph types. Generic over
/// the basis type provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneroGraph<B: HgBasis> {
    pub edges: HashMap<EdgeID, GeneroEdge<B>>,
    node_to_outbound_edges: HashMap<B, HashSet<EdgeID>>,
}

impl<B: HgBasis> GeneroGraph<B> {
    pub fn new() -> Self {
        GeneroGraph {
            edges: HashMap::new(),
            node_to_outbound_edges: HashMap::new(),
        }
    }

    pub fn clone_edges(&self) -> Vec<EdgeID> {
        self.edges.keys().cloned().collect()
    }

    /// Returns all EdgeIDs that map from this basis to another.
    pub fn get_outbound_edges(&self, basis: &B) -> HashSet<EdgeID> {
        // TODO: This is inefficient. Take the intersection of edge_ids
        // from node_to_outbound_edges before checking if they can map the
        // given basis.
        let mut ret = HashSet::new();
        for node in basis.nodes() {
            if let Some(potentials) = self.node_to_outbound_edges.get(&node) {
                for edge_id in potentials {
                    if let Some(edge) = self.edges.get(edge_id) {
                        if edge.can_map_basis(basis) {
                            ret.insert(edge_id.clone());
                        }
                    }
                }
            }
        }
        ret
    }

    pub fn query_edge(&self, edge_id: &EdgeID) -> Option<GeneroEdge<B>> {
        self.edges.get(edge_id).cloned()
    }

    /// Gets all edges such that the basis is contained in the union
    /// of the edges input and output
    pub fn get_containing_edges(&self, basis: &B) -> HashSet<EdgeID> {
        let mut ret = HashSet::new();
        for (id, edge) in self.edges.iter() {
            if edge.contains(basis) {
                ret.insert(id.clone());
            }
        }
        ret
    }

    /// Returns the cardinality of the edge for undirecteds
    pub fn get_edge_len(&self, edge_id: &EdgeID) -> Option<usize> {
        self.edges.get(edge_id).map(|e| e.input_cardinality())
    }

    /// Warning: currently only works for undirected graph types (such as HGraph)
    /// Should we include the provided edge or no? If the provided edge is 
    /// not present then we were given a bunk edge_id. If it is then we were
    /// able to at least find it.
    pub fn get_containing_edges_id(&self, edge_id: &EdgeID) -> HashSet<EdgeID> {
        if let Some(edge) = self.edges.get(edge_id) {
            let nodes: Vec<B> = edge.node_vec();
            if nodes.len() == 0 {
                HashSet::from([edge_id.clone()])
            } else {
                let n0 = &nodes[0];
                let outbounds = self.node_to_outbound_edges.get(n0).unwrap();
                outbounds.iter().filter(|e_id| {
                    let e = self.edges.get(e_id).unwrap();
                    e.contains(&edge.in_nodes)
                }).cloned().collect()
            }
        } else {
            HashSet::new()
        }
    }

    pub fn add_edge(&mut self, new_edge: GeneroEdge<B>) -> EdgeID {
        let new_id = Uuid::new_v4();
        match new_edge.direction {
            EdgeDirection::Directed  | EdgeDirection::Loop | EdgeDirection::Undirected  | EdgeDirection::Simplicial => {
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_id.clone());
                }
                self.edges.insert(new_id, new_edge);
                new_id
            }
            EdgeDirection::Symmetric => {
                for node in new_edge.in_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_id.clone());
                }
                for node in new_edge.out_nodes.nodes() {
                    self.node_to_outbound_edges
                        .entry(node)
                        .or_default()
                        .insert(new_id.clone());
                }
                self.edges.insert(new_id.clone(), new_edge);
                new_id
            }
        }
    }

    pub fn remove_edge(&mut self, edge_id: &EdgeID) -> Option<GeneroEdge<B>> {
        if let Some(edge) = self.edges.remove(edge_id) {
            match edge.direction {
                EdgeDirection::Directed | EdgeDirection::Loop => {
                    for node in edge.in_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    Some(edge)
                }
                EdgeDirection::Symmetric => {
                    for node in edge.in_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    for node in edge.out_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    Some(edge)
                }
                EdgeDirection::Undirected | EdgeDirection::Simplicial => {
                    for node in edge.in_nodes.nodes() {
                        if let Some(set) = self.node_to_outbound_edges.get_mut(&node) {
                            set.remove(edge_id);
                        }
                    }
                    Some(edge)
                }
            }
        } else {
            None
        }
    }

    pub fn edges_of_size(&self, size: usize) -> Vec<EdgeID> {
        self.edges
            .iter()
            .filter_map(|(k, v)| {
                if v.input_cardinality() == size {
                    Some(k)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }

    /// Change the input of an existing edge. If edge is a undirected type it will
    /// simply replace the undirected basis with the new basis, keeping the ID
    /// the same.
    pub fn change_edge_input(&mut self, edge_id: &EdgeID, new_input: B) {
        // Due to undirecteds it is simply easier to remove the edge and reinsert
        // the modified edge.
        if let Some(mut e) = self.remove_edge(edge_id) {
            e.change_input(new_input);
            self.add_edge(e);
        }
    }

    /// Change the output of the provided edge_id to the new basis. If edge
    /// is a undirected or loop then nothing is done, use `change_edge_input` instead.
    pub fn change_edge_output(&mut self, edge_id: &EdgeID, new_output: B) {
        // Edge is removed and re-added to avoid duplicating logic of undirected
        // or undirected style edges. For example changing output of an undirected
        // edge requires changing all of the inputs/outgoing edges from the
        // outbound map.
        if let Some(mut e) = self.remove_edge(edge_id) {
            e.change_output(new_output);
            self.add_edge(e);
        }
    }

    /// Returns the sum total of all edge weights mapping input basis `input` to output basis `output`.
    pub fn query_weight(&self, input: &B, output: &B) -> EdgeWeight {
        let mut ret = 0.;
        for (b, w) in self.map_basis(input).to_tuples() {
            if b == *output {
                ret += w;
            }
        }
        ret
    }

    pub fn query_edges(&self, input: &B, output: &B) -> Vec<EdgeID> {
        let outbounds = self.get_outbound_edges(input);
        outbounds
            .into_iter()
            .filter(|e| {
                if let Some(edge) = self.edges.get(e) {
                    edge.is_correctly_mapped(input, output)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns edge_id of undirected edges on the input basis.
    pub fn query_undirected(&self, input: &B) -> Vec<EdgeID> {
        let mut potential_edges = HashSet::new();
        for node in input.nodes() {
            if potential_edges.len() == 0 && self.node_to_outbound_edges.contains_key(&node) {
                potential_edges = potential_edges
                    .union(self.node_to_outbound_edges.get(&node).unwrap())
                    .cloned()
                    .collect();
            } else if potential_edges.len() > 0 && self.node_to_outbound_edges.contains_key(&node) {
                potential_edges = potential_edges
                    .intersection(self.node_to_outbound_edges.get(&node).unwrap())
                    .cloned()
                    .collect();
            }
        }
        potential_edges
            .into_iter()
            .filter(|potential_edge| {
                if let Some(e) = self.edges.get(&potential_edge) {
                    e.matches_undirected(input)
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn query_loop(&self, input: &B) -> Vec<Uuid> {
        let possible_loops = self.get_outbound_edges(input);
        possible_loops
            .into_iter()
            .filter(|id| {
                if let Some(e) = self.edges.get(id) {
                    e.is_correctly_mapped(input, input)
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn map_basis(&self, input: &B) -> GeneroVector<B> {
        let mut ret = GeneroVector::new();
        let mut good_edges = HashSet::new();
        for node in input.nodes() {
            if let Some(edges) = self.node_to_outbound_edges.get(&node) {
                for edge_id in edges {
                    if let Some(edge) = self.edges.get(edge_id) {
                        if edge.can_map_basis(input) {
                            good_edges.insert(edge_id);
                        }
                    }
                }
            }
        }
        for edge_id in good_edges {
            let e = self
                .edges
                .get(edge_id)
                .expect("This was checked in prior loop.");
            ret += &e.map_to_vector(input);
        }
        ret
    }
    pub fn map(&self, input: &GeneroVector<B>) -> GeneroVector<B> {
        let ret = GeneroVector::new();
        for (b, w) in input.basis_to_weight.iter() {
            let mut tmp = self.map_basis(&b);
            tmp *= *w;
        }
        ret
    }
}
