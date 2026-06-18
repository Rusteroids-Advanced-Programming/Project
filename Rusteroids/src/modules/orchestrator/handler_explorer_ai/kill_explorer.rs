use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

/// Sends a kill signal to the explorer and prints a confirmation once it acknowledges termination.
/// Sends the request over the explorer's channel, then blocks waiting for the matching response.
pub fn kill_explorer_impl(orch: &Orchestrator, expl_id: ID) {
    // Retrieve the channel tuple for this explorer; only the sender (tx1) and receiver (rx1) are needed
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::KillExplorer).unwrap();
    // Block until the explorer acknowledges the kill request
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::KillExplorerResult { explorer_id } => {
            println!("Kill explorer AI {}", explorer_id);
        }
        // Ignore any other message variant
        _ => {}
    }
}
