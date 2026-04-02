use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

pub fn get_explorer_neighbours_impl(orch: &Orchestrator, expl_id: ID) {
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    let msg = rx1.recv().unwrap();
    let mut neighbours = Vec::new();
    match msg {
        ExplorerToOrchestrator::NeighborsRequest {
            explorer_id: _explorer_id,
            current_planet_id,
        } => {
            for node in &orch.galaxy_graph.read().unwrap().nodes {
                if node.read().unwrap().value == current_planet_id {
                    for n in &node.read().unwrap().adjacent_nodes {
                        neighbours.push(n.read().unwrap().value);
                    }
                    break;
                }
            }
            tx1.send(OrchestratorToExplorer::NeighborsResponse {
                neighbors: neighbours,
            })
            .unwrap();
        }
        _ => {}
    }
}
