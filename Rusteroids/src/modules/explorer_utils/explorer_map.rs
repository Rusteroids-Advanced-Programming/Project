use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use common_game::utils::ID;
use crate::modules::explorer_utils::planet_infos::PlanetInfos;
use crate::modules::read_galaxy::graph::{Graph, Node};

/// The explorer's local knowledge of the galaxy: per-planet info, the discovered graph topology,
/// and the set of edges already traversed.
#[derive(Debug)]
pub struct ExplorerMap {
    pub infos: HashMap<ID, PlanetInfos>,
    pub graph: Graph<ID>,
    pub visited_edges: HashSet<(ID, ID)>,
}

impl ExplorerMap {

    /// Creates an empty map with no known planets, an empty graph and no visited edges.
    pub fn new() -> ExplorerMap {
        Self {
            infos: HashMap::new(),
            graph: Graph::new(),
            visited_edges: HashSet::new(),
        }
    }

    /// Normalizes an edge to a canonical (min, max) order so an undirected edge has a single key.
    fn get_edge_key(&self, a: ID, b: ID) -> (ID, ID) {
        if a < b { (a, b) } else { (b, a) }
    }

    /// Marks the undirected edge between `from` and `to` as visited.
    pub fn visit_edge(&mut self, from: ID, to: ID) {
        let edge = self.get_edge_key(from, to);
        self.visited_edges.insert(edge);
    }

    /// Returns whether the undirected edge between `from` and `to` has already been visited.
    pub fn is_edge_visited(&self, from: &ID, to: &ID) -> bool {
        let edge = self.get_edge_key(from.clone(), to.clone());
        self.visited_edges.contains(&edge)
    }

    /// Returns the number of distinct edges discovered so far.
    pub fn get_num_discovered_edges(&self) -> usize {
        self.visited_edges.len()
    }

    /// Returns whether the given planet has already been discovered (i.e. has stored info).
    pub fn is_planet_discovered(&self, planet_id: &ID) -> bool {
        self.infos.contains_key(planet_id)
    }

    /// Records a discovered planet: stores its info and wires it and its neighbours into the graph,
    /// creating any missing nodes and adjacency links.
    pub fn planet_discovery(
        &mut self,
        planet_id: ID,
        planet_infos: PlanetInfos,
        neighbours: Vec<ID>,
    ) {
        self.infos.insert(planet_id, planet_infos);

        let current_node: Arc<RwLock<Node<ID>>>;

        // Reuse the existing node if present, otherwise create it
        if !self.graph.is_node_in_graph(&planet_id) {
            current_node = self.graph.add_node(planet_id);
        } else {
            current_node = self.graph.get_node(&planet_id).unwrap();
        }

        for neighbour in neighbours {
            let neighbour_node: Arc<RwLock<Node<ID>>>;

            // Same get-or-create logic for each neighbour
            if !self.graph.is_node_in_graph(&neighbour) {
                neighbour_node = self.graph.add_node(neighbour);
            } else {
                neighbour_node = self.graph.get_node(&neighbour).unwrap();
            }

            // Avoid inserting duplicate graph connections.
            if !self.graph.is_adjacent_node(current_node.clone(), &neighbour) {
                self.graph.add_adj_node(&current_node, neighbour_node);
            }
        }
    }

    /// Replaces the adjacency list of an already-known planet with a fresh set of neighbours,
    /// creating any neighbour nodes that don't yet exist.
    pub fn update_neighbors(&mut self, planet_id: &ID, neighbors: &Vec<ID>) {
        // The planet is expected to already exist in the graph
        let current_node = self.graph.get_node(planet_id)
            .expect("Node not found");

        let mut new_adj = Vec::new();

        for neighbor_id in neighbors {
            // Get-or-create each neighbour node
            let neighbor_node = if !self.graph.is_node_in_graph(neighbor_id) {
                self.graph.add_node(neighbor_id.clone())
            } else {
                self.graph.get_node(neighbor_id).unwrap()
            };

            new_adj.push(neighbor_node);
        }

        // Overwrite the adjacency list wholesale (not a merge)
        let mut write_guard = current_node.write().unwrap();
        write_guard.adjacent_nodes = new_adj;
    }
}