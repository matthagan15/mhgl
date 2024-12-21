use std::{collections::BTreeMap, marker::PhantomData, ptr::NonNull};

use crate::EdgeSet;
use fxhash::FxHashMap;
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
    nodes: BTreeMap<u32, Edge<T>>,
    /// Number of nodes
    next_node: u32,
    _ghost: PhantomData<T>,
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

/// By Type Covariance(? Maybe Invariance) a mutable cursor should be
/// implicitly castable to an immutable one. This should be recognizable at
/// the compiler level.
#[derive(Debug)]
struct CursorMut<'a, T> {
    simplex_tree: &'a mut SimplexTree<T>,
    cur: Link<T>,
    /// Each node in a simplex tree corresponds to a unique edge set, this is
    /// the position of the cursor stored as a state.
    state: EdgeSet<u32>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn state(&self) -> Vec<u32> {
        self.state.node_vec()
    }

    pub fn advance(&mut self) -> Option<(&EdgeSet<u32>, &mut T)> {
        if self.state.len() == 0 && self.cur.is_some() {
            let node = unsafe { self.cur.unwrap().as_ref().node };
        }

        if self.cur.is_none() {
            println!("Cursor is pointing to None?");
            return None;
        }
        let cur_node = unsafe { self.cur.unwrap().as_ref().node };
        if cur_node == *self.state.0.last().unwrap() {
            if self.state.len() == 1 {
                // have explored all edges starting with the current node, need to find the next node.
                let mut next_node: Vec<_> = self
                    .simplex_tree
                    .nodes
                    .range_mut(cur_node + 1..)
                    .take(1)
                    .collect();
                if next_node.len() == 0 {
                    self.cur = None;
                    self.state = EdgeSet::new();
                    return None;
                } else if next_node.len() == 1 {
                    let (next_node, next_ref) = next_node.pop().unwrap();
                    self.state.pop_last();
                    self.state.add_node(*next_node);
                    debug_assert!(self.state.len() == 1);
                    debug_assert!(self.state.get_first_node() == Some(*next_node));
                    self.cur = Some(unsafe { NonNull::new_unchecked(next_ref as *mut Edge<T>) });
                }
            }
            // move up.
            self.state.pop_last();
            // if point.as_ref().parent.is
            todo!()
        }
        todo!()
    }
}

impl<T> Edge<T> {
    pub fn find_outgoing_match(&self, node: u32) -> Link<T> {
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
            nodes: BTreeMap::new(),
            next_node: 0,
            _ghost: PhantomData,
        }
    }

    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        let mut first = self
            .nodes
            .first_entry()
            .expect("Cannot create a cursor for an empty hypergraph.")
            .get_mut() as *mut Edge<T>;
        CursorMut {
            simplex_tree: self,
            cur: Some(unsafe { NonNull::new_unchecked(first) }),
            state: EdgeSet::new(),
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
        let mut remainder = edge;
        while remainder.len() > 0 {
            let (mut new_remainder, edge_ref) = self.traverse_mut(remainder.clone());
            if edge_ref.is_none() {
                return;
            }
            if new_remainder.is_empty() {
                if let Some(edge_ref_mut) = edge_ref {
                    edge_ref_mut.data = Some(data);
                }
                return;
            }
            if new_remainder.len() > 1 {
                let first_new_node = new_remainder.pop_first().unwrap();
                let mut parent_pointer =
                    unsafe { NonNull::new_unchecked(edge_ref.unwrap() as *mut Edge<T>) };
                let new_edge = unsafe {
                    NonNull::new_unchecked(Box::into_raw(Box::new(Edge {
                        parent: Some(parent_pointer.clone()),
                        containing_edges: Vec::new(),
                        node: first_new_node,
                        data: None,
                    })))
                };
                unsafe {
                    parent_pointer
                        .as_mut()
                        .containing_edges
                        .push(Some(new_edge));
                }
            } else if new_remainder.len() == 1 {
                let first_new_node = new_remainder.pop_first().unwrap();
                let mut parent_pointer =
                    unsafe { NonNull::new_unchecked(edge_ref.unwrap() as *mut Edge<T>) };
                let new_edge = unsafe {
                    NonNull::new_unchecked(Box::into_raw(Box::new(Edge {
                        parent: Some(parent_pointer.clone()),
                        containing_edges: Vec::new(),
                        node: first_new_node,
                        data: Some(data),
                    })))
                };
                unsafe {
                    parent_pointer
                        .as_mut()
                        .containing_edges
                        .push(Some(new_edge));
                }
                return;
            }
            remainder = new_remainder
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
        let mut next_edge = first_edge.unwrap().find_outgoing_match(next_node.unwrap());
        if next_edge.is_none() {
            input_edge.add_node(next_node.unwrap());
            return (input_edge, first_edge);
        }
        while next_edge.is_some() && !input_edge.is_empty() {
            if let Some(new_next_node) = input_edge.pop_first() {
                unsafe {
                    let e = next_edge
                        .unwrap()
                        .as_ref()
                        .find_outgoing_match(new_next_node);
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
        let mut next_edge = first_edge_unwrapped.find_outgoing_match(next_node.unwrap());
        if next_edge.is_none() {
            input_edge.add_node(next_node.unwrap());
            return (input_edge, Some(first_edge_unwrapped));
        }
        while next_edge.is_some() && !input_edge.is_empty() {
            if let Some(new_next_node) = input_edge.pop_first() {
                unsafe {
                    let e = next_edge
                        .unwrap()
                        .as_ref()
                        .find_outgoing_match(new_next_node);
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
        st.add_edge(EdgeSet::from([0, 1, 2]), 'c');
        let cursor = st.cursor_mut();
    }
}
