

use crate::structs::{EdgeID, EdgeWeight, GeneroVector};
use crate::traits::*;

//  There are currently dense and sparse hypergraphs provided. Dense hypergraphs are based on a binary encoding of the power set of possible nodes $2^N$. This means that we represent each possible subset by a binary number over $|N|$ bits. Nodes are then represented using a 1 hot encoding, where there is only a single 1 in the binary string, it's placement indicates which node is being looked at. Due to this it is rather cumbersome to add or subtract nodes, so this behavior is not yet supported for dense graphs. The reason for density is that edges can be represented using only 2n + 128 (id) + 64 (edge weight). The sparse graph is based on each node being represented by an integer, there is currently support for the primitive unsigned types u8 through u128 to allow for varying memory profiles. For u128 supported nodes we also allow for the use of the Uuid crate, where each node is securely randomly generated. Letting k represent the number of bits used for each node, a sparse edge now takes up at most k * inbound_set_size + k * outbound_set_size + 128 (id) + 64 (edge weight) + 3 (direction enum), where inbound_set_size and outbound_set_size are simply the size of the subsets being mapped to and from. If these sizes remain much smaller than the number of nodes in the graph we see that the scaling with respect to n is much better than the dense case.
/// Trait intended to capture the properties shared across HGraph, PGraph, and
/// BGraph. Currently can only return Basis structs, unclear how to make this a nicer API for the end user.
pub trait HyperGraph {
    /// The underlying basis representation, currently we have binary encoding
    /// of power sets and sparse representation where each node is saved as an
    /// unsized integer, u128's are generated with Uuid crate.
    type Basis: HgBasis;
    /// Get a list of all edges
    fn edges(&self) -> Vec<EdgeID>;

    /// Get a list of all edges leading out of a provided basis
    fn get_outbound_edges(&self, node: &Self::Basis) -> Vec<EdgeID>;
    // fn edges_with_input_cardinality(&self, cardinality: usize) -> Vec<EdgeID>;
    // fn edges_with_output_cardinality(&self, cardinality: usize) -> Vec<EdgeID>;

    /// Query for edge ID's between input and output bases.
    fn query_edges(&self, input: &Self::Basis, output: &Self::Basis) -> Vec<EdgeID>;

    /// Gets all weight from input basis to output basis, useful if multiple edges are present
    fn query_weight(&self, input: &Self::Basis, output: &Self::Basis) -> EdgeWeight;

    /// Maps an input basis to it's neighborhood in the hypergraph. The
    /// neighborhood is represented as a list of tuples
    /// (neighbor_basis, edge_weight)
    fn map_basis(&self, basis: &Self::Basis) -> Vec<(Self::Basis, EdgeWeight)>;

    /// Map a linear combination of bases to another linear combination of bases.
    fn map_vector(&self, input: &GeneroVector<Self::Basis>) -> GeneroVector<Self::Basis>;
    // fn random_basis(&self) -> SparseVector<Self::Node>;
    // fn random_step(&self, start: &mut Self::HVector);
    // fn random_basis_step(&self, start: &Self::Basis) -> Self::Basis;

    // TODO: Thoughts on a closure type hypergraph? One in which you give it a function
    // from basis elements to basis elements to the EdgeWeight type. Then when you need to
    // take a step or actually traverse the hypergraph you just call the closure. Would be
    // useful for complete hypergraphs on more than ~ 17 nodes, you essentially are trading off
    // memory for time, because if you store the hypergraph then you are fucked past 17 ish
    // nodes, similar to number of qubits.
}
