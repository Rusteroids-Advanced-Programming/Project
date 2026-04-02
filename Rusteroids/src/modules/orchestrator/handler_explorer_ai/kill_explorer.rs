use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

pub fn kill_explorer_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::KillExplorer).unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::KillExplorerResult { explorer_id } => {
            println!("Kill explorer AI {}", explorer_id);
        }
        _ => {}
    }
}
