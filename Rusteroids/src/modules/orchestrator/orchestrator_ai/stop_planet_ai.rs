use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;

pub fn stop_planet_ai_impl(orch: &Orchestrator, target: ID) {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _expl_sender) = planet_channels_guard.get(&target).unwrap();
    sender.send(OrchestratorToPlanet::StopPlanetAI).unwrap();

    let msg = receiver.recv().unwrap();
    match msg {
        PlanetToOrchestrator::StopPlanetAIResult { planet_id } => {
            println!("Planet #{} stopped", planet_id);
        }
        _ => {}
    }
}
