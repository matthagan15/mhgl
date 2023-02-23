use std::collections::HashSet;

use rand::Rng;
use uuid::Uuid;

use crate::structs::{EdgeDirection, NodeID, SparseEdge, SparseGraph, SparseBasis, GeneroGraph};
use crate::{traits::*, HGraph};
use crate::utils::power_set;

/// A basic erdos_renyi hypergraph where the probability for each dimension of input and output edge can be
/// specified. For example, an erdos_renyi hypergraph with only the probability (1,1,p) specified is equivalent
/// to the standard erdos-renyi random graph. This means that our edges are undirected.
pub fn erdos_renyi(
    num_nodes: usize,
    dimension_with_probability: Vec<(usize, usize, f64)>,
) -> HGraph {
    let mut hg = HGraph::new();
    let nodes = hg.create_nodes(num_nodes);
    let mut rng = rand::thread_rng();
    let mut seen_dims: HashSet<usize> = HashSet::new();
    for (in_dim, out_dim, prob) in dimension_with_probability {
        if seen_dims.contains(&in_dim) && seen_dims.contains(&out_dim) {
            continue;
        }
        if prob < 0.0 || prob > 1.0 {
            println!("[erdos_renyi] invalid probability encountered.");
            continue;
        }
        let pot_inputs = power_set(nodes.clone().into_iter().collect(), in_dim);
        let pot_outputs = power_set(nodes.clone().into_iter().collect(), out_dim);

        // Zero len checks are necessary in case empty set is a desired terminal of
        // an edge.
        if pot_inputs.len() == 0 && pot_outputs.len() > 0 {
            for out in pot_outputs.iter() {
                if rng.gen_bool(prob) {
                    let in_set = HashSet::new();
                    let out_set = out.into_iter().cloned().collect();
                    let e = SparseEdge::from(in_set, out_set, EdgeDirection::Undirected);
                    // hg.add_edge(e);
                    hg.create_edge(&in_set.into_iter().collect(), outputs, weight, direction)
                }
            }
        } else if pot_outputs.len() == 0 && pot_inputs.len() > 0 {
            for inp in pot_inputs.iter() {
                if rng.gen_bool(prob) {
                    let in_set = inp.into_iter().cloned().collect();
                    let out_set = HashSet::new();
                    let e = SparseEdge::from(in_set, out_set, EdgeDirection::Undirected);
                    hg.add_edge(e);
                }
            }
        } else {
            for inp in pot_inputs.iter() {
                for out in pot_outputs.iter() {
                    if rng.gen_bool(prob) {
                        let in_set = inp.into_iter().cloned().collect();
                        let out_set = out.into_iter().cloned().collect();
                        let e = SparseEdge::from(in_set, out_set, EdgeDirection::Undirected);
                        hg.add_edge(e);
                    }
                }
            }
        }
        seen_dims.insert(in_dim);
        seen_dims.insert(out_dim);
    }
    hg
}

pub fn uniform_undirected_erdos_renyi<N: HgNode>(num_nodes: usize, prob: f64) -> SparseGraph<N> {
    if num_nodes < u8::MAX as usize {
        SparseGraph::new()
    } else {
        SparseGraph::new()
    }
}

mod test {
    use super::erdos_renyi;

    #[test]
    fn test_erdos_renyi() {
        let num_nodes = 4;
        let dim_w_probs = vec![
            (1, 1, 0.5),
            (1, 2, 0.5),
            (0, 4, 1.),
            (4, 3, 0.3),
            (3, 2, 0.6),
        ];
        let h: crate::structs::SparseGraph<_> = erdos_renyi(num_nodes, dim_w_probs);
        println!("{:#?}", h);
    }
}
