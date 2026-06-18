use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;

/// Coordinates the removal sequence of an explorer from its current hosting planet.
pub fn send_outgoing_explorer_impl(orch: &Orchestrator, explorer_id: ID) -> bool {
    // Isolated lookup block to retrieve the planet ID where the explorer is currently registered
    let planet_id = {
        let pos_guard = orch.explorer_planet.read().unwrap();
        match pos_guard.get(&explorer_id) {
            Some(&id) => id,
            None => {
                return false;
            }
        }
    };

    let planet_channels_guard = orch.planet_channels.read().unwrap();
    if let Some((tx_planet, rx_planet, _)) = planet_channels_guard.get(&planet_id) {
        // Issue a synchronous request asking the planet thread to clear the explorer instance
        tx_planet
            .send(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id })
            .unwrap();

        // Evaluate the runtime response payload returned from the planet actor
        match rx_planet.recv().unwrap() {
            PlanetToOrchestrator::OutgoingExplorerResponse { res, .. } => match res {
                Ok(_) => true,
                Err(e) => false,
            },
            _resp => false,
        }
    } else {
        false
    }
}