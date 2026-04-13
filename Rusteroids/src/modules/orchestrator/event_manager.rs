use crate::modules::manual_explorer::bag_type::DummyBag;
use crate::modules::manual_explorer::manual_explorer::ManualExplorer;
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

pub trait ManageEvents {
    fn manage(&self);
    fn get_target_planet(&self) -> ID;
}

impl ManageEvents for Orchestrator {
    fn manage(&self) {
        let mut rng = rand::rng();
        let asteroid_probability = self.difficulty.get_ratio();
        let target = self.get_target_planet();
        if rng.random_bool(asteroid_probability) {
            let log_msg = format!("Sending asteroid to {}", target);
            //println!("{}",log_msg);
            self.add_log(log_msg);
            self.send_asteroid(target);
        } else {
            self.send_sunray(target);
        }
    }

    fn get_target_planet(&self) -> ID {
        let planet_vec = self.get_planet_ids_list();
        let mut rng = rand::rng();
        let rand_index = rng.random_range(0..planet_vec.len());
        planet_vec[rand_index]
    }
}

#[allow(dead_code)]
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
    explorer: Arc<ManualExplorer>,
    orch: Arc<RwLock<Orchestrator>>, //aggiunto per aggiornare l'orchestrator
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
        explorer: Arc<ManualExplorer>,
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

    pub fn explorer_event_listener(&self) {
        let (tx, rx, _tx_planet_to_expl, _rx_planet_to_expl) =
            self.explorer_channels.read().unwrap().clone();
        loop {
            let msg = rx.recv().unwrap();
            match msg {
                ExplorerToOrchestrator::NeighborsRequest {
                    explorer_id,
                    current_planet_id,
                } => {
                    println!("Received neighbors request from explorer #{}", explorer_id);
                    let mut neighbours = Vec::new();
                    for node in &self.galaxy_graph.read().unwrap().nodes {
                        let guard = node.read().unwrap();
                        if guard.value == current_planet_id {
                            for n in &guard.adjacent_nodes {
                                neighbours.push(n.read().unwrap().value);
                            }
                        }
                    }

                    println!("Neighbors 2: {:?}", neighbours);
                    tx.send(OrchestratorToExplorer::NeighborsResponse {
                        neighbors: neighbours,
                    })
                    .unwrap();
                    println!("Neighbors response sent");
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
                    //aggiunto per aggiornare orchestrator

                    let orch_guard = self.orch.read().unwrap();
                    orch_guard
                        .explorer_planet
                        .write()
                        .unwrap()
                        .insert(explorer_id, planet_id);

                    *self.explorer.base.current_planet_id.write().unwrap() = planet_id;
                }
                ExplorerToOrchestrator::BagContentResponse {
                    explorer_id: _,
                    bag_content,
                } => {
                    //aggiunto per aggiornare orchestrator
                    let mut bag_guard = self.explorer.dummy_bag.write().unwrap();
                    *bag_guard = bag_content;
                }
                ExplorerToOrchestrator::GenerateResourceResponse {
                    explorer_id,
                    generated,
                } => {
                    if generated.is_ok() {
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
                    if generated.is_ok() {
                        println!("Crafting eseguito");

                        let (tx, _, _, _) = self.explorer_channels.read().unwrap().clone();
                        tx.send(OrchestratorToExplorer::BagContentRequest).unwrap();
                    } else {
                        println!(
                            " Errore nel crafting di {}: {:?}",
                            explorer_id,
                            generated.err().unwrap()
                        );
                    }
                }

                ExplorerToOrchestrator::KillExplorerResult { explorer_id } => {
                    // DA FIXARE NON VA UN CAZZO

                    if let Ok(mut alive_lock) = self.explorer.base.alive.write() {
                        *alive_lock = false;
                    }

                    let orch_guard = self.orch.read().unwrap();

                    if let Some(orch_explorer) = orch_guard.explorers.get(&explorer_id) {
                        let mut alive_status = orch_explorer.base.alive.write().unwrap();
                        *alive_status = false;
                    }

                    let mut explorer_planet_lock = orch_guard.explorer_planet.write().unwrap();
                    explorer_planet_lock.remove(&explorer_id);

                    orch_guard.add_log(format!("ALERT: Esploratore #{} è morto.", explorer_id));
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

    pub fn send_explorer_event_manager(
        &self,
        planet_id: ID,
        explorer_id: ID,
        current_planet_id: Option<ID>,
    ) {
        //se current_planet_id == None allora si sta spawnando l'explorer
        let explorer_channels_guard = self.explorer_channels.read().unwrap();
        let (_expl_sender, _expl_receiver, tx_planet_to_expl, _rx_planet_to_expl) =
            &*explorer_channels_guard;
        let planet_channels_guard = self.planet_channels.read().unwrap();
        let planet_channels = planet_channels_guard.get(&planet_id).unwrap();
        let (sender, receiver, _expl_sender) = planet_channels;
        println!("Sending explorer #{} to {}", explorer_id, planet_id);
        sender
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id,
                new_sender: tx_planet_to_expl.clone(),
            })
            .unwrap();
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
                                    //--> mandiamo a planet_id = dst_planet, ma noi dobbiamo mandare a current planet, mandando outgoing allo stesso pianeta in cui siamo gia, mandando tuttoa puttane
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
                        // false
                    }
                }
            }
            _msg => {
                println!("Received unexpected msg while waiting incoming explorer response");
                // false
            }
        }
    }

    fn send_outgoing_explorer(&self, explorer_id: ID, planet_id: ID) -> bool {
        let planet_channels_guard = self.planet_channels.read().unwrap();
        let planet_channels = planet_channels_guard.get(&planet_id).unwrap(); //_> planet che dano problemi: #6600

        let (tx2, rx2, _ex2) = planet_channels;
        tx2.send(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id })
            .unwrap();
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
}
