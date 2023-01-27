use std::collections::HashSet;

use uuid::Uuid;

use crate::structs::*;

pub fn walk(start: HgVector<Uuid>, graph: &HyperGraph, num_steps: usize) -> HgVector<Uuid> {
    let mut ret = start;
    for _ in 0..num_steps {
        ret = graph.map_vec(ret);
    }
    ret
}

pub fn compute_cut(selected_nodes: HashSet<NodeUUID>, graph: &HyperGraph) {
    let mut pot_edges = HashSet::new();
    for node in selected_nodes.iter() {
        let new_edges = graph.get_nodes_containing_edges(node);
        for e in new_edges {
            pot_edges.insert(e);
        }
    }
    
}