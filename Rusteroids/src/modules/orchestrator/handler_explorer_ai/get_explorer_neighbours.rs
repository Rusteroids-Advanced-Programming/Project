use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

/// Computes the list of alive neighbouring planets for the explorer's current planet
/// and sends it back over the explorer's channel.
pub fn get_explorer_neighbours_impl(orch: &Orchestrator, expl_id: ID, current_planet_id: ID) {
    // Only the sender is needed here; the request is assumed to be handled by the caller
    let (tx1, _rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    // let msg = rx1.recv().unwrap();
    let mut neighbours = Vec::new();
    // match msg {
    //     ExplorerToOrchestrator::NeighborsRequest {
    //         explorer_id: _explorer_id,
    //         current_planet_id,
    //     } => {
    // Acquire a read lock on the stats map, kept alive for the whole lookup below
    let stats_map_guard = orch.stats_map.read().unwrap();

    // Find the graph node matching the current planet
    for node in &orch.galaxy_graph.read().unwrap().nodes {
        if node.read().unwrap().value == current_planet_id {
            // Collect only the adjacent planets that are still alive
            for n in &node.read().unwrap().adjacent_nodes {
                let planet_stats = stats_map_guard.get(&n.read().unwrap().value).unwrap();
                if planet_stats.alive {
                    neighbours.push(n.read().unwrap().value);
                }
            }
            // Current planet found and processed: stop scanning the graph
            break;
        }
    }
    tx1.send(OrchestratorToExplorer::NeighborsResponse {
        neighbors: neighbours,
    })
        .unwrap();
}
// _ => {}
// }
// }
