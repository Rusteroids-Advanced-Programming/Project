use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
/// A thread-safe graph node encapsulating its core payload value and list of connected adjacent neighbors.
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

/// A concurrent directed graph structure utilizing shared atomic pointers and read-write locks for safe cross-thread traversal.
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

    /// Completely removes a target node from the graph allocation space, scrubbing its instances from all adjacency lists.
    pub fn remove_node(&mut self, value: T) {
        // 1. Remove the node itself from the graph's main node list
        if let Some(index) = self
            .nodes
            .iter()
            .position(|node| node.read().unwrap().value == value)
        {
            self.nodes.remove(index);
        }

        // 2. Safely remove any references to this node from all remaining nodes' adjacency lists to prevent memory leaks
        for node in &self.nodes {
            let mut guard = node.write().unwrap();
            // Retain only neighbor pointers whose underlying internal value does not match the removed payload
            guard
                .adjacent_nodes
                .retain(|adj_node| adj_node.read().unwrap().value != value);
        }
    }

    pub fn is_node_in_graph(&self, value: &T) -> bool {
        for node in &self.nodes {
            if &node.read().unwrap().value == value {
                return true;
            }
        }
        false
    }

    pub fn get_node(&self, value: &T) -> Option<Arc<RwLock<Node<T>>>> {
        for node in &self.nodes {
            if &node.read().unwrap().value == value {
                return Some(node.clone());
            }
        }
        None
    }

    pub fn is_adjacent_node(&self, current_node: Arc<RwLock<Node<T>>>, value: &T) -> bool {
        let adjacents = &current_node.read().unwrap().adjacent_nodes;
        for adj_node in adjacents {
            if &adj_node.read().unwrap().value == value {
                return true;
            }
        }
        false
    }
}

impl<T: fmt::Display> fmt::Debug for Graph<T> {
    /// Formats the layout into an adjacency list text visualization, avoiding infinitely looping on cyclic paths.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visited = HashSet::new();

        writeln!(f, "Graph {{")?;

        for node in &self.nodes {
            let node_ptr = Arc::as_ptr(node);

            // Deduplicate pointers to verify each physical heap allocation is processed exactly once
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

#[cfg(test)]
mod test_graph {
    use crate::modules::read_galaxy::graph::Graph;

    #[test]
    fn test_graph() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let _b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_adj_node(&a, c);
        graph.remove_node("B");
        assert_eq!(
            "Graph {\n  A -> [C]\n  C -> []\n}\n",
            format!("{:?}", graph)
        );
    }
}
