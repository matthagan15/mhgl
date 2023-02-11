use std::collections::HashSet;

use uuid::Uuid;

use crate::structs::{nodes::NodeID, *, hgraph::HyperGraph};

pub fn walk<N: NodeID>(
    start: SparseVector<N>,
    walk_operator: &SparseGraph<N>,
    num_steps: usize,
) -> SparseVector<N> {
    let mut ret = start;
    for _ in 0..num_steps {
        ret = walk_operator.map_vec(ret);
    }
    ret
}

pub fn bfs() {}

pub fn compute_probabilistic_walk_graph<N: NodeID>(graph: &SparseGraph<N>) -> SparseGraph<N> {
    SparseGraph::<N>::new()
}

pub fn compute_cut<N: NodeID>(selected_nodes: HashSet<N>, graph: &SparseGraph<N>) {
    let mut pot_edges = HashSet::new();
    for node in selected_nodes.iter() {
        let new_edges = graph.get_outbound_edges(node);
        for e in new_edges {
            pot_edges.insert(e);
        }
    }
}
