use std::collections::{HashMap, HashSet};

use polars::prelude::*;
use uuid::Uuid;

use crate::{structs::HGraphCore, EdgeSet};

pub struct KVGraph<'a> {
    core: HGraphCore<HashMap<String, AnyValue<'a>>, HashMap<String, AnyValue<'a>>, u128, u128>,
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
    pub fn add_node(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.core
            .add_node_with_id(HashMap::new(), id.clone().as_u128());
        id
    }

    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<Uuid> {
        (0..num_nodes).map(|_| self.add_node()).collect()
    }

    /// Removes a node from the node set. The deleted node will be added to a
    /// dequeue to be reused later once all possible nodes have been created.
    pub fn remove_node(&mut self, node: Uuid) {
        self.core.remove_node(node.as_u128());
    }

    /// Removes a collection of nodes. The deleted nodes will be added
    /// to a dequeue to be reused later once all possible nodes have been created
    pub fn remove_nodes(&mut self, nodes: Vec<Uuid>) {
        let _: Vec<_> = nodes.into_iter().map(|id| self.remove_node(id)).collect();
    }
    /// Creates an undirected edge among the given nodes. Duplicate input nodes are removed.
    /// Returns `None` if an edge among those nodes already exists (Duplicate edges not allowed) or
    /// if less than 2 nodes are provided.
    pub fn add_edge<E>(&mut self, nodes: E) -> Option<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        let edge: EdgeSet<u128> = EdgeSet::from(node_vec);
        if edge.len() == 1 {
            return None;
        }
        let id = Uuid::new_v4();
        self.core
            .add_edge_with_id(edge, HashMap::new(), id.clone().as_u128());
        Some(id)
    }
    pub fn remove_edge<E>(&mut self, nodes: E)
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        let e = self.core.find_id(node_vec);
        if let Some(id) = e {
            self.core.remove_edge(id);
        }
    }

    pub fn remove_edge_id(&mut self, edge_id: Uuid) {
        self.core.remove_edge(edge_id.as_u128());
    }

    /// Returns the vec of nodes associated with the edge_id.
    pub fn query_edge_id(&self, edge_id: &Uuid) -> Option<Vec<Uuid>> {
        self.core.edges.get(&edge_id.as_u128()).map(|e| {
            e.nodes
                .node_vec()
                .into_iter()
                .map(|id| Uuid::from_u128(id))
                .collect()
        })
    }

    pub fn find_id<E>(&self, nodes: E) -> Option<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core.find_id(node_vec).map(|id| Uuid::from_u128(id))
    }

    /// Computes the link of the provided set. The link of a single
    /// hyperedge is computed using the complement, so a hyperedge
    /// of nodes {a, b, c, d} and a provided `face` of {a, b} would
    /// yield a link of {c, d}. The link of the graph is then the
    /// union of all the links of each hyperedge.
    pub fn link<E>(&self, nodes: E) -> Vec<Vec<Uuid>>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .link(node_vec)
            .into_iter()
            .map(|(_, edge)| {
                let mut nodes = edge.to_node_vec();
                nodes.into_iter().map(|id| Uuid::from_u128(id)).collect()
            })
            .collect()
    }

    /// Returns the set of edge of size less than or equal to `k`,
    /// inclusive. Also note that `k` refers to the cardinality of the
    /// provided sets, not the dimension.
    pub fn k_skeleton(&self, k: usize) -> HashSet<Uuid> {
        self.core
            .edges
            .iter()
            .filter(|(_, e)| e.nodes.len() <= k)
            .map(|(id, _)| Uuid::from_u128(*id))
            .collect()
    }

    /// Warning: Has to filter all edges so takes Theta(|E|) time.
    pub fn edges_of_size(&self, card: usize) -> Vec<Uuid> {
        self.core
            .edges
            .iter()
            .filter(|(id, e)| e.nodes.len() == card)
            .map(|(id, e)| Uuid::from_u128(*id))
            .collect()
    }
    pub fn get_containing_edges<E>(&self, nodes: E) -> Vec<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .get_containing_edges_strict(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }
    pub fn get_maximal_containing_edges<E>(&self, nodes: E) -> Vec<Uuid>
    where
        E: AsRef<[Uuid]>,
    {
        let node_vec: Vec<_> = nodes
            .as_ref()
            .iter()
            .cloned()
            .map(|id| id.as_u128())
            .collect();
        self.core
            .maximal_containing_edges(node_vec)
            .into_iter()
            .map(|id| Uuid::from_u128(id))
            .collect()
    }

    /// Adds a `key`-`value` pair to the provided `id`, whether `id` correspond to
    /// a node or edge. The provided pair must match the schema associated with the
    /// hypergraph, if the `key` has not been seen before it automatically creates
    /// a new schema in the structure.
    pub fn insert<S, V>(&mut self, id: &Uuid, key: S, value: V) -> Result<(), ()>
    where
        S: ToString,
        V: Into<AnyValue<'a>>,
    {
        let id = id.as_u128();
        let key_string = key.to_string();
        let any_value: AnyValue<'a> = value.into();
        if self.schema.contains_key(&key_string) == false {
            self.schema.insert(key_string.clone(), any_value.dtype());
        } else {
            if *self.schema.get(&key_string).unwrap() != any_value.dtype() {
                return Err(());
            }
        }
        if self.core.nodes.contains_key(&id) {
            self.core
                .borrow_node_mut(&id)
                .unwrap()
                .insert(key_string, any_value);
            Ok(())
        } else if self.core.edges.contains_key(&id) {
            self.core
                .borrow_edge_mut(&id)
                .unwrap()
                .insert(key_string, any_value);
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn get(&self, id: &Uuid, key: &str) -> Option<&AnyValue<'a>> {
        let id = id.as_u128();
        if self.core.nodes.contains_key(&id) {
            let query = key.to_string();
            self.core.borrow_node(&id).unwrap().get(&query)
        } else if self.core.nodes.contains_key(&id) {
            let query = key.to_string();
            self.core.borrow_edge(&id).unwrap().get(&query)
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
            let id_string = Uuid::from_u128(*node_id).to_string();
            cols.push(Series::new("id", [id_string.clone()]));
            let mut node_string = String::from("[");
            node_string.push_str(&id_string[..]);
            node_string.push(']');
            cols.push(Series::new("nodes", [node_string]));
            let kv_store = &node_struct.data;
            for (key, value) in kv_store.iter() {
                cols.push(Series::new(&key[..], [value.clone()]));
            }
            let node_df = DataFrame::new(cols).expect("no dataframe?");
            df.vstack_mut(&node_df).expect("Could not vstack");
        }
        df
    }

    /// Clones the given id's key-value pairs.
    /// The result is wrapped in an option to help the user distinguish between an empty `id`
    /// with no key-value pairs or the id is incorrect.
    pub fn get_all_kv_pairs(&self, id: &Uuid) -> Option<Vec<(String, AnyValue<'a>)>> {
        let id = id.as_u128();
        if self.core.nodes.contains_key(&id) {
            Some(
                self.core
                    .borrow_node(&id)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect(),
            )
        } else if self.core.edges.contains_key(&id) {
            Some(
                self.core
                    .borrow_edge(&id)
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
    use polars::datatypes::AnyValue;

    use crate::KVGraph;

    #[test]
    fn create_read_update_delete() {
        let mut hg = KVGraph::new();
        let n1 = hg.add_node();
        hg.insert(&n1, "test", 1.2_f64).unwrap();
        hg.insert(&n1, "weight", 1).unwrap();
        hg.insert(&n1, "booty", AnyValue::Boolean(true)).unwrap();

        dbg!(hg.get(&n1, "test"));
        println!("{:}", hg.get_dataframe());
    }
}
