use std::{collections::HashMap, marker::PhantomData, ptr::NonNull};

struct Node<N, E> {
    data: N,
    containing_edges: Vec<EdgeLink<N, E>>,
}

impl<N, E> Node<N, E> {
    pub fn new(data: N) -> Self {
        Self {
            data,
            containing_edges: Vec::new(),
        }
    }
    pub fn add_edge(&mut self, edge: EdgeLink<N, E>) {
        self.containing_edges.push(edge);
    }

    pub fn remove_edge(&mut self, edge: EdgeLink<N, E>) {
        let mut ixs_to_remove = Vec::new();
        for ix in 0..self.containing_edges.len() {
            if self.containing_edges[ix] == edge {
                ixs_to_remove.push(ix);
            }
        }
        for ix in ixs_to_remove.into_iter().rev() {
            self.containing_edges.remove(ix);
        }
    }
}

struct Edge<N, E> {
    data: E,
    nodes: Vec<NodeLink<N, E>>,
}

impl<N, E> Edge<N, E> {
    pub fn new(data: E) -> Self {
        Self {
            data,
            nodes: Vec::new(),
        }
    }

    /// Some decision to be made: check to see if the node has already been added?
    /// for now, just add it.
    pub fn add_node(&mut self, node: NodeLink<N, E>) {
        self.nodes.push(node);
    }

    /// Guaranteed to remove possibly duplicated nodes.
    pub fn remove_node(&mut self, node: NodeLink<N, E>) {
        let mut ixs_to_remove = Vec::new();
        for ix in 0..self.nodes.len() {
            if self.nodes[ix] == node {
                ixs_to_remove.push(ix);
            }
        }
        for ix in ixs_to_remove.into_iter().rev() {
            self.nodes.remove(ix);
        }
    }
}

type NodeLink<N, E> = NonNull<Node<N, E>>;
type EdgeLink<N, E> = NonNull<Edge<N, E>>;

// TODO: May be possible to get rid of this hashmap? Just return a node link when you add/delete stuff? that may be bad though?
struct HGraph<N, E> {
    nodes: HashMap<usize, NodeLink<N, E>>,
    edges: HashMap<usize, EdgeLink<N, E>>,
    next_node_id: usize,
    next_edge_id: usize,
    haunter: PhantomData<N>,
    gengar: PhantomData<E>,
}
impl<N, E> HGraph<N, E> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            next_node_id: 0,
            next_edge_id: 0,
            haunter: PhantomData,
            gengar: PhantomData,
        }
    }

    pub fn add_node(&mut self, data: N) -> usize {
        let node = Node::new(data);
        let node_box = Box::new(node);
        let node_link = unsafe { NonNull::new_unchecked(Box::into_raw(node_box)) };
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.insert(id, node_link);
        id
    }

    /// Currently allows for multiple edges, aka exactly overlapping edges
    /// Also ignores any nodes provided that may no longer be in the HGraph. This
    /// seems better than the alternative (panicking) and I don't want the API
    /// to start requiring Options all over the place.
    pub fn add_edge(&mut self, data: E, nodes: &[usize]) -> usize {
        let edge = Edge::new(data);
        let edge_box = Box::new(edge);
        let edge_link = unsafe { NonNull::new_unchecked(Box::into_raw(edge_box)) };
        for node_id in nodes.iter() {
            if let Some(node_link) = self.nodes.get_mut(node_id) {
                let node = unsafe { node_link.as_mut() };
                node.add_edge(edge_link.clone())
            }
        }
        let id = self.next_edge_id;
        self.edges.insert(id, edge_link);
        self.next_edge_id += 1;
        id
    }

    /// This will leave "dangling" edges. It is up to you, dear reader, to take care of those.
    pub fn remove_node(&mut self, node_id: usize) -> Option<N> {
        if self.nodes.contains_key(&node_id) == false {
            return None;
        }
        let mut node_link = self.nodes.remove(&node_id).unwrap();
        unsafe {
            for edge in node_link.as_mut().containing_edges.iter_mut() {
                let edge_ref = edge.as_mut();
                edge_ref.remove_node(node_link);
            }
        }
        let node = unsafe { Box::from_raw(node_link.as_ptr()) };
        return Some(node.data);
    }

    pub fn remove_edge(&mut self, edge_id: usize) -> Option<E> {
        if self.edges.contains_key(&edge_id) == false {
            return None;
        }
        let mut edge_link = self.edges.remove(&edge_id).unwrap();
        unsafe {
            for node in edge_link.as_mut().nodes.iter_mut() {
                let node_ref = node.as_mut();
                node_ref.remove_edge(edge_link);
            }
        }
        let edge = unsafe { Box::from_raw(edge_link.as_ptr()) };
        return Some(edge.data);
    }
}
