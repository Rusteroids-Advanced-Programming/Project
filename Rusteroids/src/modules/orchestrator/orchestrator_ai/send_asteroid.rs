use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use crate::modules::read_galaxy::stats::Counts;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;
use std::fmt::format;
use common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;
use common_game::logging::{LogEvent, Participant, ActorType, EventType, Channel, Payload};

/// Simulates sending an asteroid to a target planet, handling defense reactions (rockets) or catastrophic failure/annihilation.
pub fn send_asteroid_impl(orch: &Orchestrator, target: ID) {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _expl_sender) = &*planet_channels_guard.get(&target).unwrap();

    // Generate and dispatch an asteroid payload directly to the planet's simulation receiver
    sender
        .send(OrchestratorToPlanet::Asteroid(
            orch.forge.generate_asteroid(),
        ))
        .unwrap();

    let mut map_guard = orch.stats_map.write().unwrap();
    map_guard.increase_count(target, Counts::Asteroids);

    // Block on receiving the planet's diagnostic response to the asteroid event
    let msg = receiver.recv().unwrap();
    match msg {
        PlanetToOrchestrator::AsteroidAck { planet_id, rocket } => {
            match rocket {
                Some(_rocket) => {
                    let log_msg = format!("AsteroidAck from Planet #{} with rocket", planet_id);
                    orch.add_log(log_msg.clone());
                    map_guard.increase_count(target, Counts::Rockets);

                    let mut payload = Payload::new();
                    payload.insert("message".into(), format!("Asteroide distrutto dal pianeta #{}.", planet_id));

                    orch.add_structured_log(LogEvent::new(
                        Some(Participant::new(ActorType::Planet, planet_id)),
                        None,
                        EventType::InternalPlanetAction,
                        Channel::Debug,
                        payload,
                    ));
                }
                None => {
                    let log_msg = format!("AsteroidAck from Planet #{} !! NO ROCKET !!", planet_id);
                    orch.add_log(log_msg);

                    let mut payload = Payload::new();
                    payload.insert("message".into(), format!("AsteroidAck from Planet #{} !! NO ROCKET !!.", planet_id));

                    orch.add_structured_log(LogEvent::new(
                        Some(Participant::new(ActorType::Planet, planet_id)),
                        None,
                        EventType::InternalPlanetAction,
                        Channel::Debug,
                        payload,
                    ));

                    // Isolated scoping block to find if any active explorer is currently located on the doomed planet
                    let explorer_to_kill = {
                        let ep_guard = orch.explorer_planet.read().unwrap();
                        ep_guard.iter()
                            .find(|&(_, &p_id)| p_id == target)
                            .map(|(&exp_id, _)| exp_id)
                    };

                    // If an explorer is caught in the impact, issue an immediate kill instruction to its thread channel
                    if let Some(exp_id) = explorer_to_kill {
                        if let Some((tx_orch_to_exp, _, _, _)) = orch.explorer_channels.get(&exp_id) {
                            let _ = tx_orch_to_exp.send(OrchestratorToExplorer::KillExplorer);
                            println!("Pianeta #{} distrutto: Explorer #{} eliminato.\n", target, exp_id);
                            orch.add_log(format!("Pianeta #{} distrutto: Explorer #{} eliminato.", target, exp_id));

                            let mut payload = Payload::new();
                            payload.insert("message".into(), format!("Explorer #{} è morto sul pianeta #{}.", exp_id, target));

                            orch.add_structured_log(LogEvent::self_directed(
                                Participant::new(ActorType::Explorer, exp_id),
                                EventType::InternalExplorerAction,
                                Channel::Info,
                                payload,
                            ));
                        }
                    }

                    // Trigger structural planet teardown since it failed to defend against the impact
                    sender.send(OrchestratorToPlanet::KillPlanet).unwrap();
                    map_guard.planet_killed(target);

                    let kill_ack = receiver.recv().unwrap();
                    match kill_ack {
                        PlanetToOrchestrator::KillPlanetResult { planet_id } => {
                            let log_msg = format!("Killed Planet #{}", planet_id);
                            orch.add_log(log_msg.clone());

                            let mut payload = Payload::new();
                            payload.insert("message".into(), format!("Il pianeta #{} è stato distrutto.", planet_id));

                            orch.add_structured_log(LogEvent::new(
                                None,
                                Some(Participant::new(ActorType::Planet, planet_id)),
                                EventType::InternalPlanetAction,
                                Channel::Info,
                                payload,
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }

        PlanetToOrchestrator::KillPlanetResult { planet_id } => {
            println!("2 Killed Planet #{}", planet_id);
        }

        msg => {
            println!("Received unexpected msg: {:?}", msg);
        }
    }
}