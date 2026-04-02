use crate::modules::manual_explorer::bag_type::DummyBag;
use crate::modules::manual_explorer::manual_explorer::ManualExplorer;
use crate::modules::orchestrator::event_manager::ExplorerListener;
use crate::modules::orchestrator::handler_explorer_ai::HandlerExplorer;
use crate::modules::orchestrator::orchestator::Orchestrator;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::PlanetToExplorer;
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender, unbounded};
use rand::Rng;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

#[allow(dead_code)]
pub trait ExplorerInitializer {
    fn initialize_explorers(
        &mut self,
        explorers: Vec<(u32, u32)>,
        orch_clone: Arc<RwLock<Orchestrator>>,
    );

    fn check_explorer_number(&self, explorers_num: usize) -> Result<(), String>;

    fn set_spawn_planet(&self) -> ID;
}

impl ExplorerInitializer for Orchestrator {
    fn initialize_explorers(
        &mut self,
        explorers: Vec<(u32, u32)>,
        _orch_clone: Arc<RwLock<Orchestrator>>,
    ) {
        self.check_explorer_number(explorers.len()).unwrap();

        for (expl_id, planet_id) in explorers {
            let (tx1, rx1): (
                Sender<OrchestratorToExplorer>,
                Receiver<OrchestratorToExplorer>,
            ) = unbounded();

            let (tx2, rx2): (
                Sender<ExplorerToOrchestrator<DummyBag>>,
                Receiver<ExplorerToOrchestrator<DummyBag>>,
            ) = unbounded();

            let planet_channels_guard = self.planet_channels.read().unwrap();

            let (tx_planet_expl, rx_planet_expl) = unbounded::<PlanetToExplorer>();
            let mut explorer = ManualExplorer::new(expl_id, planet_id, rx1, tx2);
            self.explorer_channels
                .insert(expl_id, (tx1, rx2, tx_planet_expl, rx_planet_expl));

            let spawn_planet = self.set_spawn_planet();
            println!("Spawn planet: {}", spawn_planet);

            explorer.to_planet = RwLock::new(Some(
                planet_channels_guard.get(&spawn_planet).unwrap().2.clone(),
            )); //crabtorio have problem whit ts. no initialization of the channels. ONly them are having this problem
            explorer.from_planet = RwLock::new(Some(
                self.explorer_channels.get(&expl_id).unwrap().3.clone(),
            ));

            let tmp = Arc::new(explorer);
            let tmp_clone = tmp.clone();
            let handle = thread::spawn(move || {
                tmp_clone.run().unwrap_or(());
            });

            self.explorer_planet
                .write()
                .unwrap()
                .insert(expl_id, spawn_planet);
            // explorer.to_planet = RwLock::new(Some(self.planet_channels.get(&spawn_planet).unwrap().2.clone()));
            //  println!("Dio4");
            //  explorer.from_planet = RwLock::new(Some(self.explorer_channels.get(&expl_id).unwrap().3.clone()));
            self.explorer_threads.insert(expl_id, handle);
            self.explorers.insert(expl_id, tmp.clone());

            self.start_explorer(expl_id);
            //let spawn_planet = self.set_spawn_planet();
            sleep(Duration::from_millis(2000));
            // self.send_explorer(spawn_planet, expl_id);

            let planet_channels = self.planet_channels.clone();
            let expl_channels = Arc::new(RwLock::new(
                self.explorer_channels.get(&expl_id).unwrap().clone(),
            ));
            let graph = self.galaxy_graph.clone();

            let expl_listener = Arc::new(ExplorerListener::new(
                expl_channels,
                planet_channels,
                graph,
                tmp.clone(),
                _orch_clone.clone(),
            ));
            let listener_clone = expl_listener.clone();

            let _explorer_listener_handle = thread::spawn(move || {
                expl_listener.explorer_event_listener();
            });

            listener_clone.send_explorer_event_manager(spawn_planet, expl_id, None); //--> creare funzione spawn_explorer per primo pianeta su cui inizia explorer, se no si bugga tutto
            
            //--> send_explorer_even_manager mandava un outgoing_explorer_request allo stesso pianeta a cui mandava un incoming_explorere_request, buggando tutto.
            // listener_clone.send_explorer_event_manager(7, expl_id, None);
            // tmp.ask_for_neighbours();
            // tmp.ask_combinations();
            // tmp.ask_supported_resources();

            let _user_input_handle = thread::spawn(move || {
                loop {
                    tmp.handle_user_input();
                }
            });
        }
    }

    fn check_explorer_number(&self, explorers_num: usize) -> Result<(), String> {
        let planet_channels_guard = self.planet_channels.read().unwrap();
        if planet_channels_guard.len() <= explorers_num {
            Err(format!(
                "There are too many explorers ({}) for the number of planets ({}) ",
                explorers_num,
                planet_channels_guard.len()
            ))
        } else {
            Ok(())
        }
    }
    fn set_spawn_planet(&self) -> ID {
        let vec_planets = self.get_planet_ids_list();
        let mut available_planets = Vec::new();

        let explorer_planet_guard = self.explorer_planet.read().unwrap();

        for p_id in vec_planets {
            let is_occupied = explorer_planet_guard.values().any(|&pos| pos == p_id);
            if !is_occupied {
                available_planets.push(p_id);
            }
        }

        if available_planets.is_empty() {
            return 1;
        }

        let mut rng = rand::rng();
        let idx = rng.random_range(0..available_planets.len());
        available_planets[idx]
    }
}
