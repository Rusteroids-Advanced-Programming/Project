use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub adjacent_nodes: Vec<Arc<RwLock<Node<T>>>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Node<T> {
        Node {
            value,
            adjacent_nodes: Vec::new(),
        }
    }
}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        other.value == self.value
    }
}

pub struct Graph<T> {
    pub nodes: Vec<Arc<RwLock<Node<T>>>>,
}

impl<T: PartialEq> Graph<T> {
    pub fn new() -> Graph<T> {
        Graph { nodes: Vec::new() }
    }
    pub fn add_node(&mut self, value: T) -> Arc<RwLock<Node<T>>> {
        let rc = Arc::new(RwLock::new(Node::new(value)));
        self.nodes.push(rc.clone());
        rc
    }

    pub fn add_adj_node(&mut self, node: &Arc<RwLock<Node<T>>>, other: Arc<RwLock<Node<T>>>) {
        node.write().unwrap().adjacent_nodes.push(other);
    }

    pub fn remove_node(&mut self, value: T) {
        let mut i: usize = 0;
        let mut node_index: usize = 0;
        for node in &self.nodes {
            if node.read().unwrap().value == value {
                node_index = i.clone();
            } else {
                let mut j: usize = 0;
                let mut guard = node.write().unwrap();
                for adj_node in &guard.adjacent_nodes {
                    let tmp = adj_node.read().unwrap();
                    if tmp.value == value {
                        break;
                    }
                    j = j + 1;
                }
                guard.adjacent_nodes.remove(j);
            }
            i += 1;
        }
        self.nodes.remove(node_index);
    }
}

impl<T: fmt::Display> fmt::Debug for Graph<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visited = HashSet::new();

        writeln!(f, "Graph {{")?;

        for node in &self.nodes {
            let node_ptr = Arc::as_ptr(node);

            if !visited.insert(node_ptr) {
                continue;
            }

            let n = node.read().unwrap();

            let adj: Vec<String> = n
                .adjacent_nodes
                .iter()
                .map(|a| a.read().unwrap().value.to_string())
                .collect();

            writeln!(f, "  {} -> [{}]", n.value, adj.join(", "))?;
        }
        writeln!(f, "}}")
    }
}
