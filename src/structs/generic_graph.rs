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