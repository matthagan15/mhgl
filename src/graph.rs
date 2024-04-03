use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    structs::{EdgeWeight, GeneroEdge, GeneroGraph},
    traits::HgNode,
    EdgeDirection, SparseNodeSet,
};

/// A basic Undirected Graph. Uses a sparse representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph<N: HgNode> {
    nodes: HashSet<N>,
    next_usable_node: N,
    reusable_nodes: HashSet<N>,
    graph: GeneroGraph<SparseNodeSet<N>>,
}

impl<N: HgNode> Graph<N> {
    pub fn new() -> Self {
        Graph {
            nodes: HashSet::new(),
            next_usable_node: N::zero(),
            reusable_nodes: HashSet::new(),
            graph: GeneroGraph::new(),
        }
    }

    /// Returns true if node added successfully, false otherwise
    pub fn add_node(&mut self, node: N) -> bool {
        if self.nodes.contains(&node) {
            false
        } else {
            if self.reusable_nodes.contains(&node) {
                self.reusable_nodes.remove(&node);
            }
            if self.next_usable_node == node {
                self.next_usable_node.plus_one();
            }
            self.nodes.insert(node.clone());
            true
        }
    }

    pub fn add_nodes(&mut self, nodes: Vec<N>) {
        for node in nodes {
            self.add_node(node);
        }
    }

    pub fn from(edges: Vec<(N, N)>) -> Self {
        let mut g = Graph::new();
        for (u, v) in edges {
            g.add_edge(u, v);
        }
        g
    }

    pub fn from_weights(edges: Vec<(N, N, EdgeWeight)>) -> Self {
        let mut g = Graph::new();
        for (u, v, w) in edges {
            g.reusable_nodes.remove(&u);
            g.reusable_nodes.remove(&v);
            g.nodes.insert(u);
            g.nodes.insert(v);
            // Do this twice because could have v = u + 1.
            if g.next_usable_node == u || g.next_usable_node == v {
                g.next_usable_node.plus_one();
            }
            if g.next_usable_node == u || g.next_usable_node == v {
                g.next_usable_node.plus_one();
            }
            let e = GeneroEdge::from(
                SparseNodeSet::from_slice(&[u]),
                SparseNodeSet::from_slice(&[v]),
                w,
                EdgeDirection::Symmetric,
            );
            g.graph.add_edge(e);
        }
        g
    }

    pub fn add_edge(&mut self, u: N, v: N) {
        if self.nodes.contains(&u) == false {
            self.add_node(u);
        }
        if self.nodes.contains(&v) == false {
            self.add_node(v);
        }
        let u_basis = SparseNodeSet::from(&u);
        let v_basis = SparseNodeSet::from(&v);
        self.graph.add_edge(GeneroEdge::from(
            u_basis,
            v_basis,
            1.0,
            EdgeDirection::Symmetric,
        ));
    }

    /// Returns the neighbors of a node.
    pub fn neighbors(&self, node: N) -> Vec<N> {
        let b = SparseNodeSet::from(&node);
        self.graph
            .map_basis(&b)
            .basis()
            .into_iter()
            .map(|(b, _)| {
                let v: Vec<N> = b.node_set().into_iter().collect();
                v[0] // TODO: This may badly panic?
            })
            .collect()
    }

    pub fn minimum_weight_perfect_match(&self) {}
}
