use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::components::resource::ComplexResourceType;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

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
            Ok(generated) => {
                println!(" explorer generated {:?}", generated);
            }
            Err(_err) => {
                println!(" explorer could not combine");
            }
        },
        _ => {}
    }
}
