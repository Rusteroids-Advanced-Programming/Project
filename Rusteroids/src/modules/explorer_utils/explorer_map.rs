use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use common_game::utils::ID;
use crate::modules::explorer_utils::planet_infos::PlanetInfos;
use crate::modules::read_galaxy::graph::{Graph, Node};

#[derive(Debug)]
pub struct ExplorerMap{
    pub infos: HashMap<ID, PlanetInfos>,
    pub graph: Graph<ID>
}

impl ExplorerMap{
    pub fn new() -> ExplorerMap {
        Self{infos: HashMap::new(), graph: Graph::new()}
    }

    pub fn is_planet_discovered(&self, planet_id: &ID) -> bool {
        self.infos.contains_key(planet_id)
    }

    pub fn planet_discovery(&mut self, planet_id: ID, planet_infos: PlanetInfos, neighbours: Vec<ID>) {
        self.infos.insert(planet_id, planet_infos);

        let current_node: Arc<RwLock<Node<ID>>>;

        if !self.graph.is_node_in_graph(&planet_id) {
            current_node = self.graph.add_node(planet_id);
        }

        else {
            current_node = self.graph.get_node(&planet_id).unwrap();
        }

        for neighbour in neighbours {
            let neighbour_node: Arc<RwLock<Node<ID>>>;

            if !self.graph.is_node_in_graph(&neighbour) {
                neighbour_node = self.graph.add_node(neighbour);
            }
            else {
                neighbour_node = self.graph.get_node(&neighbour).unwrap();
            }

            if !self.graph.is_adjacent_node(current_node.clone(), &neighbour) {
                self.graph.add_adj_node(&current_node, neighbour_node);
            }
        }


    }
}
