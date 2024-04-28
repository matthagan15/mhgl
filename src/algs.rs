use crate::{EdgeSet, HyperGraph};

#[derive(Debug, Clone)]
pub enum WalkType {
    Link,
    UpDown,
    DownUp,
}

pub fn random_walk<Walker: HyperGraph>(
    hgraph: &Walker,
    start: impl AsRef<[Walker::NodeID]>,
    num_steps: usize,
    walk_type: WalkType,
) -> Option<Vec<Walker::NodeID>> {
    let start_nodes: EdgeSet<Walker::NodeID> = start.into();
    todo!()
}
