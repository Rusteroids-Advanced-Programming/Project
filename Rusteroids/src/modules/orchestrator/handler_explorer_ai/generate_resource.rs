use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::components::resource::BasicResourceType;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

/// Sends a `GenerateResourceRequest` to the explorer identified by `expl_id`
/// asking it to produce a unit of `to_generate`, then synchronously waits for
/// the response and logs the outcome. Non-matching message variants on the
/// response channel are silently ignored.
pub fn generate_resource_impl(orch: &Orchestrator, expl_id: ID, to_generate: BasicResourceType) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    println!("tx1 {:?}", tx1);
    tx1.send(OrchestratorToExplorer::GenerateResourceRequest { to_generate })
        .unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::GenerateResourceResponse {
            explorer_id: _explorer_id,
            generated,
        } => match generated {
            Ok(generated) => {
                println!(" explorer generated {:?}", generated);
            }
            Err(_err) => {
                println!(" explorer could not generate {:?}", to_generate);
            }
        },

        _ => {}
    }
}
