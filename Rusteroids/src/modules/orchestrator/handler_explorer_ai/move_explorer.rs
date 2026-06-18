use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender, unbounded};

/// Tells the explorer to move to the given planet, providing it a fresh channel toward that planet,
/// then blocks waiting for the move acknowledgement.
pub fn move_explorer_impl(orch: &Orchestrator, expl_id: ID, planet_id: ID) {
    // Channel handed to the explorer so it can talk to the destination planet;
    // the local receiver end (_tmp) is discarded here
    let (to_planet, _tmp): (Sender<ExplorerToPlanet>, Receiver<ExplorerToPlanet>) = unbounded();

    // Unused reverse channel (planet -> explorer); created but not wired up
    let (_tmp2, _from_planet2): (Sender<PlanetToExplorer>, Receiver<PlanetToExplorer>) =
        unbounded();

    // Retrieve the channel tuple for this explorer; only the sender (tx1) and receiver (rx1) are needed
    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();

    tx1.send(OrchestratorToExplorer::MoveToPlanet {
        sender_to_new_planet: Some(to_planet),
        planet_id,
    })
    .unwrap();
    // Block until the explorer confirms the move
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::MovedToPlanetResult {
            explorer_id,
            planet_id: _planet_id,
        } => {
            println!(" explorer  {} moved", explorer_id);
        }
        // Ignore any other message variant
        _ => {}
    }
}
