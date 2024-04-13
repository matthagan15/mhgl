use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::structs::HGraphCore;


/// A generic hypergraph over (N)ode and (E)dge datatypes. 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NEGraph<NodeData, EdgeData> {
    next_usable_node: u32,
    reusable_nodes: VecDeque<u32>,
    core: HGraphCore<u32, NodeData, EdgeData>,
}

impl<NodeData, EdgeData> NEGraph<NodeData, EdgeData> {
    pub fn new() -> Self {
        Self {
            next_usable_node: 0,
            reusable_nodes: VecDeque::new(),
            core: HGraphCore::new(),
        }
    }

     // TODO: Need to overhaul the add_nodes api to panic if new nodes
    // cannot be added. I also do not like the idea of reusing nodes.
    pub fn add_node(&mut self, data: NodeData) -> u32 {
        if self.next_usable_node < u32::MAX {
            let ret = self.next_usable_node;
            self.next_usable_node += 1;
            self.core.add_node(ret, data);
            ret
        } else if self.reusable_nodes.len() > 0 {
            self.reusable_nodes.pop_front().expect("No nodes left.")
        } else {
            panic!("No nodes remaining to be added.")
        }
    }

    /// Panics if cannot add all nodes.
    /// TODO: The proper thing to do is return a 
    /// result with the OK variant being the Vec<u32> of each node added and 
    /// the Error variant containing the vec of the nodes added and 
    /// all the data that was not properly added. 
    pub fn add_nodes(&mut self, data: Vec<NodeData>) -> Vec<u32> {
        // TODO: Should the user control what nodes are present? We don't
        // really care what numbers are used to store nodes, so why go through
        // all this hassle
        let num_nodes = data.len();
        let mut data = data;
        data.reverse();
        let mut ret = Vec::with_capacity(num_nodes);
        let mut counter = self.next_usable_node;
        let mut nodes_available = counter < u32::MAX || self.reusable_nodes.len() > 0;
        while nodes_available && ret.len() < num_nodes {
            // Prefer adding never before seen nodes.
            if counter < u32::MAX {
                if self.core.nodes.contains_key(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.core.add_node(counter, data.pop().unwrap());
                    ret.push(counter);
                }
                counter += 1;
            } else {
                // If the counter has reached the max, then we start reusing nodes
                // TODO: This is rather inefficient, can just cache a boolean
                // if we already added the max value or not.
                if self.core.nodes.contains_key(&counter) == false
                    && self.reusable_nodes.contains(&counter) == false
                {
                    self.core.add_node(counter, data.pop().unwrap());
                    ret.push(counter);
                } else {
                    if let Some(old_node) = self.reusable_nodes.pop_front() {
                        if self.core.nodes.contains_key(&old_node) == false {
                            self.core.add_node(old_node, data.pop().unwrap());
                            ret.push(old_node);
                        }
                    }
                }
            }
            nodes_available = counter < u32::MAX || self.reusable_nodes.len() > 0;
        }
        self.next_usable_node = counter;
        ret
    }
}

mod tests {
    use super::NEGraph;

    #[test]
    fn test_node_creation() {
        let mut hg = NEGraph::<String, String>::new();
        hg.add_node(String::from("node 1"));
        hg.add_nodes(vec![
            String::from("node 2"),
            String::from("node 3"),
        ]);
        dbg!(hg);
    }
}