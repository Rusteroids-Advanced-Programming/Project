//! Defines [`ExplorerBase`]: the shared state and communication channels
//! every explorer implementation is built on top of (see the [`Explorer`]
//! and [`ExplorerAI`] traits, which operate on this struct).

use crate::modules::explorer_utils::bag_type::{BagType, DummyBag};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashSet;
use std::sync::RwLock;

/// Holds an explorer's identity, resource bag, current planet/connection
/// state, and the channels used to talk to the orchestrator and to
/// whichever planet it's currently on (`to_planet`/`from_planet` are
/// `None` while not connected to any planet).
pub struct ExplorerBase {
    pub explorer_id: ID,
    pub bag: RwLock<BagType>,
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
    /// Builds a new `ExplorerBase`. `stopped` and `alive` aren't taken as
    /// parameters: every new explorer starts as not stopped and alive.
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
