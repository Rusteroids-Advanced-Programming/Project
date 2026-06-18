use crate::modules::orchestrator::orchestrator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;
use std::time::Duration;

/// Executes the cross-thread routing protocol to transfer an explorer from its current location onto a target planet.
pub fn send_ingoing_explorer_impl(orch: &Orchestrator, planet_id: ID, explorer_id: ID) -> bool {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _expl_sender) = planet_channels_guard.get(&planet_id).unwrap();
    let tx1 = orch.explorer_channels.get(&explorer_id).unwrap().2.clone();

    let expl = orch.explorers.get(&explorer_id).unwrap();

    // Sanity check to abort immediately if the explorer instance has been terminated
    if !*expl.get_base().alive.read().unwrap() {
        return false;
    }

    println!("Sending explorer #{} to {}", explorer_id, planet_id);

    // Handshake Step 1: Request the target planet to register the incoming explorer thread and link its sender clone
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
                    // Handshake Step 2: Inform the previous location/planet that the explorer is departing
                    if orch.send_outgoing_explorer(explorer_id) {
                        let (tx2, rx2, _, _) = orch.explorer_channels.get(&explorer_id).unwrap();
                        let (_, _, expl_to_planet) = planet_channels_guard.get(&planet_id).unwrap();

                        // Handshake Step 3: Command the explorer thread to update its active outbound planet-channel link
                        tx2.send(OrchestratorToExplorer::MoveToPlanet {
                            sender_to_new_planet: Some(expl_to_planet.clone()),
                            planet_id,
                        })
                        .unwrap();

                        // Wait with a safety timeout for the explorer thread to acknowledge the movement transition
                        let expl_resp = rx2.recv_timeout(Duration::from_millis(2000)).unwrap();
                        match expl_resp {
                            ExplorerToOrchestrator::MovedToPlanetResult {
                                explorer_id,
                                planet_id,
                            } => {
                                let mut pos_guard = orch.explorer_planet.write().unwrap();
                                pos_guard.insert(explorer_id, planet_id);

                                drop(pos_guard);

                                // Update local architectural states inside the global tracker map and the explorer structure
                                let explorer = orch.explorers.get(&explorer_id).unwrap().clone();
                                *explorer.get_base().from_planet.write().unwrap() = Some(
                                    orch.explorer_channels.get(&explorer_id).unwrap().3.clone(),
                                );
                                *explorer.get_base().current_planet_id.write().unwrap() = planet_id;

                                orch.explorer_planet
                                    .write()
                                    .unwrap()
                                    .insert(explorer_id, planet_id);

                                // Topology scan: Query the static layout graph to discover all raw neighbor pathways for the landing node
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

                                // Automatically transmit neighboring topological references right into the landing explorer
                                let (tx_to_ex, _, _, _) =
                                    orch.explorer_channels.get(&explorer_id).unwrap();
                                tx_to_ex
                                    .send(OrchestratorToExplorer::NeighborsResponse { neighbors })
                                    .unwrap();

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
                    let mut payload = Payload::new();
                    payload.insert("Error while moving explorer".into(), format!("{:?}", e));

                    orch.add_structured_log(LogEvent::new(
                        Some(Participant::new(ActorType::Orchestrator, 0u32)),
                        None,
                        EventType::InternalOrchestratorAction,
                        Channel::Error,
                        payload,
                    ));

                    false
                }
            }
        }
        msg => {
            let mut payload = Payload::new();
            payload.insert(
                "Received unexpected msg while waiting incoming explorer response".into(),
                format!("{:?}", msg),
            );

            orch.add_structured_log(LogEvent::new(
                Some(Participant::new(ActorType::Orchestrator, 0u32)),
                None,
                EventType::InternalOrchestratorAction,
                Channel::Error,
                payload,
            ));

            false
        }
    }
}
