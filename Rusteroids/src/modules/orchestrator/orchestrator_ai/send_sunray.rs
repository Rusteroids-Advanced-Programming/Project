use crate::modules::orchestrator::orchestrator::Orchestrator;
use crate::modules::read_galaxy::stats::Counts;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::utils::ID;

/// Dispatches a generated sunray event payload directly to a targeted planet and increments its interaction counter.
pub fn send_sunray_impl(orch: &Orchestrator, target: ID) -> Option<ID> {
    let sunray = orch.forge.generate_sunray();
    let planet_channels_guard = orch.planet_channels.read().unwrap();
    let (sender, receiver, _) = &*planet_channels_guard.get(&target).unwrap();
    sender.send(OrchestratorToPlanet::Sunray(sunray)).unwrap();

    let mut res_id = None;
    let ack = receiver.recv().unwrap();

    match ack {
        PlanetToOrchestrator::SunrayAck { planet_id } => {
            res_id = Some(planet_id);

            // Safely lock the stats metric map to track total sunray counts successfully processed by this planet
            orch.stats_map
                .write()
                .unwrap()
                .increase_count(target, Counts::Sunrays);
        }

        msg => {
            let mut payload = Payload::new();
            payload.insert(
                "Received unexpected msg while waiting for sunray ack".into(),
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
    res_id
}
