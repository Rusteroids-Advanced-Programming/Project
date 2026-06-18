use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;
use common_game::utils::ID;

/// Requests the explorer's current planet and prints which planet it is located on.
/// Sends the request over the explorer's channel, then blocks waiting for the matching response.
pub fn get_explorer_planet_impl(orch: &Orchestrator, expl_id: ID) {
    // Retrieve the channel tuple for this explorer; only the sender (tx1) and receiver (rx1) are needed
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::CurrentPlanetRequest)
        .unwrap();
    // Block until the explorer replies
    let _msg = rx1.recv().unwrap();
}
