use std::{marker::PhantomData, ptr::NonNull};

use fxhash::FxHashMap;

use crate::EdgeSet;

/// First what are the operations I would like the hgraph to
/// be able to implement:
/// - add_node(data)
/// - add_edge(nodes, data)
/// - remove_node(node)
/// - remove_edge(id)
/// - find_id(nodes) -> id
/// - find_nodes(id) -> nodes
/// - link(nodes)
/// - maximal_edges(nodes)
/// - containing_edges(nodes)
/// I think I will definitely need a hashmap to lookup the nodes, that seems
/// unavoidable. What about finding edges via id?
#[derive(Debug)]
pub struct SimplexTree<T> {
    nodes: FxHashMap<u32, Edge<T>>,
    /// Number of nodes
    next_node: u32,
    _gone_ghost: PhantomData<T>,
}

type Link<T> = Option<NonNull<Edge<T>>>;

/// Todo: Can turn node into a vec of nodes, this would make it a radix-tree
/// type trie.
#[derive(Debug)]
struct Edge<T> {
    parent: Link<T>,
    containing_edges: Vec<Link<T>>,
    node: u32, // Is this the best storage format??
    data: Option<T>,
}

struct Cursor {}

impl<T> Edge<T> {
    pub fn descend_once(&self, node: u32) -> Link<T> {
        for containing_edge in self.containing_edges.iter().filter(|x| x.is_some()) {
            if let Some(edge) = containing_edge {
                unsafe {
                    let e = edge.as_ref();
                    if e.node == node {
                        return *containing_edge;
                    }
                }
            }
        }
        None
    }
}

impl<T> SimplexTree<T> {
    pub fn new() -> Self {
        Self {
            nodes: FxHashMap::default(),
            next_node: 0,
            _gone_ghost: PhantomData,
        }
    }

    pub fn add_node(&mut self, data: T) -> u32 {
        let node_id = self.next_node;
        let new_edge = Edge {
            parent: None,
            containing_edges: Vec::new(),
            node: node_id,
            data: Some(data),
        };
        self.nodes.insert(node_id, new_edge);
        self.next_node += 1;
        node_id
    }

    pub fn add_edge(&mut self, edge: EdgeSet<u32>, data: T) {
        if edge.len() < 2 {
            return;
        }
        let (remainder, last_edge) = self.traverse_mut(edge.clone());
        if remainder.is_empty() {
            if let Some(final_edge) = last_edge {
                final_edge.data = Some(data);
            } else {
            }
        }
    }

    pub fn traverse(&self, edge: EdgeSet<u32>) -> (EdgeSet<u32>, Option<&Edge<T>>) {
        if edge.is_empty() {
            return (edge, None);
        }
        let mut input_edge = edge;
        let mut next_node = input_edge.pop_first();
        let first_edge = self.nodes.get(&next_node.unwrap());
        // let mut next_edge = None;
        if first_edge.is_none() {
            input_edge.add_node(next_node.unwrap());
            return (input_edge, None);
        }
        next_node = input_edge.pop_first();
        if next_node.is_none() {
            return (input_edge, first_edge);
        }
        let mut next_edge = first_edge.unwrap().descend_once(next_node.unwrap());
        if next_edge.is_none() {
            input_edge.add_node(next_node.unwrap());
            return (input_edge, first_edge);
        }
        while next_edge.is_some() && !input_edge.is_empty() {
            if let Some(new_next_node) = input_edge.pop_first() {
                unsafe {
                    let e = next_edge.unwrap().as_ref().descend_once(new_next_node);
                    if e.is_none() {
                        input_edge.add_node(new_next_node);
                        return (input_edge, next_edge.map(|edge_ptr| edge_ptr.as_ref()));
                    } else {
                        next_edge = e;
                    }
                }
            }
        }
        return (
            input_edge,
            next_edge.map(|edge_ptr| unsafe { edge_ptr.as_ref() }),
        );
    }

    pub fn traverse_mut(&mut self, edge: EdgeSet<u32>) -> (EdgeSet<u32>, Option<&mut Edge<T>>) {
        if edge.is_empty() {
            return (edge, None);
        }
        let mut input_edge = edge;
        let mut next_node = input_edge.pop_first();
        let first_edge = self.nodes.get_mut(&next_node.unwrap());
        // let mut next_edge = None;
        if first_edge.is_none() {
            input_edge.add_node(next_node.unwrap());
            return (input_edge, None);
        }
        next_node = input_edge.pop_first();
        if next_node.is_none() {
            return (input_edge, first_edge);
        }
        let first_edge_unwrapped = first_edge.unwrap();
        let mut next_edge = first_edge_unwrapped.descend_once(next_node.unwrap());
        if next_edge.is_none() {
            input_edge.add_node(next_node.unwrap());
            return (input_edge, Some(first_edge_unwrapped));
        }
        while next_edge.is_some() && !input_edge.is_empty() {
            if let Some(new_next_node) = input_edge.pop_first() {
                unsafe {
                    let e = next_edge.unwrap().as_ref().descend_once(new_next_node);
                    if e.is_none() {
                        input_edge.add_node(new_next_node);
                        return (input_edge, next_edge.map(|mut edge_ptr| edge_ptr.as_mut()));
                    } else {
                        next_edge = e;
                    }
                }
            }
        }
        return (
            input_edge,
            next_edge.map(|mut edge_ptr| unsafe { edge_ptr.as_mut() }),
        );
    }

    pub fn find_edge(&self, edge: impl AsRef<[u32]>) -> Option<&T> {
        let sorted_edge = EdgeSet::from(edge.as_ref());
        if let Some(node) = sorted_edge.get_first_node() {
            let edges = self.nodes.get(&node);
            todo!()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::EdgeSet;

    use super::SimplexTree;

    #[test]
    fn creation_annihilation() {
        let mut st = SimplexTree::new();
        let n0 = st.add_node('a');
        let n1 = st.add_node('b');
        dbg!(st.traverse(EdgeSet::from([0, 1, 2, 3, 4])));
        dbg!(st.traverse(EdgeSet::from([1, 2, 3, 4])));
    }
}
