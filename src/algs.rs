use crate::{EdgeSet, HyperGraph};
use rand::prelude::*;

/// `UpDown` and `DownUp` maps edges of cardinality k to cardinality k
/// whereas link can map to different subset sizes.
#[derive(Debug, Clone)]
pub enum WalkType {
    Link,
    UpDown,
    DownUp,
}

/// Assumes the uniform distribution over outgoing edges for
/// `Link`, `UpDown`, and `DownUp` types. If at any point no output of
/// the chosen walk is generated then the walk returns `None` indicating a
/// zero vector or an invalid input was encountered. For example, calling
/// `boundary_up` on a maximal edge would not give an edge in the hypergraph,
/// so we return `None`. To further illustrate this, if you have a normal graph
/// with vertices and lines (i.e. pairs of vertices (u, v)) and you try to call
/// boundary up on a line (u, v), the graph does not have any triangles `(u, v, w)`
/// so we must return `None`. Note that this is because the empty set (`Vec::new()`)
/// can be a valid return option (think of the `boundary_down` of a single vertex {v}).
pub fn random_walk<Walker: HyperGraph>(
    hgraph: &Walker,
    start: impl AsRef<[Walker::NodeID]>,
    num_steps: usize,
    walk_type: WalkType,
) -> Option<Vec<Walker::NodeID>> {
    if num_steps == 0 {
        return None;
    }
    let mut rng = thread_rng();
    match walk_type {
        WalkType::Link => {
            let mut current_location: EdgeSet<Walker::NodeID> = start.into();
            for _ in 0..num_steps {
                let link_out = hgraph.link_of_nodes(current_location.node_vec());
                if link_out.len() == 0 {
                    return None;
                }
                let link_ix = rng.gen_range(0..link_out.len());
                let chosen_nodes = &link_out[link_ix].1;
                current_location = EdgeSet::from(&chosen_nodes[..]);
            }
            Some(current_location.to_node_vec())
        }
        WalkType::UpDown => {
            let mut current_location: Option<Walker::EdgeID> = None;
            for _ in 0..num_steps {
                let up_out = if current_location.is_none() {
                    hgraph.boundary_up_of_nodes(start.as_ref())
                } else {
                    hgraph.boundary_up(&current_location.unwrap())
                };
                if up_out.len() == 0 {
                    return None;
                }
                let up_out_ix = rng.gen_range(0..up_out.len());
                let down_out = hgraph.boundary_down(&up_out[up_out_ix]);
                if down_out.len() == 0 {
                    return None;
                }
                let down_out_ix = rng.gen_range(0..down_out.len());
                current_location = Some(down_out[down_out_ix]);
            }
            hgraph.query_edge(&current_location.unwrap())
        }
        WalkType::DownUp => {
            let mut current_location: Option<Walker::EdgeID> = None;
            for _ in 0..num_steps {
                let down_out = if current_location.is_none() {
                    hgraph.boundary_down_of_nodes(start.as_ref())
                } else {
                    hgraph.boundary_down(&current_location.unwrap())
                };
                if down_out.len() == 0 {
                    return None;
                }
                let down_out_ix = rng.gen_range(0..down_out.len());
                let up_out = hgraph.boundary_down(&down_out[down_out_ix]);
                if up_out.len() == 0 {
                    return None;
                }
                let up_out_ix = rng.gen_range(0..up_out.len());
                current_location = Some(up_out[up_out_ix]);
            }
            hgraph.query_edge(&current_location.unwrap())
        }
    }
}

struct BFSWalker<H: HyperGraph> {
    hgraph: H,
}
