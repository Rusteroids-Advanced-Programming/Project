use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

/// Requests the list of combinations supported by the explorer and prints it.
/// Sends the request over the explorer's channel, then blocks waiting for the matching response.
pub fn get_supported_combinations_impl(orch: &Orchestrator, expl_id: ID) {
    // Retrieve the channel tuple for this explorer; only the sender (tx1) and receiver (rx1) are needed
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::SupportedCombinationRequest)
        .unwrap();
    // Block until the explorer replies
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::SupportedCombinationResult {
            explorer_id,
            combination_list,
        } => {
            println!(
                " explorer  {} support combinations {:?}",
                explorer_id, combination_list
            );
        }
        // Ignore any other message variant
        _ => {}
    }
}
