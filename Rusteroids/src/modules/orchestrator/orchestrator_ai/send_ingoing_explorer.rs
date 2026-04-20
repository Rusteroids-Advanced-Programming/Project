use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;
use std::time::Duration;

pub fn send_ingoing_explorer_impl(orch: &Orchestrator, planet_id: ID, explorer_id: ID) -> bool {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _expl_sender) = planet_channels_guard.get(&planet_id).unwrap();
    let tx1 = orch.explorer_channels.get(&explorer_id).unwrap().2.clone();

    let expl = orch.explorers.get(&explorer_id).unwrap();

    if ! *expl.get_base().alive.read().unwrap() {
        return false;
    }

    println!("Sending explorer #{} to {}", explorer_id, planet_id);
    sender
        .send(OrchestratorToPlanet::IncomingExplorerRequest {
            explorer_id,
            new_sender: tx1,
        })
        .unwrap();
    let msg = receiver.recv().unwrap();
    match msg {
        PlanetToOrchestrator::IncomingExplorerResponse {
            planet_id,
            explorer_id,
            res,
        } => {
            match res {
                Ok(_response) => {
                    if orch.send_outgoing_explorer(explorer_id) {
                        let (tx2, rx2, _, _) = orch.explorer_channels.get(&explorer_id).unwrap();
                        let (_, _, expl_to_planet) = planet_channels_guard.get(&planet_id).unwrap();
                        tx2.send(OrchestratorToExplorer::MoveToPlanet {
                            sender_to_new_planet: Some(expl_to_planet.clone()),
                            planet_id,
                        })
                        .unwrap();
                        let expl_resp = rx2.recv_timeout(Duration::from_millis(2000)).unwrap();
                        match expl_resp {
                            ExplorerToOrchestrator::MovedToPlanetResult {
                                explorer_id,
                                planet_id,
                            } => {
                                //inizio

                                let mut pos_guard = orch.explorer_planet.write().unwrap();
                                pos_guard.insert(explorer_id, planet_id);

                                println!("Mappa aggiornata. Stato attuale: {:?}", pos_guard);
                                drop(pos_guard);

                                //fine
                                let explorer = orch.explorers.get(&explorer_id).unwrap().clone();
                                *explorer.get_base().from_planet.write().unwrap() = Some(
                                    orch.explorer_channels.get(&explorer_id).unwrap().3.clone(),
                                );
                                *explorer.get_base().current_planet_id.write().unwrap() = planet_id;

                                orch.explorer_planet
                                    .write()
                                    .unwrap()
                                    .insert(explorer_id, planet_id);

                                // inizio
                                let neighbors = {
                                    let graph_guard = orch.galaxy_graph.read().unwrap();
                                    graph_guard
                                        .nodes
                                        .iter()
                                        .find(|n| n.read().unwrap().value == planet_id)
                                        .map(|n| {
                                            n.read()
                                                .unwrap()
                                                .adjacent_nodes
                                                .iter()
                                                .map(|adj| adj.read().unwrap().value)
                                                .collect::<Vec<ID>>()
                                        })
                                        .unwrap_or_default()
                                };
                                let (tx_to_ex, _, _, _) =
                                    orch.explorer_channels.get(&explorer_id).unwrap();
                                tx_to_ex
                                    .send(OrchestratorToExplorer::NeighborsResponse { neighbors })
                                    .unwrap();
                                // fine
                                true
                            }
                            resp => {
                                println!(
                                    "Received unexpected response while waiting for movedtoplanetresult: {:?}",
                                    resp
                                );
                                false
                            }
                        }
                    } else {
                        false
                    }
                }
                Err(e) => {
                    println!("Error while trying to move explorer: {:?}", e);
                    false
                }
            }
        }
        msg => {
            println!(
                "Received unexpected msg while waiting incoming explorer response: {:?}",
                msg
            );
            false
        }
    }
}
