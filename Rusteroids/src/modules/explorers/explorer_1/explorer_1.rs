use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use common_game::components::resource::{BasicResource, BasicResourceType, ComplexResource, ComplexResourceType, GenericResource};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use crate::modules::explorer_utils::explorer_base::ExplorerBase;
use crate::modules::explorer_utils::handlers::AIHandlers;
use crate::modules::explorer_utils::tasks::Task;
use crate::modules::explorers::explorer_1::tasks::visit_all_planet::TotalPlanetsVisitedTask;
use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};

pub struct Explorer1 {
    pub base: ExplorerBase,
    pub tot_visits_task: RwLock<TotalPlanetsVisitedTask>
}

impl AIHandlers for Explorer1 {
    fn start_ai_handler(&self) {}

    fn reset_ai_handler(&self) {}

    fn kill_handler(&self) {}

    fn generate_resource_handler(&self, result: &Option<&BasicResource>) {}

    fn combine_resource_handler(&self, result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>) {}

    fn move_to_planet_handler(&self) {
        self.tot_visits_task.write().unwrap().update_progress()
    }
}
