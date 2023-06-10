use std::collections::HashSet;

use rand::thread_rng;
use rand::Rng;

use crate::structs::*;
use crate::traits::*;

/// Applies an input graph to an input vector a specified amount of times.
pub fn walk<B: HgBasis, H: HyperGraph<Basis = B>>(
    start: GeneroVector<B>,
    walk_operator: &H,
    num_steps: usize,
) -> GeneroVector<B> {
    let mut ret = start;
    for _ in 0..num_steps {
        ret = walk_operator.map_vector(&ret);
    }
    ret
}

/// first pass at basic BFS, probably something done incorrectly
pub fn bfs_base<B: HgBasis, H: HyperGraph<Basis = B>>(
    graph: &H,
    start: &B,
    steps: usize,
) -> Vec<HgPath<B>> {
    // TODO: change this to a dequeue.
    let mut visited = HashSet::new();
    let start_path = HgPath::new(start.clone());
    let mut frontier = vec![start_path];
    let mut completed = Vec::new();
    while frontier.len() > 0 {
        let cur_path = frontier.pop().expect("loop should not execute if empty.");
        visited.insert(cur_path.last_basis());
        for (b, w) in graph.map_basis(&cur_path.last_basis()) {
            let path = cur_path.clone() + (b, w);
            if path.len() < steps && visited.contains(&path.last_basis()) == false {
                frontier.insert(0, path);
            } else if path.len() == steps {
                completed.push(path);
            }
        }
    }
    completed
}

// Thoughts on making an enum of walkers?
// Thoughts on making iterators of walkers?
/// First pass at basic DFS, probably something done incorrectly.
pub fn dfs_base<B: HgBasis>(graph: &GeneroGraph<B>, start: &B, steps: usize) -> Vec<HgPath<B>> {
    let mut visited = HashSet::new();
    let start_path = HgPath::new(start.clone());
    let mut frontier = vec![start_path];
    let mut completed = Vec::new();
    while frontier.len() > 0 {
        let cur_path = frontier.pop().expect("loop should not execute if empty.");
        visited.insert(cur_path.last_basis());
        let new_paths = cur_path.extend(graph);
        for path in new_paths.into_iter() {
            if path.len() < steps && visited.contains(&path.last_basis()) == false {
                frontier.push(path);
            } else if path.len() == steps {
                completed.push(path);
            }
        }
    }
    completed
}

pub fn random_walk<B: HgBasis>(graph: &GeneroGraph<B>, start: &B, steps: usize) -> B {
    let mut rng = thread_rng();
    let mut walker_location = start.clone();
    for _ in 0..steps {
        let outputs = graph.map_basis(start).to_tuples();
        let tot = outputs.iter().fold(0., |acc, (_b, w)| acc + w);
        for ix in 0..outputs.len() {
            if outputs[ix].1 / tot <= rng.gen() {
                walker_location = outputs[ix].0.clone();
                break;
            }
        }
    }
    walker_location
}

mod test {
    use crate::{algs::traversal::bfs_base, structs::SparseBasis, HGraph};

    use super::random_walk;

    #[test]
    fn test_bfs() {
        let mut hg = HGraph::new();
        let mut nodes = hg.create_nodes(10);
        nodes.sort();
        let start = &nodes[0..2];
        let b1 = [nodes[0], nodes[1], nodes[2]];
        let b2 = [nodes[0], nodes[1], nodes[3]];
        let b3 = [nodes[3]];
        let b4 = [nodes[4]];
        let b5 = [nodes[5]];
        hg.create_edge(start, &b1, 1., crate::EdgeDirection::Directed);
        hg.create_edge(start, &b2, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b1, &b3, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b1, &b4, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b2, &b3, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b2, &b5, 1., crate::EdgeDirection::Directed);
        println!("nodes: {:#?}", nodes);
        println!("graph: {:#?}", hg.graph);
        println!(
            "bfs: {:#?}",
            bfs_base(&hg, &SparseBasis::from(start.iter().cloned().collect()), 2)
        );
    }

    #[test]
    fn test_random_walk() {
        let mut hg = HGraph::new();
        let mut nodes = hg.create_nodes(10);
        nodes.sort();
        let start = &nodes[0..2];
        let start_basis = SparseBasis::from(start.iter().cloned().collect());
        let b1 = [nodes[0], nodes[1], nodes[2]];
        let b2 = [nodes[0], nodes[1], nodes[3]];
        let b3 = [nodes[3]];
        let b4 = [nodes[4]];
        let b5 = [nodes[5]];
        hg.create_edge(start, &b1, 1., crate::EdgeDirection::Directed);
        hg.create_edge(start, &b2, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b1, &b3, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b1, &b4, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b2, &b3, 1., crate::EdgeDirection::Directed);
        hg.create_edge(&b2, &b5, 1., crate::EdgeDirection::Directed);
        println!("graph:\n{:#?}", hg);
        println!(
            "random_walk output: {:#?}",
            random_walk(&hg.graph, &start_basis, 5)
        )
    }
}
