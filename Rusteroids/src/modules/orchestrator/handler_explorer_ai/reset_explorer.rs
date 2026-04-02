use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

pub fn reset_explorer_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::ResetExplorerAI).unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id } => {
            println!("reset explorer AI {}", explorer_id);
        }
        _ => {}
    }
}
