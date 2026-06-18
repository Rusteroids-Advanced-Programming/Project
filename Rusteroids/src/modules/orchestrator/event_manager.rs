use crate::modules::explorer_utils::bag_type::DummyBag;
use crate::modules::explorers::manual_explorer::manual_explorer::ManualExplorer;
use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use crate::modules::read_galaxy::graph::Graph;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use crate::modules::explorer_utils::explorer::ExplorerBehaviour;
use crate::modules::orchestrator::handler_explorer_ai::HandlerExplorer;

/// Abstraction layer for dispatching system-wide environment events (asteroids, solar rays) to target nodes.
pub trait ManageEvents {
    fn manage(&self) -> bool;
    fn get_target_planet(&self) -> Option<ID>;
}

impl ManageEvents for Orchestrator {
    /// Evaluates probabilistic dice rolls to spawn cosmic hazards onto a randomly selected live planet.
    fn manage(&self) -> bool{
        let mut rng = rand::rng();
        let asteroid_probability = self.difficulty.get_ratio();
        let target = self.get_target_planet();
        if let Some(target) = target {
            if rng.random_bool(asteroid_probability) {
                let log_msg = format!("Sending asteroid to {}", target);
                self.add_log(log_msg.clone());
                self.send_asteroid(target);

                let mut payload = Payload::new();
                payload.insert("message".into(), log_msg);
                payload.insert("type".into(), "asteroid".into());

                self.add_structured_log(LogEvent::new(
                    Some(Participant::new(ActorType::Orchestrator, 0u32)),
                    Some(Participant::new(ActorType::Planet, target)),
                    EventType::MessageOrchestratorToPlanet,
                    Channel::Debug,
                    payload,
                ));
            } else {
                let log_msg = format!("Sending sunray to {}", target);

                let mut payload = Payload::new();
                payload.insert("message".into(), log_msg);
                payload.insert("type".into(), "sunray".into());

                self.add_structured_log(LogEvent::new(
                    Some(Participant::new(ActorType::Orchestrator, 0u32)),
                    Some(Participant::new(ActorType::Planet, target)),
                    EventType::MessageOrchestratorToPlanet,
                    Channel::Debug,
                    payload,
                ));

                self.send_sunray(target);
            }

            true
        }
        else {
            println!("Tutti i pianeti esplosi, gioco finito");
            false
        }
    }

    /// Selects a random active planet identifier from the existing system registry.
    fn get_target_planet(&self) -> Option<ID> {
        let planet_vec = self.get_planet_ids_list();
        if planet_vec.len() == 0 {
            None
        }
        else {
            let mut rng = rand::rng();
            let rand_index = rng.random_range(0..planet_vec.len());
            Some(planet_vec[rand_index])
        }
    }
}

#[allow(dead_code)]
/// Asynchronous message listener running an infinite event loop over crossbeam channels linked to a single explorer.
pub struct ExplorerListener {
    explorer_channels: Arc<
        RwLock<(
            Sender<OrchestratorToExplorer>,
            Receiver<ExplorerToOrchestrator<DummyBag>>,
            Sender<PlanetToExplorer>,
            Receiver<PlanetToExplorer>,
        )>,
    >,
    planet_channels: Arc<
        RwLock<
            HashMap<
                ID,
                (
                    Sender<OrchestratorToPlanet>,
                    Receiver<PlanetToOrchestrator>,
                    Sender<ExplorerToPlanet>,
                ),
            >,
        >,
    >,
    galaxy_graph: Arc<RwLock<Graph<ID>>>,
    explorer: Arc<dyn ExplorerBehaviour>,
    orch: Arc<RwLock<Orchestrator>>,
}

#[allow(dead_code)]
impl ExplorerListener {
    pub fn new(
        explorer_channels: Arc<
            RwLock<(
                Sender<OrchestratorToExplorer>,
                Receiver<ExplorerToOrchestrator<DummyBag>>,
                Sender<PlanetToExplorer>,
                Receiver<PlanetToExplorer>,
            )>,
        >,
        planet_channels: Arc<
            RwLock<
                HashMap<
                    ID,
                    (
                        Sender<OrchestratorToPlanet>,
                        Receiver<PlanetToOrchestrator>,
                        Sender<ExplorerToPlanet>,
                    ),
                >,
            >,
        >,
        galaxy_graph: Arc<RwLock<Graph<ID>>>,
        explorer: Arc<dyn ExplorerBehaviour>,
        orch: Arc<RwLock<Orchestrator>>,
    ) -> Self {
        Self {
            explorer_channels,
            planet_channels,
            galaxy_graph,
            explorer,
            orch,
        }
    }

    /// Infinite polling loop blocking on incoming message structures dispatched by the explorer thread.
    pub fn explorer_event_listener(&self) {
        let (tx, rx, _tx_planet_to_expl, _rx_planet_to_expl) =
            self.explorer_channels.read().unwrap().clone();
        loop {
            // Block until a new network payload arrives on the crossbeam channel receiver
            let msg = rx.recv().unwrap();
            match msg {
                ExplorerToOrchestrator::NeighborsRequest {
                    explorer_id,
                    current_planet_id,
                } => {
                    self.orch.read().unwrap().get_explorer_neighbours(explorer_id, current_planet_id)
                }
                ExplorerToOrchestrator::TravelToPlanetRequest {
                    explorer_id,
                    current_planet_id,
                    dst_planet_id,
                } => {
                    self.send_explorer_event_manager(
                        dst_planet_id,
                        explorer_id,
                        Some(current_planet_id),
                    );
                }
                ExplorerToOrchestrator::MovedToPlanetResult {
                    explorer_id,
                    planet_id,
                } => {
                    let orch_guard = self.orch.read().unwrap();
                    orch_guard
                        .explorer_planet
                        .write()
                        .unwrap()
                        .insert(explorer_id, planet_id);

                    let base_guard = self.explorer.get_base();
                    *base_guard.current_planet_id.write().unwrap() = planet_id;

                    let log_msg = format!("Explorer #{} è atterrato sul pianeta #{}.", explorer_id, planet_id);
                    let mut payload = Payload::new();
                    payload.insert("message".into(), log_msg);

                    orch_guard.add_structured_log(LogEvent::new(
                        Some(Participant::new(ActorType::Explorer, explorer_id)),
                        Some(Participant::new(ActorType::Planet, planet_id)),
                        EventType::MessageExplorerToPlanet,
                        Channel::Info,
                        payload,
                    ));

                    // Verify if the landing destination was eliminated by an asteroid during the transit sequence
                    let stats_guard = orch_guard.stats_map.read().unwrap();
                    let planet_stats = stats_guard.get(&planet_id);

                    match planet_stats {
                        None =>  {}
                        Some(planet_stats) => {
                            if !planet_stats.alive{
                                *base_guard.alive.write().unwrap() = false;
                            }
                        }
                    }
                }

                ExplorerToOrchestrator::BagContentResponse {
                    explorer_id: _,
                    bag_content,
                } => {
                    let mut bag_guard = self.explorer.get_dummy_bag_mut();
                    *bag_guard = bag_content;
                }

                ExplorerToOrchestrator::GenerateResourceResponse {
                    explorer_id,
                    generated,
                } => {
                    if let Ok(risorsa_estratta) = &generated {
                        let log_res = format!("Explorer #{} ha estratto: {:?}.", explorer_id, risorsa_estratta);
                        let mut payload = Payload::new();
                        payload.insert("message".into(), log_res);
                        payload.insert("resource_type".into(), format!("{:?}", risorsa_estratta));

                        self.orch.read().unwrap().add_structured_log(LogEvent::self_directed(
                            Participant::new(ActorType::Explorer, explorer_id),
                            EventType::InternalExplorerAction,
                            Channel::Debug,
                            payload,
                        ));

                        let (tx, _, _, _) = self.explorer_channels.read().unwrap().clone();
                        tx.send(OrchestratorToExplorer::BagContentRequest).unwrap();
                    } else {
                        println!("Errore nella generazione della risorsa: {:?}", generated);
                    }
                }

                ExplorerToOrchestrator::CombineResourceResponse {
                    explorer_id,
                    generated,
                } => {
                    if let Ok(oggetto_craftato) = &generated {
                        let log_craft = format!("Explorer #{} ha craftato: {:?}.", explorer_id, oggetto_craftato);
                        println!("{}", log_craft);
                        let mut payload = Payload::new();
                        payload.insert("message".into(), log_craft);
                        payload.insert("crafted_item".into(), format!("{:?}", oggetto_craftato));
                        self.orch.read().unwrap().add_structured_log(LogEvent::self_directed(
                            Participant::new(ActorType::Explorer, explorer_id),
                            EventType::InternalExplorerAction,
                            Channel::Debug,
                            payload,
                        ));

                        let (tx, _, _, _) = self.explorer_channels.read().unwrap().clone();
                        tx.send(OrchestratorToExplorer::BagContentRequest).unwrap();
                    } else {
                        println!(
                            "Errore nel crafting dell'explorer #{}: {:?}",
                            explorer_id,
                            generated.err().unwrap()
                        );
                    }
                }

                ExplorerToOrchestrator::KillExplorerResult { explorer_id } => {
                    if let Ok(mut alive_lock) = self.explorer.get_base().alive.write() {
                        *alive_lock = false;
                    }

                    let orch_guard = self.orch.read().unwrap();
                    orch_guard.add_log(format!("ALERT: Esploratore #{} è morto.", explorer_id));

                    let mut payload = Payload::new();
                    payload.insert("message".into(), format!("Explorer #{} è morto.", explorer_id));

                    orch_guard.add_structured_log(LogEvent::self_directed(
                        Participant::new(ActorType::Explorer, explorer_id),
                        EventType::InternalExplorerAction,
                        Channel::Info,
                        payload,
                    ));

                    // Break the listener loop and shut down this specific handling thread
                    break;
                }
                msg => {
                    println!(
                        "1 Received unexpected msg while waiting for an explorer's request: {:?}",
                        msg
                    )
                }
            }
        }
    }

    /// Orchestrates destination handshakes, supporting both initial actor spawning or active planetary relocation steps.
    pub fn send_explorer_event_manager(
        &self,
        planet_id: ID,
        explorer_id: ID,
        current_planet_id: Option<ID>,
    ) {
        let explorer_channels_guard = self.explorer_channels.read().unwrap();
        let (_expl_sender, _expl_receiver, tx_planet_to_expl, _rx_planet_to_expl) =
            &*explorer_channels_guard;
        let planet_channels_guard = self.planet_channels.read().unwrap();

        println!("planet channels: {:?}\nplanet id: {}", planet_channels_guard, planet_id);

        let planet_channels = planet_channels_guard.get(&planet_id).unwrap();
        let (sender, receiver, _expl_sender) = planet_channels;

        if ! *self.explorer.get_base().alive.read().unwrap() {
            return;
        }

        println!("Sending explorer #{} to {}", explorer_id, planet_id);

        let incoming_expl_msg_sent = sender
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id,
                new_sender: tx_planet_to_expl.clone(),
            });

        // Fail-safe handling if the target planet instance collapses while receiving the incoming transaction
        if let Err(e) = incoming_expl_msg_sent {
            println!("Error sending incoming explorer message: {:?}", e);
            return;
        }

        let msg = receiver.recv().unwrap();
        match msg {
            PlanetToOrchestrator::IncomingExplorerResponse {
                planet_id,
                explorer_id,
                res,
            } => {
                match res {
                    Ok(_response) => {
                        println!(
                            "Incoming explorer response received from planet #{}",
                            planet_id
                        );
                        match current_planet_id {
                            Some(current_planet_id) => {
                                if self.send_outgoing_explorer(explorer_id, current_planet_id) {
                                    let (tx2, _rx2, _, _rx_planet_to_expl) =
                                        &*explorer_channels_guard;
                                    let (_, _, expl_to_planet) = planet_channels;
                                    tx2.send(OrchestratorToExplorer::MoveToPlanet {
                                        sender_to_new_planet: Some(expl_to_planet.clone()),
                                        planet_id,
                                    })
                                        .unwrap();
                                }
                            }
                            None => {
                                let (tx2, _rx2, _, _rx_planet_to_expl) = &*explorer_channels_guard;
                                let (_, _, expl_to_planet) = planet_channels;
                                tx2.send(OrchestratorToExplorer::MoveToPlanet {
                                    sender_to_new_planet: Some(expl_to_planet.clone()),
                                    planet_id,
                                })
                                    .unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error while trying to move explorer: {:?}", e);
                    }
                }
            }
            _msg => {
                println!("Received unexpected msg while waiting incoming explorer response");
            }
        }
    }

    /// Asks the actor's current location to clear and unregister its local explorer state data tracking.
    fn send_outgoing_explorer(&self, explorer_id: ID, planet_id: ID) -> bool {
        let planet_channels_guard = self.planet_channels.read().unwrap();
        let planet_channels = planet_channels_guard.get(&planet_id).unwrap();

        let (tx2, rx2, _ex2) = planet_channels;

        let outgoing_msg_to_planet = tx2.send(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id });
        match outgoing_msg_to_planet {
            Ok(_) => {
                let planet_resp = rx2.recv().unwrap();
                match planet_resp {
                    PlanetToOrchestrator::OutgoingExplorerResponse {
                        planet_id: _,
                        explorer_id: _,
                        res,
                    } => match res {
                        Ok(_) => true,
                        Err(_err) => false,
                    },
                    resp => {
                        println!(
                            "2 received unexpected msg while waiting for outgoing explorer response{:?}",
                            resp
                        );
                        false
                    }
                }
            }
            Err(e) => {
                println!("Explorer #{} tried to escape an exploding planet #{} and he died", explorer_id, planet_id);
                false
            }
        }
    }
}