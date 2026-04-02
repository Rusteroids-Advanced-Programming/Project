use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use crate::modules::read_galaxy::stats::Counts;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;
use std::fmt::format;
use common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;

pub fn send_asteroid_impl(orch: &Orchestrator, target: ID) {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _expl_sender) = &*planet_channels_guard.get(&target).unwrap();
    sender
        .send(OrchestratorToPlanet::Asteroid(
            orch.forge.generate_asteroid(),
        ))
        .unwrap();

    let mut map_guard = orch.stats_map.write().unwrap();
    map_guard.increase_count(target, Counts::Asteroids);

    let msg = receiver.recv().unwrap();
    match msg {
        PlanetToOrchestrator::AsteroidAck { planet_id, rocket } => {
            match rocket {
                Some(_rocket) => {
                    let log_msg = format!("AsteroidAck from Planet #{} with rocket", planet_id);
                    //println!("{}", planet_id);
                    orch.add_log(log_msg);
                    map_guard.increase_count(target, Counts::Rockets);
                }
                None => {
                    let log_msg = format!("AsteroidAck from Planet #{} !! NO ROCKET !!", planet_id);
                    //println!("{}", log_msg);
                    orch.add_log(log_msg);
                    //AGGIUNTO INVIO MESSAGGIO PER UCCIDERE L'EXPLORER, VA BENE QUA?
                    let explorer_to_kill = {
                        let ep_guard = orch.explorer_planet.read().unwrap();
                        ep_guard.iter()
                            .find(|&(_, &p_id)| p_id == target)
                            .map(|(&exp_id, _)| exp_id)
                    };

                    if let Some(exp_id) = explorer_to_kill {
                        if let Some((tx_orch_to_exp, _, _, _)) = orch.explorer_channels.get(&exp_id) {
                            let _ = tx_orch_to_exp.send(OrchestratorToExplorer::KillExplorer);
                            orch.add_log(format!("Pianeta #{} distrutto: Explorer #{} eliminato.", target, exp_id));
                        }
                    }

                    sender.send(OrchestratorToPlanet::KillPlanet).unwrap();
                    map_guard.planet_killed(target);

                    let kill_ack = receiver.recv().unwrap();
                    match kill_ack {
                        PlanetToOrchestrator::KillPlanetResult { planet_id } => {

                            let log_msg = format!("Killed Planet #{}", planet_id);
                            //println!("{}",log_msg);
                            orch.add_log(log_msg);
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
