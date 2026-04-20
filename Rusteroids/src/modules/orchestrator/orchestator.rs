use crate::modules::manual_explorer::bag_type::DummyBag;
use crate::modules::manual_explorer::manual_explorer::ManualExplorer;
use crate::modules::orchestrator::event_manager::ManageEvents;
use crate::modules::read_galaxy::graph::Graph;
use crate::modules::read_galaxy::stats::StatsMap;
use common_game::components::forge::Forge;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use crate::modules::explorer_utils::explorer::ExplorerBehaviour;

pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Peaceful,
}

impl Difficulty {
    pub fn get_ratio(&self) -> f64 {
        match self {
            Difficulty::Easy => 0.001,
            Difficulty::Medium => 0.005,
            Difficulty::Hard => 0.5,
            Difficulty::Peaceful => 0.0,
        }
    }
}

#[allow(dead_code)]
pub struct Orchestrator {
    pub galaxy_graph: Arc<RwLock<Graph<ID>>>,
    pub forge: Forge,
    pub stats_map: RwLock<StatsMap>,
    pub planet_channels: Arc<
        RwLock<
            HashMap<
                ID,
                (
                    Sender<OrchestratorToPlanet>,
                    Receiver<PlanetToOrchestrator>,
                    Sender<ExplorerToPlanet>,
                ),
            >,
        >,
    >,
    pub planet_threads: HashMap<ID, JoinHandle<()>>,
    pub explorer_threads: HashMap<ID, JoinHandle<()>>,
    pub explorer_channels: HashMap<
        ID,
        (
            Sender<OrchestratorToExplorer>,
            Receiver<ExplorerToOrchestrator<DummyBag>>,
            Sender<PlanetToExplorer>,
            Receiver<PlanetToExplorer>,
        ),
    >,
    pub explorer_planet: RwLock<HashMap<ID, ID>>, //aggiunto rwlock perché mi serve modificarlo per aggiornare il visualizer
    pub explorers: HashMap<ID, Arc<dyn ExplorerBehaviour>>,
    pub difficulty: Difficulty,
    pub planet_resources: HashMap<ID, (Vec<String>, Vec<String>)>, //aggiunta per Visualizer
    pub logs: RwLock<VecDeque<String>>,
    // self.explorer_channels
}

impl Orchestrator {
    pub fn new(diff: u8) -> Orchestrator {
        let diff = match diff {
            0 => Difficulty::Easy,
            1 => Difficulty::Medium,
            2 => Difficulty::Hard,
            3 => Difficulty::Peaceful,
            _ => Difficulty::Easy,
        };
        Orchestrator {
            galaxy_graph: Arc::new(RwLock::new(Graph::new())),
            stats_map: RwLock::new(StatsMap::new()),
            forge: Forge::new().unwrap(),
            planet_channels: Arc::new(RwLock::new(HashMap::new())),
            planet_threads: HashMap::new(),
            explorer_threads: HashMap::new(),
            explorer_channels: HashMap::new(),
            explorer_planet: RwLock::new(HashMap::new()),
            explorers: HashMap::new(),
            difficulty: diff,
            planet_resources: HashMap::new(),
            logs: RwLock::new(VecDeque::with_capacity(100)),
        }
    }

    pub fn add_log(&self, msg: String) {
        //log dell'orchestrator
        let mut logs = self.logs.write().unwrap();
        if logs.len() >= 50 {
            logs.pop_front();
        }
        logs.push_back(msg);
    }

    pub fn get_planet_ids_list(&self) -> Vec<ID> {
        let graph_guard = self.galaxy_graph.read().unwrap();
        graph_guard
            .nodes
            .iter()
            .filter_map(|node| {
                let guard = self.stats_map.read().unwrap();
                let node_guard = node.read().unwrap();

                if guard.contains_key(&node_guard.value) {
                    let is_stats = guard.get(&node_guard.value).unwrap();
                    if is_stats.alive {
                        Some(node_guard.value)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    //funzione per vicini, duplicato da sistemare

    pub fn get_neighbors_of(&self, planet_id: ID) -> Vec<ID> {
        let graph_guard = self.galaxy_graph.read().unwrap();
        graph_guard
            .nodes
            .iter()
            .find(|n| n.read().unwrap().value == planet_id)
            .map(|n| {
                n.read()
                    .unwrap()
                    .adjacent_nodes
                    .iter()
                    .map(|adj| adj.read().unwrap().value)
                    .collect::<Vec<ID>>()
            })
            .unwrap_or_default()
    }

    pub fn run(&self) {
        loop {
            self.manage();
            sleep(Duration::from_millis(500));
        }
    }
}
