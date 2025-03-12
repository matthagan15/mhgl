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
    nodes: BTreeMap<u32, SimpTreeNode<T>>,
    /// Number of nodes
    next_node: u32,
    _ghost: PhantomData<T>,
}

type Link<T> = Option<NonNull<SimpTreeNode<T>>>;

/// Todo: Can turn node into a vec of nodes, this would make it a radix-tree
/// type trie.
#[derive(Debug)]
struct SimpTreeNode<T> {
    /// Should only be `None` on nodes
    parent: Link<T>,
    containing_edges: Vec<Link<T>>,
    node: u32, // Is this the best storage format??
    data: Option<T>,
}

/// By Type Covariance(? Maybe Invariance) a mutable cursor should be
/// implicitly castable to an immutable one.
/// There is a "double cover" here, I need to be able to distinguish from
/// a cursor traversing down a simplex tree vs traversing up. The way to
/// distinguish these is if the cur pointer points to the same node
/// as the last node in the state, then the cursor is traversing upwards.
/// if the cur pointer points to a different node then it is traversing down.
#[derive(Debug)]
struct CursorMut<'a, T> {
    simplex_tree: &'a mut SimplexTree<T>,
    next_simp_node: Link<T>,
    /// Each node in a simplex tree corresponds to a unique edge set, this is
    /// the position of the cursor stored as a state.
    state: Vec<u32>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn state(&self) -> Vec<u32> {
        self.state.clone()
    }

    pub fn step(&mut self) {
        if self.next_simp_node.is_none() {
            panic!("Cursor is finished.")
        }
        let cur_ptr_node = unsafe { self.next_simp_node.unwrap().as_ref().node };
        if self.state.is_empty() {
            let next_ptr: Link<T> = if unsafe {
                self.next_simp_node
                    .unwrap()
                    .as_ref()
                    .containing_edges
                    .is_empty()
            } {
                if let Some((_, next_ptr)) = self
                    .simplex_tree
                    .nodes
                    .range_mut(cur_ptr_node + 1..)
                    .take(1)
                    .next()
                {
                    Some(unsafe { NonNull::new_unchecked(next_ptr as *mut SimpTreeNode<T>) })
                } else {
                    None
                }
            } else {
                *unsafe {
                    self.next_simp_node
                        .unwrap()
                        .as_ref()
                        .containing_edges
                        .first()
                        .unwrap()
                }
            };
            self.state.push(cur_ptr_node);
            self.next_simp_node = next_ptr;
        } else {
            let last_state = self.state.last().unwrap();
            let next_ptr = if *last_state > cur_ptr_node {
                // Just came from a previously traversed branch
                let ix = unsafe {
                    self.next_simp_node
                        .unwrap()
                        .as_ref()
                        .containing_edges
                        .binary_search_by_key(last_state, |x| x.unwrap().as_ref().node)
                        .unwrap()
                };
                if ix == unsafe { self.next_simp_node.unwrap().as_ref().containing_edges.len() } - 1
                {
                    // no other edges to move to
                    self.state.pop();
                    unsafe { self.next_simp_node.unwrap().as_ref().parent }
                } else {
                    self.state.push(value);
                    unsafe { self.next_simp_node.unwrap().as_ref().containing_edges[ix + 1] }
                }
            } else {
                // traversing downwards
                self.state.push(cur_ptr_node);
                todo!()
            };
        }
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
        let first = self
            .nodes
            .first_entry()
            .expect("Cannot create a cursor for an empty hypergraph.")
            .get_mut() as *mut SimpTreeNode<T>;
        CursorMut {
            simplex_tree: self,
            next_simp_node: Some(unsafe { NonNull::new_unchecked(first) }),
            state: Vec::new(),
        }
    }

    pub fn add_node(&mut self, data: T) -> u32 {
        let node_id = self.next_node;
        let new_edge = SimpTreeNode {
            parent: None,
            containing_edges: Vec::new(),
            node: node_id,
            data: Some(data),
        };
        self.nodes.insert(node_id, new_edge);
        self.next_node += 1;
        node_id
    }
}

#[cfg(test)]
mod test {
    use crate::EdgeSet;

    use super::SimplexTree;

    #[test]
    fn cursor_mut_basic_test() {
        let mut st = SimplexTree::new();
        let n0 = st.add_node('a');
        let n1 = st.add_node('b');
        let n2 = st.add_node('c');
        let cursor = st.cursor_mut();
    }
}
