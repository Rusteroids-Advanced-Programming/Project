use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;
use common_game::utils::ID;

/// Requests the bag content from a specific explorer and prints it.
/// Sends a request over the explorer's channel, then blocks waiting for the matching response.
pub fn get_explorer_bag_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::BagContentRequest).unwrap();
    let _msg = rx1.recv().unwrap();
}
