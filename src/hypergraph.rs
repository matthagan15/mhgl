use crate::HgNode;

/// The connectivity features of a hypergraph, used for developing algorithms
/// in a struct independent way.
pub trait HyperGraph {
    type NodeID: HgNode;
    type EdgeID: HgNode;

    /// Retrieve the nodes associated with the given `EdgeID`
    fn query_edge(&self, edge: &Self::EdgeID) -> Option<Vec<Self::NodeID>>;

    /// Find all edge ids such that the given nodes are a subset or equal to
    /// the edge.
    fn containing_edges_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID>;

    /// Find all edges such that the nodes of a given edge are a **strict** subset of.
    fn containing_edges(&self, edge: &Self::EdgeID) -> Vec<Self::EdgeID>;

    /// Computes the link of the provided nodes by pairs of edge ids and what
    /// the link of the provided nodes are within the associated id.
    /// Ex: If the graph has edges {1, 2, 3}, {2, 3, 4}, {3, 4, 5}, and {2, 3} with
    /// ids 1,2, 3, and 4 respectively, then the link of edge_id = 4 would be
    /// vec![(1, [1]), (2, [2])].
    fn link(&self, edge: &Self::EdgeID) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)>;

    /// Computes the link of the provided nodes by pairs of edge ids and what
    /// the link of the provided nodes are within the associated id.
    /// Ex: If the graph has edges {1, 2, 3}, {2, 3, 4}, and {3, 4, 5}, with
    /// ids 1,2, and 3 respectively, then the link of [3] would be
    /// vec![(1, [1, 2]), (2, [2, 4]), (3, [4, 5])].
    fn link_of_nodes(
        &self,
        nodes: impl AsRef<[Self::NodeID]>,
    ) -> Vec<(Self::EdgeID, Vec<Self::NodeID>)>;

    /// Finds the edges containing the edge associated with the provided
    /// ID that are not contained in any other edge. If the edge of the
    /// provided ID is maximal, it is not included in its return.
    /// Ex: {1, 2, 3}, {1,2, 3, 4}, {1, 2, 3, 4, 5} and you give the id
    /// of {1, 2, 3}, then the id of {1, 2, 3, 4, 5} will be returned.
    fn maximal_edges(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID>;

    /// Finds all edges containing provided nodes that are not contained
    /// in any other edge. If the provided nodes are a maximal edge, then
    /// that edges ID is returned.
    fn maximal_edges_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID>;

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    fn edges_of_size(&self, card: usize) -> Vec<Self::EdgeID>;

    /// Returns the edges that have cardinality less than or equal to the input `cardinality`. Takes Theta(|E|) time.
    fn skeleton(&self, cardinality: usize) -> Vec<Self::EdgeID>;

    /// Returns edges that constitute the boundary up operator, which
    /// adds a single node to the provided edge.
    /// Example: If a graph has edges {1, 2}, {1,2, 3}, {1,2,4}, and {1, 2, 3, 4} with ids 1, 2, 3, and 4 respectively, then `boundary_up(1)` would give
    /// vec![2, 3].
    fn boundary_up(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID>;

    /// Finds the edges that are the same as the provided edge_id but
    /// have a single node removed. For example, {1, 2} would be in
    /// boundary_down of {1, 2, 3} if both edges were present.
    /// Returns an empty vec if the edge_id is incorrect.
    fn boundary_down(&self, edge_id: &Self::EdgeID) -> Vec<Self::EdgeID>;

    /// Finds all edges which contain one more node than the provided
    /// node.
    fn boundary_up_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID>;

    /// Finds all edges that have one node removed from the provided nodes.
    fn boundary_down_of_nodes(&self, nodes: impl AsRef<[Self::NodeID]>) -> Vec<Self::EdgeID>;
}
