use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;

/// Sends a termination signal to freeze the autonomous background logic thread of a planet.
pub fn stop_planet_ai_impl(orch: &Orchestrator, target: ID) {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _expl_sender) = planet_channels_guard.get(&target).unwrap();
    sender.send(OrchestratorToPlanet::StopPlanetAI).unwrap();

    // Blocks execution until the planet thread successfully halts its routine loop and responds
    let msg = receiver.recv().unwrap();
    match msg {
        PlanetToOrchestrator::StopPlanetAIResult { planet_id } => {
            println!("Planet #{} stopped", planet_id);
        }
        _ => {}
    }
}