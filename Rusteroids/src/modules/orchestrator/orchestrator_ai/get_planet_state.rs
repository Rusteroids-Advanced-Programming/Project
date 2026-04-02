use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use common_game::components::planet::DummyPlanetState;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;

/*pub fn get_planet_state_impl(orch: &Orchestrator, target: ID) -> Option<DummyPlanetState>{
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _) = &*planet_channels_guard.get(&target).unwrap();
    sender.send(OrchestratorToPlanet::InternalStateRequest).unwrap();

    let msg = receiver.recv().unwrap();
    let mut dummy_state= None;
    match msg {
        PlanetToOrchestrator::InternalStateResponse {planet_id: _, planet_state} => {
            dummy_state = Some(planet_state);
        }

        msg => {
            println!("Received unexpected msg while waiting for internal state: {:?}", msg);
        }
    }

    dummy_state
}*/

pub fn get_planet_state_impl(orch: &Orchestrator, target: ID) -> Option<DummyPlanetState> {
    let planet_channels_guard = orch.planet_channels.read().ok()?; // Evita panic se il lock è poisoned
    let (sender, receiver, _) = planet_channels_guard.get(&target)?; // Restituisce None se il pianeta non esiste più

    // Usa .ok() invece di .unwrap() per gestire la chiusura dei canali senza far crashare il server
    sender
        .send(OrchestratorToPlanet::InternalStateRequest)
        .ok()?;

    match receiver.recv().ok()? {
        PlanetToOrchestrator::InternalStateResponse { planet_state, .. } => Some(planet_state),
        _ => None,
    }
}
