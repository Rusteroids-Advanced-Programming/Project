/*use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;
use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;

pub fn send_outgoing_explorer_impl(orch: &Orchestrator, explorer_id: ID) -> bool {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&explorer_id).unwrap();
    tx1.send(OrchestratorToExplorer::CurrentPlanetRequest).unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::CurrentPlanetResult {explorer_id, planet_id} => {
            let (tx2, rx2, _ex2) = planet_channels_guard.get(&planet_id).unwrap();
            tx2.send(OrchestratorToPlanet::OutgoingExplorerRequest {explorer_id}).unwrap();
            let planet_resp = rx2.recv().unwrap();
            match planet_resp {
                PlanetToOrchestrator::OutgoingExplorerResponse {planet_id: _,explorer_id: _, res} => {
                    match res {
                        Ok(_) => {
                            true
                        }
                        Err(_err) => {
                            false
                        }
                    }
                }
                resp => {
                    println!("received unexpected msg while waiting for outgoing explorer response{:?}", resp);
                    false
                }
            }
        }
        msg => {
            println!("Received unexpected msg while waiting for current planet result: {:?}", msg);
            false
        }
    }
}
 */
use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;

pub fn send_outgoing_explorer_impl(orch: &Orchestrator, explorer_id: ID) -> bool {
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
        tx_planet
            .send(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id })
            .unwrap();

        match rx_planet.recv().unwrap() {
            PlanetToOrchestrator::OutgoingExplorerResponse { res, .. } => match res {
                Ok(_) => true,
                Err(e) => false,
            },
            resp => false,
        }
    } else {
        false
    }
}
