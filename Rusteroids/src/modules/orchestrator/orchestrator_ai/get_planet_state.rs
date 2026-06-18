use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::components::planet::DummyPlanetState;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;

/// Safely queries the internal operational state of a target planet managed by the orchestrator.
pub fn get_planet_state_impl(orch: &Orchestrator, target: ID) -> Option<DummyPlanetState> {
    // Avoid panics by handling potential lock poisoning safely using .ok()?
    let planet_channels_guard = orch.planet_channels.read().ok()?;

    // Gracefully return None if the target planet identifier no longer exists in the channel registry
    let (sender, receiver, _) = planet_channels_guard.get(&target)?;

    // Use .ok()? instead of .unwrap() to handle disconnected channels without crashing the thread
    sender
        .send(OrchestratorToPlanet::InternalStateRequest)
        .ok()?;

    // Match the incoming network message or abort if the channel was closed unexpectedly
    match receiver.recv().ok()? {
        PlanetToOrchestrator::InternalStateResponse { planet_state, .. } => Some(planet_state),
        _ => None,
    }
}
