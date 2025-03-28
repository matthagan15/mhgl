use std::{collections::BTreeMap, io::Write, marker::PhantomData, ptr::NonNull};

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
    nodes: BTreeMap<NodeType, SimpTreeNode<T>>,
    /// Number of nodes
    next_node: NodeType,
    _ghost: PhantomData<T>,
}

type Link<T> = Option<NonNull<SimpTreeNode<T>>>;

type NodeType = u32;

pub enum SimplexTreeError<T> {
    NodesNotPresent {
        nodes: Vec<NodeType>,
        provided_data: T,
    },
}

/// Todo: Can turn node into a vec of nodes, this would make it a radix-tree
/// type trie.
#[derive(Debug)]
struct SimpTreeNode<T> {
    /// Should only be `None` on nodes
    parent: Link<T>,
    containing_edges: Vec<Link<T>>,
    node: NodeType, // Is this the best storage format??
    data: Option<T>,
}

/// Searches a collection of links for the provided node, returning the index
/// for the first link that points to the given node.
unsafe fn search_link_pointers<T>(outbound_edges: &Vec<Link<T>>, node: NodeType) -> Option<usize> {
    for ix in 0..outbound_edges.len() {
        if let Some(ptr) = outbound_edges[ix] {
            if ptr.as_ref().node == node {
                return Some(ix);
            }
        }
    }
    None
}

#[derive(Debug)]
struct NewCursorMut<'a, T> {
    simplex_tree: &'a mut SimplexTree<T>,
    prev_node: Option<NodeType>,
    cur_ptr: Link<T>,
}

impl<'a, T> NewCursorMut<'a, T> {
    pub fn advance(&mut self) {
        println!("In advance");
        std::io::stdout().flush().unwrap();
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
            println!("hello");
            dbg!(&cur_ref.node);
            dbg!(cur_ref.containing_edges.len());
            dbg!(&cur_ref.containing_edges);
            let found_ix = cur_ref
                .containing_edges
                .binary_search_by_key(&self.prev_node.unwrap(), |x| unsafe {
                    x.unwrap().as_ref().node
                });
            match found_ix {
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

    /// Moves the pointer to the provided `next_node` if possible.
    pub fn advance_to(&mut self, next_node: NodeType) -> bool {
        if let Some(next_ix) = unsafe {
            search_link_pointers(&self.cur_ptr.unwrap().as_ref().containing_edges, next_node)
        } {
            let cur_node = unsafe { self.cur_ptr.unwrap().as_ref().node };
            self.prev_node = Some(cur_node);
            self.cur_ptr = unsafe { self.cur_ptr.unwrap().as_ref().containing_edges[next_ix] };
            true
        } else {
            false
        }
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

    /// advances the edge to the closest point in the tree. For example,
    /// a hypergraph with one edge [1, 2, 3] then seek([1,2,3,4]) would
    /// return  `(vec![1,2,3], vec![4])` and the cursor would point to the end
    /// of [1] -> [2] -> [3].
    pub fn seek(&mut self, edge: impl AsRef<[NodeType]>) -> (Vec<NodeType>, Vec<NodeType>) {
        // first check that each node is contained in the simplex tree
        let mut unprocessed_nodes: Vec<NodeType> = edge.as_ref().iter().cloned().collect();
        if unprocessed_nodes.is_empty() {
            return (vec![], vec![]);
        }
        unprocessed_nodes.sort();
        unprocessed_nodes.reverse();
        let mut processed_nodes = Vec::new();
        for node in unprocessed_nodes.iter() {
            if self.simplex_tree.nodes.contains_key(node) == false {
                panic!("Currently assumes that the simplex tree contains every node in the edge. Found a node not contained in the simplex tree")
            }
        }
        self.prev_node = None;
        let cur_node = unprocessed_nodes.pop().unwrap();
        self.cur_ptr = unsafe {
            Some(NonNull::new_unchecked(
                self.simplex_tree.nodes.get_mut(&cur_node).unwrap() as *mut SimpTreeNode<T>,
            ))
        };
        processed_nodes.push(cur_node);
        while unprocessed_nodes.len() > 0 {
            let node_to_process = unprocessed_nodes.pop().unwrap();
            if let Some(next_ix) = unsafe {
                search_link_pointers(
                    &self.cur_ptr.unwrap().as_ref().containing_edges,
                    node_to_process,
                )
            } {
                processed_nodes.push(node_to_process);
                self.prev_node = Some(node_to_process);
                self.cur_ptr = unsafe { self.cur_ptr.unwrap().as_ref().containing_edges[next_ix] };
            } else {
                unprocessed_nodes.push(node_to_process);
                break;
            }
        }
        unprocessed_nodes.reverse();
        (processed_nodes, unprocessed_nodes)
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

    pub fn add_node(&mut self, data: T) -> NodeType {
        let node_id = self.next_node;
        let new_edge = SimpTreeNode {
            parent: None,
            containing_edges: Vec::new(),
            node: node_id,
            data: Some(data),
        };
        self.nodes.insert(node_id, new_edge);
        if self.next_node == NodeType::MAX {
            panic!("Data type used to store node id's has ran out of storage. Too many nodes are present in the hypergraph. There is currently no fix to this issue, so you must make your hypergraph smaller.")
        }
        self.next_node += 1;
        node_id
    }

    pub fn add_edge(
        &mut self,
        edge: impl AsRef<[NodeType]>,
        data: T,
    ) -> Result<Option<T>, SimplexTreeError<T>> {
        let mut cursor = self.cursor_mut();
        let (found_sub_edge, mut not_found_remainder) = cursor.seek(&edge);
        let backup_not_found = not_found_remainder.clone();
        dbg!(&found_sub_edge);
        dbg!(&not_found_remainder);
        not_found_remainder.reverse();
        while !not_found_remainder.is_empty() {
            let next_up_node = not_found_remainder.pop().unwrap();
            dbg!(&next_up_node);
            let st_node = SimpTreeNode {
                parent: cursor.cur_ptr.clone(),
                containing_edges: Vec::new(),
                node: next_up_node,
                data: None,
            };
            let st_node_ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(st_node))) };
            let new_ix = unsafe {
                cursor
                    .cur_ptr
                    .unwrap()
                    .as_ref()
                    .containing_edges
                    .binary_search_by_key(&next_up_node, |link| link.unwrap().as_ref().node)
                    .expect_err(
                        "If this node was found then it should have already been processed.",
                    )
            };
            println!("Found ix: {:}", new_ix);
            unsafe {
                cursor
                    .cur_ptr
                    .unwrap()
                    .as_mut()
                    .containing_edges
                    .insert(new_ix, Some(st_node_ptr));
                dbg!("checking if node added.");
                dbg!(cursor.cur_ptr.unwrap().as_ref().containing_edges.len());
                dbg!(cursor.cur_ptr.unwrap().as_ref().node);
            };
            dbg!(cursor.advance_to(next_up_node));
            unsafe {
                dbg!(cursor.cur_ptr.unwrap().as_ref().node);
            }
            dbg!(not_found_remainder.is_empty());
        }
        let mut cursor = self.cursor_mut();
        println!("after while loop");
        let (found_sub_edge, mut not_found_remainder) = cursor.seek(&edge);
        dbg!(found_sub_edge);
        dbg!(not_found_remainder);
        unsafe {
            // cursor.seek(edge);
            dbg!("after the fact");
            dbg!(cursor.cur_ptr.unwrap().as_mut().node);
            dbg!(cursor.cur_ptr.unwrap().as_ref().containing_edges.len());
            Ok(cursor.cur_ptr.unwrap().as_mut().data.replace(data))
        }
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
        let n3 = st.add_node('d');
        let mut cursor = st.cursor_mut();
        println!("nodes: {:}, {:}, {:}.", n0, n1, n2);
        cursor.print_state();
        cursor.advance();
        cursor.print_state();
        cursor.advance();
        cursor.print_state();
        cursor.advance();
        cursor.print_state();
        let (found, not_found) = cursor.seek(vec![n0, n1, n2]);
        dbg!(found);
        dbg!(not_found);
        cursor.print_state();
        st.add_edge([n0, n1, n2, n3], 'e');
        println!("After adding edge.");
        let mut new_cursor = st.cursor_mut();
        println!("new cursor made?");
        for _ in 0..5 {
            new_cursor.advance();
            new_cursor.print_state();
        }
    }
}
