use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

pub fn start_explorer_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::StartExplorerAI).unwrap();
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
