use uuid::Uuid;

use crate::traits::HgBasis;

use super::{EdgeID, EdgeWeight, EdgeDirection};

pub struct GeneroEdge<B: HgBasis> {
    pub id: EdgeID,
    weight: EdgeWeight,
    in_nodes: B,
    out_nodes: B,
    direction: EdgeDirection,
}

impl<B: HgBasis> GeneroEdge<B> {
    pub fn new() -> Self {
        GeneroEdge { id: Uuid::new_v4(), weight: 1., in_nodes: B::new_empty(), out_nodes: B::new_empty(), direction: EdgeDirection::Directed }
    }

    // TODO: This currently trusts the user way too much, what if we give the same nodes for in and out but specify the direction as Blob? Need to do some basic checks first.
    pub fn from(in_nodes: B, out_nodes: B, weight: EdgeWeight, edge_type: EdgeDirection) -> Self {
        GeneroEdge { id: Uuid::new_v4(), weight: weight, in_nodes: in_nodes, out_nodes: out_nodes, direction: edge_type }
    }

    
}