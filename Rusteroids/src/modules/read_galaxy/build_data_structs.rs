use crate::modules::read_galaxy::graph::{Graph, Node};
use crate::modules::read_galaxy::read_galaxy_file::read_galaxy_file;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub fn build_galaxy_graph() -> Graph<u32> {
    let mut nodes_map: HashMap<u32, Arc<RwLock<Node<u32>>>> = HashMap::new();

    let mut graph = Graph::new();
    for line in read_galaxy_file().unwrap() {
        let node = graph.add_node(line[0]);
        nodes_map.insert(line[0], node);
    }

    for line in read_galaxy_file().unwrap() {
        let node = nodes_map.get(&line[0]).unwrap();
        let mut line_iter = line.into_iter().skip(1);
        for word in &mut line_iter {
            graph.add_adj_node(node, nodes_map.get(&word).unwrap().clone());
        }
    }

    graph
}
