//!# Matt's HyperGraph Library (mhgl)
//! This crate provides a small number of hypergraph datastructures and some algorithms for working
//! with them. The datastructures provided are based on the most general possible hypergraph mathematically, that is a directed, weighted hypergraph. Directed and weighted hypergraphs are fairly different than the usually studied "undirected" hypergraph, in which case an edge is simply a subset of nodes. A directed and weighted hypergraph maps a subset of nodes to another subset of nodes with a specified edge weight.
//!
//! We provide the following three hypergraph variants:
//! 1. HGraph - Represents nodes as UUIDs that are randomly assigned. The easiest and most straightforward to use.
//! 2. PGraph<N> - A "performance" oriented version of HGraph that represents nodes as unsigned integers and is generic over which unsigned integer to use. Allows for smaller memory profiles than HGraph.
//! 3. BGraph - Represents subsets of nodes using a binary encoding. Each node is assigned to a bit, so any subset of nodes can be represented using n bits. This is advantageous for very dense hypergraphs on fewer nodes.
//!
//!
//! There are currently dense and sparse hypergraphs provided. Dense hypergraphs are based on a binary encoding of the power set of possible nodes $2^N$. This means that we represent each possible subset by a binary number over $|N|$ bits. Nodes are then represented using a 1 hot encoding, where there is only a single 1 in the binary string, it's placement indicates which node is being looked at. Due to this it is rather cumbersome to add or subtract nodes, so this behavior is not yet supported for dense graphs. The reason for density is that edges can be represented using only 2n + 128 (id) + 64 (edge weight). The sparse graph is based on each node being represented by an integer, there is currently support for the primitive unsigned types u8 through u128 to allow for varying memory profiles. For u128 supported nodes we also allow for the use of the Uuid crate, where each node is securely randomly generated. Letting k represent the number of bits used for each node, a sparse edge now takes up at most k * inbound_set_size + k * outbound_set_size + 128 (id) + 64 (edge weight) + 3 (direction enum), where inbound_set_size and outbound_set_size are simply the size of the subsets being mapped to and from. If these sizes remain much smaller than the number of nodes in the graph we see that the scaling with respect to n is much better than the dense case.
//! There are plans to support three types of
//! storage, the first being a sparse representation, the second being a more dense version utilizing a binary
//! encoding, the third being the most dense version in
//! which a single matrix is used to store the hypergraph. This library uses ndarray for matrices, as
//! it is currently the most mature and most general purpose matrix library in the ecosystem in my
//! opinion.
//!
//! # Nodes
//! This library uses the notation "node" as opposed to vertex to align with my research. The
//! rationale behind this is that "V" is preferred to represent a vector space as opposed to
//! some other letter, which makes using "N" for the node set of a hypergraph a bit easier.
//!
//! The fundamental concept of the nodes in the graph are objects that one is interested in
//! modeling that are uniquely identifiable and one is interested in behavior of groups of
//! these objects. To align with the 3 different hypergraph implementations there are currently
//! 3 ways nodes are represented (respectively) 1. 128 bit Uuids using the uuid crate 2. A single
//! "1" in a binary number (if N nodes are being used in a graph we use N bits to represent the graph,
//! so each node gets its own "spot") 3. As a specific index to the adjacency/walk matrix used for the densest hypergraphs
//! (indices may also represent subsets of nodes, so all nodes are indices not all indices are nodes).
//! There is currently no traits or other generalizations to make this easier to work with. We do
//! not yet support adding labels to nodes either, and that must be done by hand by the end user.
//!
//! # Edges
//! HyperEdges constitute the main objects of a HyperGraph. The fundamental type of edge is
//! a directed, weighted edge which maps an input subset of nodes to an output subsets of nodes
//! with an associated number. Again, there are 3 ways to represent a directed, weighted hyperedge:
//! 1. An input set, an output set, and a weight. This is preferred for sparse graphs.
//! 2. An input binary number, where each "1" indicates that node is present in the input, an output
//! binary number (similar to input), and the weight.
//! 3. As an entry in the hypgraph adjacency matrix (input index represents a set, output index represents a
//! set, and the value in the matrix the weight.)
//!
//! From these different representations of a directed, weighted hyperedge many other
//! direction types can be constructed. For example, undirected hyperedges which maps a subset in both directions,
//! oriented hyperedges in which the weight is negated going the "opposite" direction of the orientation, loops which
//! map a subset back to itself, or "blob"
//! types (which are the standard "undirected" hyperedge in the literature) which will map any subset to it's complement
//! in the "blob".
//!
//! # Graphs
//! Essentially just an organizing feature for interacting with a fixed set of nodes and multiple hyperedges.
//! There are some datastructures used to facilitate finding nodes or hyperedges easier, these are discussed
//! more in the individual structs documentation.
//!
//! # Vectors
//! Similarly to graphs "acting" on vectors, the best way to think of hypergraphs is as "acting" on vectors
//! in a much larger vector space. The simplest way to view this vector space is to consider the set of
//! all possible subsets of nodes (aka the power set of the node set N) and assign to each subset a unique
//! basis vector. This vector space is clearly 2^|N| dimensional and is organized by the cardinality of the
//! underlying node subset. For example, the vector for the empty subset {} has cardinality 0, each basis vector
//! for each node {a}, {b}, ... has cardinality 1, {a,b} cardinality 2, and so on up to the full set N with
//! cardinality |N|. Currently vectors are only supported for sparse hypergraphs, the plan is to have a generic
//! vector trait or type that can work will all of them.
//!
//! # Algorithms
//! Pretty underdeveloped at the moment. Currently only have basic builders (erdos-renyi) and traversal (random walks).
//! Plan on developing more in the future.
//!
//! # Development Guidelines
//! This crate currently forbids unsafe code, as it is designed to be used as a utility for other crates and I don't trust my own
//! or other's unsafe coding abilities enough yet. This may change if significant performance improvements can be demonstrated using
//! thoroughly vetted unsafe code. I am of the opinion that safety is a better guarantee than 2-5% speed improvements.
//!
//! Another guideline this crate tries to follow is to avoid premature generalization. I am of the opinion that code should
//! be written first, used second, and generalized third. General code is a lot more flexible, and this tends to eat into
//! ergonomics at some level. For example we do not allow nodes or edges or graphs to be generic over a "data" type. For example many
//! existing graph crates allow you to create objects `Node<N>`, where the node is generic over a data type `N`. I find this to be slightly
//! cumbersome when I don't want to store any data, and if I did it is not hard to simply add a hashmap between nodes and the data I want to store.
//! Traits are another point of generalization, and I am of the opinion that fewer traits that capture behavior better are preferred to many
//! traits that are hard to get a grasp of what they do. This crate should try to develop the objects first and then introduce traits that
//! capture shared behavior.
//!
//! # Name
//! I was initially using "graphene" for my personal project, turns out that was already used for a graph library.
//! Hyperion my second choice was also taken, and so was graphite my third. Instead of trying to come up with a
//! cool sounding name I went full 90's and just named it matts hypergraph library (mhgl).
//!
//! # TODO
//! Need to make a decision on the library API. I'm leaning towards providing
//! two types of structures: a deterministic node type where the graph simply
//! iterates through the numbers for storing nodes (i.e. create_node() would give 0 on first go, 1 on second, etc.) and another type for storing Uuids. The
//! behavior is split over these two so I'll just have to do that.

#![forbid(unsafe_code)]

use uuid::Uuid;

pub mod algs;
mod bgraph;
mod hgraph;
mod pgraph;
pub mod structs;
pub mod traits;
pub mod utils;

type HGraph8 = structs::SparseGraph<u8>;
type HGraph16 = structs::SparseGraph<u16>;
type HGraph32 = structs::SparseGraph<u32>;
type HGraph64 = structs::SparseGraph<u64>;
type HGraph128 = structs::SparseGraph<u128>;

#[cfg(test)]
mod tests {
    use crate::{structs::*, traits::HyperGraph};

    #[test]
    fn it_works() {
        let hg = SparseGraph::<u8>::new();
        println!("it works? {:#?}", hg);
    }
}
