
use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    structs::{
        bit_nodes::BitNodes, EdgeDirection, EdgeWeight, GeneroEdge, GeneroGraph,
    },
    traits::{HgBasis},
    utils::PowerSetBits,
};

#[derive(Debug)]
/// An implementation of a HyperGraph using a binary encoding of node subsets.
/// Utilizes constant generics and there is currently an issue where the number
/// of nodes does not match the constant provided.
/// Ex: BGraph<10> actually encodes 80 bits, the 10 specifies the number of u8's to use as the basis of the encoding. Nodes are referenced by u32's, if
/// a BGraph<n> is created then you can use nodes from 0..8*n (exclusive).
///
/// ## Example Usage
/// ```
/// const n: usize = 80/8;
/// let mut bg = BGraph::<n>::new();
/// bg.create_edge(&[0,1,2], &[1,2,3], 1.2, EdgeDirection::Undirected);
/// bg.create_edge(&[0,1,2], &[], 1.3, EdgeDirection::Loop);
/// let v = vec![
/// (HashSet::from([1,2,3]), 1.2),
/// (HashSet::from([0,1,2]), 1.3),
/// ];
/// assert_eq!(bg.step(&[0,1,2]), v)
/// ```
pub struct BGraph<const K: usize> {
    pub name: String,
    graph: GeneroGraph<BitNodes<K>>,
}

// TODO: currently has to be known at compile-time and has to be specified in a weird way... not sure how to get around this.
impl<const K: usize> BGraph<K> {
    pub fn new() -> Self {
        BGraph {
            name: "".to_string(),
            graph: GeneroGraph::new(),
        }
    }

    pub fn create_edge(
        &mut self,
        inputs: &[u32],
        outputs: &[u32],
        weight: EdgeWeight,
        direction: EdgeDirection,
    ) -> u128 {
        let mut pb = PowerSetBits { bits: [0; K] };
        for input in inputs {
            pb.flip_kth_bit(*input);
        }

        let mut input_basis = pb.to_bit_nodes();
        let mut pb = PowerSetBits { bits: [0; K] };
        for output in outputs {
            pb.flip_kth_bit(*output);
        }
        let output_basis = pb.to_bit_nodes();
        match direction {
            EdgeDirection::Directed | EdgeDirection::Undirected | EdgeDirection::Oriented => {
                let e = GeneroEdge::from(input_basis, output_basis, weight, direction);
                let id = e.id.clone();
                self.graph.add_edge(e);
                id.as_u128()
            }
            EdgeDirection::Loop | EdgeDirection::Blob => {
                input_basis.union_with(&output_basis);
                let e = GeneroEdge::from(input_basis, BitNodes::new(), weight, direction);
                let id = e.id.clone();
                self.graph.add_edge(e);
                id.as_u128()
            }
        }
    }

    pub fn remove_edge(&mut self, edge_id: &u128) {
        let id = Uuid::from_u128(*edge_id);
        self.graph.remove_edge(&id);
    }

    pub fn step(&self, start_nodes: &[u32]) -> Vec<(HashSet<u32>, EdgeWeight)> {
        let mut pb = PowerSetBits { bits: [0; K] };
        for node in start_nodes {
            pb.flip_kth_bit(*node);
        }
        let start_basis = pb.to_bit_nodes();
        let out = self.graph.map_basis(&start_basis);
        let v = out.to_tuples();
        v.into_iter().map(|(b, w)| (b.to_u32(), w)).collect()
    }
}

mod test {
    
    

    #[test]
    fn test_bit_graph_new() {
        const n: usize = 80 / 8;
        let bg = BGraph::<n>::new();
        println!("bg: {:#?}", bg);
    }

    #[test]
    fn test_directed_edge_creation() {
        const n: usize = 80 / 8;
        let mut bg = BGraph::<n>::new();
        bg.create_edge(&[0, 1, 2], &[1, 2, 3], 1., EdgeDirection::Undirected);
        println!("bg: {:#?}", bg);
    }

    #[test]
    fn test_step() {
        const n: usize = 80 / 8;
        let mut bg = BGraph::<n>::new();
        bg.create_edge(&[0, 1, 2], &[1, 2, 3], 1., EdgeDirection::Undirected);
        bg.create_edge(&[0, 1, 2], &[], 1., EdgeDirection::Loop);
        println!("bg: {:#?}", bg);
        println!("step output: {:#?}", bg.step(&[0, 1, 2]));
    }
}
