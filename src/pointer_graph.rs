use std::{collections::HashMap, marker::PhantomData, ptr::NonNull};

use rustc_hash::{FxHashMap, FxHashSet};

struct Node<N, E> {
    id: usize,
    data: N,
    containing_edges: FxHashSet<EdgeLink<N, E>>,
}

impl<N, E> Node<N, E> {
    pub fn new(id: usize, data: N) -> Self {
        Self {
            id,
            data,
            containing_edges: FxHashSet::default(),
        }
    }
    pub fn add_edge(&mut self, edge: EdgeLink<N, E>) {
        self.containing_edges.insert(edge);
    }

    pub fn remove_edge(&mut self, edge: &EdgeLink<N, E>) {
        self.containing_edges.remove(edge);
    }
}

struct Edge<N, E> {
    id: usize,
    data: E,
    nodes: FxHashSet<NodeLink<N, E>>,
}

impl<N, E> Edge<N, E> {
    pub fn new(id: usize, data: E) -> Self {
        Self {
            id,
            data,
            nodes: FxHashSet::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Some decision to be made: check to see if the node has already been added?
    /// for now, just add it.
    pub fn add_node(&mut self, node: NodeLink<N, E>) {
        self.nodes.insert(node);
    }

    /// Guaranteed to remove possibly duplicated nodes.
    pub fn remove_node(&mut self, node: NodeLink<N, E>) {
        self.nodes.remove(&node);
    }

    pub fn contains_nodes<'a, I>(&'a self, nodes: I) -> bool
    where
        I: IntoIterator<Item = &'a NodeLink<N, E>>,
    {
        nodes
            .into_iter()
            .map(|node| self.nodes.contains(node))
            .fold(true, |acc, x| acc & x)
    }

    pub fn link(&self, nodes: &FxHashSet<NodeLink<N, E>>) -> FxHashSet<NodeLink<N, E>> {
        if self.contains_nodes(nodes) == false {
            return FxHashSet::default();
        }
        self.nodes
            .iter()
            .filter(|node| nodes.contains(&node))
            .cloned()
            .collect()
    }
}

type NodeLink<N, E> = NonNull<Node<N, E>>;
type EdgeLink<N, E> = NonNull<Edge<N, E>>;

// TODO: May be possible to get rid of this hashmap? Just return a node link when you add/delete stuff? that may be bad though?
/// An undirected hypergraph.
///
/// Add nodes with `add_node`
/// Add edges with `add_edge`
///
/// change the data for a node or edge with
/// `insert_node` and `insert_edge`
pub struct HGraph<N, E> {
    nodes: FxHashMap<usize, NodeLink<N, E>>,
    edges: FxHashMap<usize, EdgeLink<N, E>>,
    next_node_id: usize,
    next_edge_id: usize,
    haunter: PhantomData<N>,
    gengar: PhantomData<E>,
}
impl<N, E> HGraph<N, E> {
    pub fn new() -> Self {
        Self {
            nodes: FxHashMap::default(),
            edges: FxHashMap::default(),
            next_node_id: 0,
            next_edge_id: 0,
            haunter: PhantomData,
            gengar: PhantomData,
        }
    }
    pub fn len_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn len_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn add_node(&mut self, data: N) -> usize {
        let id = self.next_node_id;
        let node = Node::new(id, data);
        let node_box = Box::new(node);
        let node_link = unsafe { NonNull::new_unchecked(Box::into_raw(node_box)) };
        self.next_node_id += 1;
        self.nodes.insert(id, node_link);
        id
    }

    /// Currently allows for multiple edges, aka exactly overlapping edges
    /// Also ignores any nodes provided that may no longer be in the HGraph. This
    /// seems better than the alternative (panicking) and I don't want the API
    /// to start requiring Options all over the place.
    pub fn add_edge(&mut self, data: E, nodes: &[usize]) -> usize {
        let id = self.next_edge_id;
        let edge = Edge::new(id, data);
        let edge_box = Box::new(edge);
        let edge_link = unsafe { NonNull::new_unchecked(Box::into_raw(edge_box)) };
        for node_id in nodes.iter() {
            if let Some(node_link) = self.nodes.get_mut(node_id) {
                let node = unsafe { node_link.as_mut() };
                node.add_edge(edge_link.clone())
            }
        }
        self.edges.insert(id, edge_link);
        self.next_edge_id += 1;
        id
    }

    /// This will leave "dangling" edges. It is up to you, dear reader, to take care of those.
    pub fn remove_node(&mut self, node_id: usize) -> Option<N> {
        if self.nodes.contains_key(&node_id) == false {
            return None;
        }
        let node_link = self.nodes.remove(&node_id).unwrap();
        let node = unsafe { Box::from_raw(node_link.as_ptr()) };
        for mut edge in node.containing_edges.into_iter() {
            unsafe { edge.as_mut().remove_node(node_link) };
        }
        return Some(node.data);
    }

    pub fn remove_edge(&mut self, edge_id: usize) -> Option<E> {
        if self.edges.contains_key(&edge_id) == false {
            return None;
        }
        let edge_link = self.edges.remove(&edge_id).unwrap();
        let edge = unsafe { Box::from_raw(edge_link.as_ptr()) };
        for mut node in edge.nodes.into_iter() {
            unsafe {
                node.as_mut().containing_edges.remove(&edge_link);
            }
        }
        return Some(edge.data);
    }

    pub fn get_node(&self, node_id: usize) -> Option<&N> {
        unsafe {
            self.nodes
                .get(&node_id)
                .map(|node_link| &node_link.as_ref().data)
        }
    }

    pub fn get_node_mut(&mut self, node_id: usize) -> Option<&mut N> {
        unsafe {
            self.nodes
                .get_mut(&node_id)
                .map(|node_link| &mut node_link.as_mut().data)
        }
    }
    pub fn get_edge(&self, edge_id: usize) -> Option<&E> {
        unsafe {
            self.edges
                .get(&edge_id)
                .map(|edge_link| &edge_link.as_ref().data)
        }
    }

    pub fn get_edge_mut(&mut self, edge_id: usize) -> Option<&mut E> {
        unsafe {
            self.edges
                .get_mut(&edge_id)
                .map(|edge_link| &mut edge_link.as_mut().data)
        }
    }

    /// If the provided nodes is empty this returns an empty vector.
    pub fn get_containing_edges(&self, nodes: &[usize]) -> Vec<usize> {
        if nodes.len() == 0 {
            return Vec::new();
        }

        let first_node = self
            .nodes
            .get(&nodes[0])
            .expect("Provided node is not valid.");
        let node_link_iter = nodes.iter().filter_map(|node_id| self.nodes.get(node_id));
        unsafe {
            first_node
                .as_ref()
                .containing_edges
                .iter()
                .filter_map(|edge| {
                    if edge.as_ref().contains_nodes(node_link_iter.clone()) {
                        Some(edge.as_ref().id)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
}
