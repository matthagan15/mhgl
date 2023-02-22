use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    structs::{EdgeDirection, EdgeWeight, GeneroEdge, GeneroGraph, SparseBasis},
    traits::HgNode,
};

#[derive(Debug, Clone)]
/// A hypergraph type that simply counts nodes as they are created,
/// as opposed to HGraph which utilizes Uuid's random generation.
/// This allows for smaller data types to store nodes, which
/// theoretically can significantly reduce memory footprint for smaller,
/// denser hypergraphs. Since we use smaller integer types however, this means
/// that adding nodes could possibly fail. Due to this we also will re-use
/// previously deleted nodes. Intended more for network analytics as opposed to
/// production environments.
pub struct PGraph<N: HgNode> {
    pub name: String,
    nodes: HashSet<N>,
    reusable_nodes: Vec<N>,
    graph: GeneroGraph<SparseBasis<N>>,
}

impl<N: HgNode> PGraph<N> {
    pub fn new() -> Self {
        PGraph {
            name: "".to_string(),
            nodes: HashSet::new(),
            reusable_nodes: Vec::new(),
            graph: GeneroGraph::new(),
        }
    }

    /// May return no nodes if they cannot be created. For example, using u8 as the underlying storage method means only 255 nodes can be created. If you try adding more nodes after this then you get nothing back. Also it will reuse nodes, as this structure is intended more for analysis than production environments.
    // TODO: This is absolutely atrocious. Fix later.
    pub fn add_nodes(&mut self, num_nodes: usize) -> Option<HashSet<N>> {
        let mut ret = HashSet::with_capacity(num_nodes);
        let mut counter = N::zero();
        while ret.len() < num_nodes && counter < N::max_number() {
            if self.nodes.contains(&counter) && self.reusable_nodes.len() > 0 {
                if let Some(new_node) = self.reusable_nodes.pop() {
                    ret.insert(new_node);
                    self.nodes.insert(new_node);
                }
            } else if self.nodes.contains(&counter) && self.reusable_nodes.len() == 0 {
                counter.plus_one();
            } else if self.nodes.contains(&counter) == false {
                self.nodes.insert(counter);
                ret.insert(counter);
                counter.plus_one();
            }
        }
        if ret.len() > 0 {
            Some(ret)
        } else {
            None
        }
    }

    pub fn remove_node(&mut self, node: N) {
        if self.nodes.remove(&node) {
            let node_basis = SparseBasis::from(HashSet::from([node.clone()]));
            let edges = self.graph.get_containing_edges(&node_basis);
            for edge in edges {
                if let Some(mut old_edge) = self.graph.remove_edge(&edge) {
                    old_edge.remove_node(&node_basis);
                    self.graph.add_edge(old_edge);
                }
            }
            self.reusable_nodes.push(node);
        }
    }

    pub fn create_directed_edge(
        &mut self,
        inputs: &[N],
        outputs: &[N],
        weight: EdgeWeight,
    ) -> u128 {
        let mut e = GeneroEdge::new();
        let input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
        e.add_input_nodes(&input_basis);

        let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
        e.add_output_nodes(&output_basis);
        e.change_direction(crate::structs::EdgeDirection::Directed);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_blob(&mut self, blob: &[N], weight: EdgeWeight) -> u128 {
        let mut e = GeneroEdge::new();
        let basis = SparseBasis::from(blob.iter().cloned().collect());
        e.change_direction(EdgeDirection::Blob);
        e.add_input_nodes(&basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_loop(&mut self, nodes: &[N], weight: EdgeWeight) -> u128 {
        let mut e = GeneroEdge::new();
        let basis = SparseBasis::from(nodes.iter().cloned().collect());
        e.change_direction(EdgeDirection::Loop);
        e.add_input_nodes(&basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_undirected_edge(
        &mut self,
        inputs: &[N],
        outputs: &[N],
        weight: EdgeWeight,
    ) -> u128 {
        let mut e = GeneroEdge::new();
        let input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
        e.change_direction(EdgeDirection::Undirected);
        e.add_input_nodes(&input_basis);

        let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
        e.add_output_nodes(&output_basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn create_oriented_edge(
        &mut self,
        inputs: &[N],
        outputs: &[N],
        weight: EdgeWeight,
    ) -> u128 {
        let mut e = GeneroEdge::new();
        e.change_direction(EdgeDirection::Oriented);
        let input_basis = SparseBasis::from(inputs.into_iter().cloned().collect());
        e.add_input_nodes(&input_basis);

        let output_basis = SparseBasis::from(outputs.into_iter().cloned().collect());
        e.add_output_nodes(&output_basis);
        e.change_weight(weight);
        let id = e.id.clone();
        self.graph.add_edge(e);
        id.as_u128()
    }

    pub fn remove_edge(&mut self, edge_id: u128) {
        let id = Uuid::from_u128(edge_id);
        let e = self.graph.remove_edge(&id);
        if e.is_some() {
            for node in e.unwrap().nodes() {
                for x in node.node_set() {
                    self.nodes.remove(&x);
                }
            }
        }
    }

    /// Takes a single step in the graph, returning the subsets the given nodes map to with the weight.
    pub fn step(&self, nodes: &[N]) -> Vec<(HashSet<N>, EdgeWeight)> {
        let start_basis = SparseBasis::from(nodes.iter().cloned().collect());
        let out_vector = self.graph.map_basis(&start_basis);
        out_vector
            .to_tuples()
            .into_iter()
            .map(|(b, w)| (b.to_node_set(), w))
            .collect()
    }
}

mod test {
    use super::PGraph;

    #[test]
    fn test_node_creation_deletion() {
        let mut pg = PGraph::<u8>::new();
        let mut nodes: Vec<_> = pg.add_nodes(1000).expect("no nodes?").into_iter().collect();
        nodes.sort();
        println!("nodes! {:?}", nodes);
        assert!(pg.add_nodes(1).is_none())
    }
}
