use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

pub fn get_supported_resources_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::SupportedResourceRequest)
        .unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::SupportedResourceResult {
            explorer_id,
            supported_resources,
        } => {
            println!(
                " explorer  {} supports resources {:?}",
                explorer_id, supported_resources
            );
        }
        _ => {}
    }
}
