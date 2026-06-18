use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

/// Sends a start signal to the explorer's AI and prints a confirmation once it acknowledges.
/// Sends the request over the explorer's channel, then blocks waiting for the matching response.
pub fn start_explorer_impl(orch: &Orchestrator, expl_id: ID) {
    // Retrieve the channel tuple for this explorer; only the sender (tx1) and receiver (rx1) are needed
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::StartExplorerAI).unwrap();
    // Block until the explorer acknowledges the start
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::StartExplorerAIResult { explorer_id } => {
            println!("Start explorer AI {}", explorer_id);
        }
        msg => {
            println!(
                "Unexpected message while waiting for start explorer AI: {:?}",
                msg
            );
        }
    }
}
