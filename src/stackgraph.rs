use std::collections::HashSet;

use uuid::Uuid;

use crate::structs::EdgeDirection;
use crate::{
    structs::{ConstGenBitBasis, EdgeID, EdgeWeight, GeneroEdge, GeneroGraph},
    traits::{HgBasis, HyperGraph},
    utils::PowerSetBits,
};

#[derive(Debug)]
/// # UNDER CONSTRUCTION
/// An implementation of a HyperGraph using a binary encoding of node subsets.
/// Utilizes constant generics so the size of the graph **must** be known at compile time. As a result this type of graph will not be as widely useful as the other two. One nice upside to this decision though is that subsets of nodes can be represented as a single array of bytes that can be stored on the stack.
/// There is currently an issue where the number
/// of nodes does not match the constant provided,
/// ex: StackGraph<10> actually encodes 80 bits, the 10 specifies the number of u8's to use as the basis of the encoding. Nodes are referenced by u32's, if
/// a StackGraph<n> is created then you can use nodes from 0..8*n (exclusive).
///
/// ### Note:
/// Currently uses custom built bit field that relies on a constant generic
/// usize to determine the size of an array to be stored. Should be updated
/// with bitfields from the crate `bitvec`.
/// ## Example Usage
/// ```
/// const n: usize = 80 / 8;
/// let mut sg = StackGraph::<n>::new();
/// sg.create_edge(&[0,1,2], &[1,2,3], 1.2, EdgeDirection::Undirected);
/// sg.create_edge(&[0,1,2], &[], 1.3, EdgeDirection::Loop);
/// let v = vec![
/// (HashSet::from([1,2,3]), 1.2),
/// (HashSet::from([0,1,2]), 1.3),
/// ];
/// assert_eq!(sg.step(&[0,1,2]), v)
/// ```
pub struct StackGraph<const K: usize> {
    pub name: String,
    graph: GeneroGraph<ConstGenBitBasis<K>>,
}

// TODO: currently has to be known at compile-time and has to be specified in a weird way... not sure how to get around this.
impl<const K: usize> StackGraph<K> {
    pub fn new() -> Self {
        StackGraph {
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
            EdgeDirection::Directed | EdgeDirection::Symmetric | EdgeDirection::Oriented => {
                let e = GeneroEdge::from(input_basis, output_basis, weight, direction);
                let id = e.id.clone();
                self.graph.add_edge(e);
                id.as_u128()
            }
            EdgeDirection::Loop | EdgeDirection::Undirected => {
                input_basis.union_with(&output_basis);
                let e = GeneroEdge::from(input_basis, ConstGenBitBasis::new(), weight, direction);
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

impl<const K: usize> HyperGraph for StackGraph<K> {
    type Basis = ConstGenBitBasis<K>;
    fn edges(&self) -> Vec<crate::structs::EdgeID> {
        self.graph.clone_edges()
    }

    fn get_outbound_edges(&self, node: &Self::Basis) -> Vec<EdgeID> {
        self.graph.get_outbound_edges(node).into_iter().collect()
    }

    fn query_edges(
        &self,
        input: &Self::Basis,
        output: &Self::Basis,
    ) -> Vec<crate::structs::EdgeID> {
        self.graph.query_edges(input, output)
    }

    fn query_weight(&self, input: &Self::Basis, output: &Self::Basis) -> EdgeWeight {
        self.graph.query_weight(input, output)
    }

    fn map_basis(&self, input: &Self::Basis) -> Vec<(Self::Basis, EdgeWeight)> {
        self.graph.map_basis(input).to_tuples()
    }

    fn map_vector(
        &self,
        input: &crate::structs::GeneroVector<Self::Basis>,
    ) -> crate::structs::GeneroVector<Self::Basis> {
        self.graph.map(input)
    }
}

mod test {
    

    #[test]
    fn test_bit_graph_new() {
        const n: usize = 80 / 8;
        let sg = StackGraph::<n>::new();
        println!("sg: {:#?}", sg);
    }

    #[test]
    fn test_directed_edge_creation() {
        const n: usize = 80 / 8;
        let mut sg = StackGraph::<n>::new();
        sg.create_edge(&[0, 1, 2], &[1, 2, 3], 1., EdgeDirection::Symmetric);
        println!("sg: {:#?}", sg);
    }

    #[test]
    fn test_step() {
        const n: usize = 80 / 8;
        let mut sg = StackGraph::<n>::new();
        sg.create_edge(&[0, 1, 2], &[1, 2, 3], 1., EdgeDirection::Symmetric);
        sg.create_edge(&[0, 1, 2], &[], 1., EdgeDirection::Loop);
        println!("sg: {:#?}", sg);
        println!("step output: {:#?}", sg.step(&[0, 1, 2]));
    }
}
