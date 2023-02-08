use super::{EdgeWeight, GraphID};
use ndarray::{Array2, ShapeBuilder};
use std::collections::HashMap;
use uuid::Uuid;

struct MatrixGraph {
    id: GraphID,
    matrix: Array2<EdgeWeight>,
    num_nodes: u8,
    basis_to_index: HashMap<Vec<u8>, usize>,
}

impl MatrixGraph {
    pub fn new() -> MatrixGraph {
        MatrixGraph {
            id: Uuid::new_v4(),
            matrix: Array2::<f64>::zeros((1, 1).f()),
            num_nodes: 0,
            basis_to_index: HashMap::new(),
        }
    }
}
