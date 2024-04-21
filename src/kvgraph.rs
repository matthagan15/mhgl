use std::collections::{HashMap, HashSet};

use polars::prelude::*;

use crate::{
    structs::{EdgeID, HGraphCore},
    EdgeSet,
};

pub struct KVGraph<'a> {
    core: HGraphCore<EdgeID, HashMap<String, AnyValue<'a>>, HashMap<String, AnyValue<'a>>>,
    schema: HashMap<String, DataType>,
}

impl<'a> KVGraph<'a> {
    pub fn new() -> Self {
        Self {
            core: HGraphCore::new(),
            schema: HashMap::new(),
        }
    }
    // TODO: Need to overhaul the add_nodes api to panic if new nodes
    // cannot be added. I also do not like the idea of reusing nodes.
    pub fn add_node(&mut self) -> EdgeID {
        self.core.add_node(HashMap::new())
    }

    /// Adds `num_nodes` nodes to the graph, returning a vector containing
    /// the nodes created. The number of nodes returned may be less than
    /// the number of nodes requested due to the use of u32 to store nodes.
    /// Nodes that get deleted are reused in a First In First Out (FIFO) format.
    // TODO: This should panic if it cannot offer the right amount of nodes.
    // Or return a Ret<Ok, Err> type. That would be the best option.
    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<EdgeID> {
        self.core
            .add_nodes((0..num_nodes).map(|_| HashMap::new()).collect())
    }

    /// Removes a node from the node set. The deleted node will be added to a
    /// dequeue to be reused later once all possible nodes have been created.
    pub fn remove_node(&mut self, node: EdgeID) {
        self.core.remove_node(node);
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: Vec<EdgeID>) {
        self.core.remove_nodes(nodes);
    }
    /// Creates an undirected edge among the given nodes. Duplicate inputs are removed. Does not allow for duplicate edges at the moment.
    pub fn add_edge<E>(&mut self, nodes: E) -> Option<EdgeID>
    where
        E: Into<EdgeSet<u64>>,
    {
        let edge: EdgeSet<u64> = nodes.into();
        if edge.len() == 1 {
            return None;
        }
        let id = self.core.add_edge(edge, HashMap::new());
        Some(id.expect("Graph busted"))
    }
    pub fn remove_edge(&mut self, nodes: &[EdgeID]) {
        let e = self.core.query_id(nodes);
        if let Some(id) = e {
            self.core.remove_edge(id);
        }
    }

    pub fn remove_edge_id(&mut self, edge_id: EdgeID) {
        self.core.remove_edge(edge_id);
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn query_edge_id(&self, edge_id: &EdgeID) -> Option<Vec<EdgeID>> {
        self.core.edges.get(edge_id).map(|e| e.nodes.node_vec())
    }

    pub fn get_edge_id(&self, nodes: &[EdgeID]) -> Option<EdgeID> {
        self.core.query_id(nodes)
    }

    /// Computes the link of the provided set. The link of a single
    /// hyperedge is computed using the complement, so a hyperedge
    /// of nodes {a, b, c, d} and a provided `face` of {a, b} would
    /// yield a link of {c, d}. The link of the graph is then the
    /// union of all the links of each hyperedge.
    pub fn link<E>(&self, nodes: E) -> Vec<EdgeSet<EdgeID>>
    where
        E: Into<EdgeSet<EdgeID>>,
    {
        self.core
            .link(nodes)
            .into_iter()
            .map(|(_, edge)| edge)
            .collect()
    }

    /// Returns the set of edge of size less than or equal to `k`,
    /// inclusive. Also note that `k` refers to the cardinality of the
    /// provided sets, not the dimension.
    pub fn k_skeleton(&self, k: usize) -> HashSet<EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= k)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    pub fn edges_of_size(&self, card: usize) -> Vec<EdgeID> {
        self.core
            .edges
            .iter()
            .filter(|(id, e)| e.nodes.len() == card)
            .map(|(id, e)| id)
            .cloned()
            .collect()
    }
    pub fn get_containing_edges<E>(&self, nodes: E) -> Vec<EdgeID>
    where
        E: Into<EdgeSet<EdgeID>>,
    {
        self.core
            .get_containing_edges_strict(nodes)
            .into_iter()
            .collect()
    }
    pub fn get_maximal_containing_edges<E>(&self, nodes: E) -> Vec<EdgeID>
    where
        E: Into<EdgeSet<EdgeID>>,
    {
        self.core.maximal_containing_edges(nodes)
    }

    /// Adds a `key`-`value` pair to the provided `id`, whether `id` correspond to
    /// a node or edge. The provided pair must match the schema associated with the
    /// hypergraph, if the `key` has not been seen before it automatically creates
    /// a new schema in the structure.
    pub fn insert<S, V>(&mut self, id: &EdgeID, key: S, value: V) -> Result<(), ()>
    where
        S: ToString,
        V: Into<AnyValue<'a>>,
    {
        let key_string = key.to_string();
        let any_value: AnyValue<'a> = value.into();
        if self.schema.contains_key(&key_string) == false {
            self.schema.insert(key_string.clone(), any_value.dtype());
        } else {
            if *self.schema.get(&key_string).unwrap() != any_value.dtype() {
                return Err(());
            }
        }
        if self.core.nodes.contains_key(id) {
            self.core
                .borrow_node_mut(id)
                .unwrap()
                .insert(key_string, any_value);
            Ok(())
        } else if self.core.edges.contains_key(id) {
            self.core
                .borrow_edge_mut(id)
                .unwrap()
                .insert(key_string, any_value);
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn get(&self, id: &EdgeID, key: &str) -> Option<&AnyValue<'a>> {
        if self.core.nodes.contains_key(id) {
            let query = key.to_string();
            self.core.borrow_node(id).unwrap().get(&query)
        } else if self.core.nodes.contains_key(id) {
            let query = key.to_string();
            self.core.borrow_edge(id).unwrap().get(&query)
        } else {
            None
        }
    }

    /// Returns a copy of the given schema being used
    pub fn get_schema(&self) -> HashMap<String, DataType> {
        self.schema.clone()
    }

    pub fn get_dataframe(&self) -> DataFrame {
        let mut df = DataFrame::default();
        for (node_id, node_struct) in self.core.nodes.iter() {
            let mut cols = Vec::new();
            cols.push(Series::new("id", [*node_id]));
            let mut node_string = String::from("[");
            node_string.push_str(&node_id.to_string());
            node_string.push(']');
            cols.push(Series::new("nodes", [node_string]));
            let kv_store = &node_struct.data;
            for (key, value) in kv_store.iter() {
                cols.push(Series::new(&key[..], [value.clone()]));
            }
        }
        todo!()
    }

    /// Clones the given id's key-value pairs.
    /// The result is wrapped in an option to help the user distinguish between an empty `id`
    /// with no key-value pairs or the id is incorrect.
    pub fn get_all_kv_pairs(&self, id: &EdgeID) -> Option<Vec<(String, AnyValue<'a>)>> {
        if self.core.nodes.contains_key(id) {
            Some(
                self.core
                    .borrow_node(id)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect(),
            )
        } else if self.core.edges.contains_key(id) {
            Some(
                self.core
                    .borrow_edge(id)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect(),
            )
        } else {
            None
        }
    }
}

mod tests {
    use crate::KVGraph;

    #[test]
    fn create_read_update_delete() {
        let mut hg = KVGraph::new();
        let n1 = hg.add_node();
        hg.insert(&n1, "test", 1.2_f64);
        dbg!(hg.get(&n1, "test"));
    }
}
