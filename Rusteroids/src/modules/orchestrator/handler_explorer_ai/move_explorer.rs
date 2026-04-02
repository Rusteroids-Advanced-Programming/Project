use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender, unbounded};

pub fn move_explorer_impl(orch: &Orchestrator, expl_id: ID, planet_id: ID) {
    let (to_planet, _tmp): (Sender<ExplorerToPlanet>, Receiver<ExplorerToPlanet>) = unbounded();

    let (_tmp2, _from_planet2): (Sender<PlanetToExplorer>, Receiver<PlanetToExplorer>) =
        unbounded();

    let (tx1, rx1, _, _) = orch.explorer_channels.get(&expl_id).unwrap();

    tx1.send(OrchestratorToExplorer::MoveToPlanet {
        sender_to_new_planet: Some(to_planet),
        planet_id,
    })
    .unwrap();
    let msg = rx1.recv().unwrap();
    match msg {
        ExplorerToOrchestrator::MovedToPlanetResult {
            explorer_id,
            planet_id: _planet_id,
        } => {
            println!(" explorer  {} moved", explorer_id);
        }
        _ => {}
    }
}
