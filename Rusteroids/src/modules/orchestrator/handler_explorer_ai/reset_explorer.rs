use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

/// Sends a reset signal to the explorer's AI and prints a confirmation once it acknowledges.
/// Sends the request over the explorer's channel, then blocks waiting for the matching response.
pub fn reset_explorer_impl(orch: &Orchestrator, expl_id: ID) {
    // Retrieve the channel tuple for this explorer; only the sender (tx1) and receiver (rx1) are needed
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::ResetExplorerAI).unwrap();
    // Block until the explorer acknowledges the reset
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id } => {
            println!("reset explorer AI {}", explorer_id);
        }
        _ => {}
    }
}
