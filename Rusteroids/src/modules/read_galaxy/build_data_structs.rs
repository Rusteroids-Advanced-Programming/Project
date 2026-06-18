use crate::modules::read_galaxy::graph::{Graph, Node};
use crate::modules::read_galaxy::read_galaxy_file::read_galaxy_file;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Parses raw galaxy layout configuration files to assemble a thread-safe static topological Graph map.
pub fn build_galaxy_graph() -> Graph<u32> {
    let mut nodes_map: HashMap<u32, Arc<RwLock<Node<u32>>>> = HashMap::new();
    let mut graph = Graph::new();

    // First Pass: Extract and instantiate unique raw node objects to register them into the lookup registry map
    for line in read_galaxy_file().unwrap() {
        let node = graph.add_node(line[0]);
        nodes_map.insert(line[0], node);
    }

    // Second Pass: Iterate back through the configuration rows to safely stitch and link mutual adjacent neighbors
    for line in read_galaxy_file().unwrap() {
        let node = nodes_map.get(&line[0]).unwrap();
        let mut line_iter = line.into_iter().skip(1);

        for word in &mut line_iter {
            // Retrieve the allocated destination pointer and append it to the active origin's adjacency list
            graph.add_adj_node(node, nodes_map.get(&word).unwrap().clone());
        }
    }

    graph
}
