use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::components::resource::ComplexResourceType;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

/// Sends a `CombineResourceRequest` to the explorer identified by `expl_id`
/// asking it to craft `to_generate`, then synchronously waits for its response
/// and logs the outcome. Non-matching message variants on the response channel
/// are silently ignored.
pub fn combine_resources_impl(orch: &Orchestrator, expl_id: ID, to_generate: ComplexResourceType) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::CombineResourceRequest { to_generate })
        .unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::CombineResourceResponse {
            explorer_id: _explorer_id,
            generated,
        } => match generated {
            Ok(_generated) => {}
            Err(_err) => {}
        },
        _ => {}
    }
}
