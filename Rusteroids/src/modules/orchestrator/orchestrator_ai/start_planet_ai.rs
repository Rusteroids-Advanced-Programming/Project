use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use std::time::Duration;

#[allow(unused)]
/// Signals the designated planet thread to initiate its autonomous routine loop.
pub fn start_planet_ai_impl(orch: &Orchestrator, target: u32) {
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _) = &*planet_channels_guard.get(&target).unwrap();
    sender.send(OrchestratorToPlanet::StartPlanetAI).unwrap();

    // Await execution confirmation with a fallback threshold to prevent blocking on deadlocked AI threads
    let ack = receiver.recv_timeout(Duration::from_millis(2000)).unwrap();
    match ack {
        PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {}

        msg => {
            let mut payload = Payload::new();
            payload.insert(
                "Received unexpected msg while starting planet AI".into(),
                format!("{:?}", msg),
            );

            orch.add_structured_log(LogEvent::new(
                Some(Participant::new(ActorType::Orchestrator, 0u32)),
                Some(Participant::new(ActorType::Planet, target)),
                EventType::InternalOrchestratorAction,
                Channel::Error,
                payload,
            ));
        }
    }
}
