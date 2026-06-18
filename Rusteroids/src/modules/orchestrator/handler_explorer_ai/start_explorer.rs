use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;

#[allow(unused)]
/// Sends a start signal to the explorer's AI and prints a confirmation once it acknowledges.
/// Sends the request over the explorer's channel, then blocks waiting for the matching response.
pub fn start_explorer_impl(orch: &Orchestrator, expl_id: ID) {
    // Retrieve the channel tuple for this explorer; only the sender (tx1) and receiver (rx1) are needed
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();
    tx1.send(OrchestratorToExplorer::StartExplorerAI).unwrap();
    // Block until the explorer acknowledges the start
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::StartExplorerAIResult { explorer_id } => {}
        msg => {
            let mut payload = Payload::new();
            payload.insert(
                "Received unexpected msg while starting explorer AI".into(),
                format!("{:?}", msg),
            );

            orch.add_structured_log(LogEvent::new(
                Some(Participant::new(ActorType::Orchestrator, 0u32)),
                Some(Participant::new(ActorType::Explorer, expl_id)),
                EventType::InternalOrchestratorAction,
                Channel::Error,
                payload,
            ));
        }
    }
}
