use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

pub fn get_supported_combinations_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::SupportedCombinationRequest)
        .unwrap();
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
        _ => {}
    }
}
