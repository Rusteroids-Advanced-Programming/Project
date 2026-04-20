use std::collections::HashSet;
use std::sync::RwLock;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};

pub struct ExplorerBase {
    pub explorer_id: ID,
    pub bag: RwLock<BagType>,
    // pub dummy_bag: RwLock<DummyBag>, // aggiunto per visualizer
    pub current_planet_id: RwLock<ID>,
    pub stopped: RwLock<bool>,
    pub alive: RwLock<bool>,
    pub from_orchestrator: Receiver<OrchestratorToExplorer>,
    pub to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
    pub to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>,
    pub from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>,
    pub neighbours: RwLock<Vec<ID>>,
    pub basic_resources: RwLock<HashSet<BasicResourceType>>,
    pub combinations: RwLock<HashSet<ComplexResourceType>>,
    
}

impl ExplorerBase {
    pub fn new(
        explorer_id: ID,
        bag: RwLock<BagType>,
        current_planet_id: RwLock<ID>,
        to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
        from_orchestrator: Receiver<OrchestratorToExplorer>,
        to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>,
        from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>,
        neighbours: RwLock<Vec<ID>>,
        basic_resources: RwLock<HashSet<BasicResourceType>>,
        combinations: RwLock<HashSet<ComplexResourceType>>,
        
    ) -> Self {
        ExplorerBase {
            explorer_id,
            bag,
            current_planet_id,
            stopped: RwLock::new(false),
            alive: RwLock::new(true),
            from_orchestrator,
            to_orchestrator,
            to_planet,
            from_planet,
            neighbours,
            basic_resources,
            combinations,
        }
    }
}