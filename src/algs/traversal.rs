use std::collections::HashSet;

use uuid::Uuid;

use crate::structs::*;
use crate::traits::*;

pub fn walk<N: HgNode>(
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

pub fn bfs_base<B: HgBasis>(graph: GeneroGraph<B>, start: &B, steps: usize) {
    // TODO: change this to a dequeue.
    // let mut frontier = Vec::new();
    // let mut visited = HashSet::new();
    todo!()
}

pub fn compute_probabilistic_walk_graph<N: HgNode>(graph: &SparseGraph<N>) -> SparseGraph<N> {
    SparseGraph::<N>::new()
}

pub fn compute_cut<N: HgNode>(selected_nodes: HashSet<N>, graph: &SparseGraph<N>) {
    let mut pot_edges = HashSet::new();
    for node in selected_nodes.iter() {
        let new_edges = graph.get_outbound_edges(node);
        for e in new_edges {
            pot_edges.insert(e);
        }
    }
}
