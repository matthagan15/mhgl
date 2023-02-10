# mhgl

## Matt's HyperGraph Library (mhgl)
A straightforward library that aims to provide a small number of well crafted hypergraph datastructures and some algorithms for working
with them. We base our datastructures on the most general possible hypergraph mathematically, that is a directed, weighted hypergraph as
oppossed to the standard undirected, unweighted hypergraph as the un-directed and un-weighted can
be constructed from the directed and weighted variant. Sometimes the prefix "hyper" will be dropped from
hypergraph and hyperedge, this is a (mostly harmless) bad habit. There are plans to support three types of
storage, the first being a sparse representation, the second being a more dense version utilizing a binary
encoding, the third being the most dense version in
which a single matrix is used to store the hypergraph. This library uses ndarray for matrices, as
it is currently the most mature and most general purpose matrix library in the ecosystem in my
opinion.

## Nodes
This library uses the notation "node" as opposed to vertex to align with my research. The
rationale behind this is that "V" is preferred to represent a vector space as opposed to
some other letter, which makes using "N" for the node set of a hypergraph a bit easier.

The fundamental concept of the nodes in the graph are objects that one is interested in
modeling that are uniquely identifiable and one is interested in behavior of groups of
these objects. To align with the 3 different hypergraph implementations there are currently
3 ways nodes are represented (respectively) 1. 128 bit Uuids using the uuid crate 2. A single
"1" in a binary number (if N nodes are being used in a graph we use N bits to represent the graph,
so each node gets its own "spot") 3. As a specific index to the adjacency/walk matrix used for the densest hypergraphs
(indices may also represent subsets of nodes, so all nodes are indices not all indices are nodes).
There is currently no traits or other generalizations to make this easier to work with. We do
not yet support adding labels to nodes either, and that must be done by hand by the end user.

## Edges
HyperEdges constitute the main objects of a HyperGraph. The fundamental type of edge is
a directed, weighted edge which maps an input subset of nodes to an output subsets of nodes
with an associated number. Again, there are 3 ways to represent a directed, weighted hyperedge:
1. An input set, an output set, and a weight. This is preferred for sparse graphs.
2. An input binary number, where each "1" indicates that node is present in the input, an output
binary number (similar to input), and the weight.
3. As an entry in the hypgraph adjacency matrix (input index represents a set, output index represents a
set, and the value in the matrix the weight.)

From these different representations of a directed, weighted hyperedge many other
direction types can be constructed. For example, undirected hyperedges which maps a subset in both directions,
oriented hyperedges in which the weight is negated going the "opposite" direction of the orientation, loops which
map a subset back to itself, or "blob"
types (which are the standard "undirected" hyperedge in the literature) which will map any subset to it's complement
in the "blob".

## Graphs
Essentially just an organizing feature for interacting with a fixed set of nodes and multiple hyperedges.
There are some datastructures used to facilitate finding nodes or hyperedges easier, these are discussed
more in the individual structs documentation.

## Vectors
Similarly to graphs "acting" on vectors, the best way to think of hypergraphs is as "acting" on vectors
in a much larger vector space. The simplest way to view this vector space is to consider the set of
all possible subsets of nodes (aka the power set of the node set N) and assign to each subset a unique
basis vector. This vector space is clearly 2^|N| dimensional and is organized by the cardinality of the
underlying node subset. For example, the vector for the empty subset {} has cardinality 0, each basis vector
for each node {a}, {b}, ... has cardinality 1, {a,b} cardinality 2, and so on up to the full set N with
cardinality |N|. Currently vectors are only supported for sparse hypergraphs, the plan is to have a generic
vector trait or type that can work will all of them.

## Algorithms
Pretty underdeveloped at the moment. Currently only have basic builders (erdos-renyi) and traversal (random walks).
Plan on developing more in the future.

## Development Guidelines
This crate currently forbids unsafe code, as it is designed to be used as a utility for other crates and I don't trust my own
or other's unsafe coding abilities enough yet. This may change if significant performance improvements can be demonstrated using
thoroughly vetted unsafe code. I am of the opinion that safety is a better guarantee than 2-5% speed improvements.

Another guideline this crate tries to follow is to avoid premature generalization. I am of the opinion that code should
be written first, used second, and generalized third. General code is a lot more flexible, and this tends to eat into
ergonomics at some level. For example we do not allow nodes or edges or graphs to be generic over a "data" type. For example many
existing graph crates allow you to create objects `Node<N>`, where the node is generic over a data type `N`. I find this to be slightly
cumbersome when I don't want to store any data, and if I did it is not hard to simply add a hashmap between nodes and the data I want to store.
Traits are another point of generalization, and I am of the opinion that fewer traits that capture behavior better are preferred to many
traits that are hard to get a grasp of what they do. This crate should try to develop the objects first and then introduce traits that
capture shared behavior.

## Name
I was initially using "graphene" for my personal project, turns out that was already used for a graph library.
Hyperion my second choice was also taken, and so was graphite my third. Instead of trying to come up with a
cool sounding name I went full 90's and just named it matts hypergraph library (mhgl).

## TODO
Need to make a decision on the library API. I'm leaning towards providing
two types of structures: a deterministic node type where the graph simply
iterates through the numbers for storing nodes (i.e. create_node() would give 0 on first go, 1 on second, etc.) and another type for storing Uuids. The
behavior is split over these two so I'll just have to do that.

License: MIT
