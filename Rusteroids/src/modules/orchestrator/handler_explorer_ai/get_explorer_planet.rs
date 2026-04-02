use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

pub fn get_explorer_planet_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::CurrentPlanetRequest)
        .unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::CurrentPlanetResult {
            explorer_id,
            planet_id,
        } => {
            println!(" explorer  {} is in planet {}", explorer_id, planet_id);
        }
        _ => {}
    }
}
