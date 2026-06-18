use crate::modules::explorer_utils::bag_type::DummyBag;
use crate::modules::explorers::manual_explorer::manual_explorer::ManualExplorer;
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
use common_game::logging::{LogEvent,Participant,ActorType,Channel,Payload};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use crate::modules::explorer_utils::explorer::ExplorerBehaviour;

/// Defines game-wide event occurrence intervals and probability metrics.
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Peaceful,
}

impl Difficulty {
    /// Returns the exact mathematical percentage ratio used to compute random hazard events.
    pub fn get_ratio(&self) -> f64 {
        match self {
            Difficulty::Easy => 0.05,
            Difficulty::Medium => 0.1,
            Difficulty::Hard => 0.9,
            Difficulty::Peaceful => 0.0,
        }
    }
}

#[allow(dead_code)]
/// Core central controller instance mapping, managing, and synchronizing all running planet
/// and explorer backgrounds concurrent handles.
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
    pub explorer_planet: RwLock<HashMap<ID, ID>>,
    pub explorers: HashMap<ID, Arc<dyn ExplorerBehaviour>>,
    pub difficulty: Difficulty,
    pub planet_resources: HashMap<ID, (Vec<String>, Vec<String>)>,
    pub logs: RwLock<VecDeque<String>>,
    pub structured_logs: Arc<RwLock<Vec<LogEvent>>>,
}

impl Orchestrator {
    /// Constructs a clean orchestrator shell, configuring default structural tracking vectors and difficulty layers.
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
            structured_logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Appends a raw debug string entry to the internal circular log allocation queue, dropping older records.
    pub fn add_log(&self, msg: String) {
        let mut logs = self.logs.write().unwrap();

        // Enforce a strict sliding window layout boundary to prevent unbounded memory heap accumulation
        if logs.len() >= 50 {
            logs.pop_front();
        }
        logs.push_back(msg);
    }

    /// Pushes a strongly-typed structured telemetry layout packet directly to the permanent log array.
    pub fn add_structured_log(&self, event: LogEvent) {
        event.emit();
        if let Ok(mut guard) = self.structured_logs.write() {
            guard.push(event);
        }
    }

    /// Scans the entire active galaxy framework nodes returning all living planet identifiers.
    pub fn get_planet_ids_list(&self) -> Vec<ID> {
        let graph_guard = self.galaxy_graph.read().unwrap();
        graph_guard
            .nodes
            .iter()
            .filter_map(|node| {
                let guard = self.stats_map.read().unwrap();
                let node_guard = node.read().unwrap();

                // Check structural map keys to discard nodes that are dead or completely annihilated
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

    /// Queries the galaxy topological layout structure to extract immediate raw destination neighbors connected to the target.
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

    /// Launches the main orchestrator tick loop, evaluating background hazards until all planet targets collapse.
    pub fn run(&self) {
        loop {
            if !self.manage() {
                break;
            }
            sleep(Duration::from_millis(500));
        }
    }
}