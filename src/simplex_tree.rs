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

#[derive(Debug)]
struct NewCursorMut<'a, T> {
    simplex_tree: &'a mut SimplexTree<T>,
    prev_node: Option<u32>,
    cur_ptr: Link<T>,
}

impl<'a, T> NewCursorMut<'a, T> {
    pub fn advance(&mut self) {
        if self.cur_ptr.is_none() {
            return;
        }

        let cur_ref = unsafe { self.cur_ptr.unwrap().as_ref() };
        let cur_node = cur_ref.node;
        if self.prev_node.is_none() {
            if let Some(next_ptr) = cur_ref.containing_edges.first() {
                self.prev_node = Some(cur_node);
                self.cur_ptr = *next_ptr;
                return;
            } else {
                // have to find the next node
                if let Some((_next_node, next_ptr)) = self
                    .simplex_tree
                    .nodes
                    .range_mut(cur_node + 1..)
                    .take(1)
                    .next()
                {
                    self.cur_ptr =
                        Some(unsafe { NonNull::new_unchecked(next_ptr as *mut SimpTreeNode<T>) });
                    self.prev_node = None;
                    return;
                } else {
                    // reached the end of the line
                    self.prev_node = None;
                    self.cur_ptr = None;
                    return;
                }
            }
        }
        if cur_node > self.prev_node.unwrap() {
            // previously moved down
            if let Some(next_ptr) = cur_ref.containing_edges.first() {
                self.cur_ptr = *next_ptr;
            } else {
                // Reached a leaf node, move back up.
                self.cur_ptr = cur_ref.parent;
            }
        } else {
            // just moved up
            match cur_ref
                .containing_edges
                .binary_search_by_key(&self.prev_node.unwrap(), |x| unsafe {
                    x.unwrap().as_ref().node
                }) {
                Ok(prev_ix) => {
                    if prev_ix == cur_ref.containing_edges.len() - 1 {
                        // we just came from the last branch, need to move up.
                        self.cur_ptr = cur_ref.parent;
                    } else {
                        // can still traverse down
                        self.cur_ptr = cur_ref.containing_edges[prev_ix + 1];
                    }
                }
                Err(_) => {
                    panic!("This should not be an accessible state. The previous node visited was not found in the current nodes children, but cur_node < prev_node.")
                }
            }
        }
        self.prev_node = Some(cur_node);
    }

    pub fn print_state(&self) {
        let mut state = Vec::new();
        let mut cur_pointer = self.cur_ptr;
        if cur_pointer.is_none() {
            println!("Cursor is at the end of the line.");
            return;
        }
        let mut cur_ref = unsafe { self.cur_ptr.unwrap().as_ref() };
        while cur_ref.parent.is_some() {
            state.push(cur_ref.node);
            cur_ref = unsafe { cur_ref.parent.unwrap().as_ref() };
        }
        state.push(cur_ref.node);
        state.reverse();
        let mut s = String::from("[");
        for node in state {
            s.push_str(&node.to_string());
            s.push(',');
        }
        s.pop();
        s.push(']');
        println!("current state: {:}", s);
    }

    ///
    pub fn seek(&mut self, seek_start: impl AsRef<[u32]>) -> Vec<u32> {
        todo!()
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

    pub fn cursor_mut(&mut self) -> NewCursorMut<T> {
        let first = self
            .nodes
            .first_entry()
            .expect("Cannot create a cursor for an empty hypergraph.")
            .get_mut() as *mut SimpTreeNode<T>;
        NewCursorMut {
            simplex_tree: self,
            prev_node: None,
            cur_ptr: unsafe { Some(NonNull::new_unchecked(first)) },
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
        let mut cursor = st.cursor_mut();
        println!("nodes: {:}, {:}, {:}.", n0, n1, n2);
        cursor.print_state();
        cursor.advance();
        cursor.print_state();
        cursor.advance();
        cursor.print_state();
        cursor.advance();
        cursor.print_state();
    }
}
