use std::collections::HashSet;

use uuid::Uuid;

use crate::structs::{nodes::NodeID, *};

pub fn walk<N: NodeID>(
    start: HgVector<N>,
    walk_operator: &HyperGraph<N>,
    num_steps: usize,
) -> HgVector<N> {
    let mut ret = start;
    for _ in 0..num_steps {
        ret = walk_operator.map_vec(ret);
    }
    ret
}

pub fn compute_probabilistic_walk_graph<N: NodeID>(graph: &HyperGraph<N>) -> HyperGraph<N> {
    HyperGraph::new()
}

pub fn compute_cut<N: NodeID>(selected_nodes: HashSet<N>, graph: &HyperGraph<N>) {
    let mut pot_edges = HashSet::new();
    for node in selected_nodes.iter() {
        let new_edges = graph.get_nodes_containing_edges(node);
        for e in new_edges {
            pot_edges.insert(e);
        }
    }
}
