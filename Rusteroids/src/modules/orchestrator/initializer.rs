use crate::modules::orchestrator::orchestrator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use crate::modules::read_galaxy::build_data_structs::build_galaxy_graph;
use common_game::components::planet::{Planet, PlanetType};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use crossbeam_channel::{Receiver, Sender, unbounded};
use rustrelli::ExplorerRequestLimit;
use std::sync::{Arc, RwLock};
use std::thread;

#[allow(dead_code)]
/// Trait mapping foundational bootstrap operations required to build the global galaxy cluster layout.
pub trait Initializer {
    fn initialize(&mut self);
}

impl Initializer for Orchestrator {
    /// Iterates through the generated topology graph nodes to build and spin up distinct planet simulation runtime hooks.
    fn initialize(&mut self) {
        self.galaxy_graph = Arc::new(RwLock::new(build_galaxy_graph()));
        let graph_guard = self.galaxy_graph.read().unwrap();

        let mut map_guard = self.stats_map.write().unwrap();

        for node in &graph_guard.nodes {
            let tmp = node.read().unwrap();

            let mut planet_wrapper: Planet;

            // Allocate localized crossbeam channel triples tracking communication boundaries for each individual planet thread
            let (tx1, rx1): (Sender<OrchestratorToPlanet>, Receiver<OrchestratorToPlanet>) =
                unbounded::<OrchestratorToPlanet>();
            let (tx2, rx2) = unbounded::<PlanetToOrchestrator>();
            let (ex1, ex2) = unbounded::<ExplorerToPlanet>();

            let planet_name: String;
            let planet_type: PlanetType;

            /// Local hardcoded blueprint mapping to retrieve matching ingredient vectors depending on the planet's internal type index.
            fn get_known_resources(planet_id: u32) -> (Vec<String>, Vec<String>) {
                match planet_id {
                    1 => (
                        vec![
                            "Carbon".into(),
                            "Hydrogen".into(),
                            "Oxygen".into(),
                            "Silicon".into(),
                        ],
                        vec![],
                    ),
                    2 => (vec!["Carbon".into()], vec![]),
                    3 => (vec!["Oxygen".into()], vec![]),
                    4 => (
                        vec![
                            "Carbon".into(),
                            "Hydrogen".into(),
                            "Oxygen".into(),
                            "Silicon".into(),
                        ],
                        vec!["Water".into()],
                    ),
                    5 => (
                        vec!["Hydrogen".into()],
                        vec![
                            "Water".into(),
                            "Diamond".into(),
                            "Life".into(),
                            "Robot".into(),
                            "Dolphin".into(),
                            "AIPartner".into(),
                        ],
                    ),
                    6 => (
                        vec!["Carbon".into()],
                        vec![
                            "Water".into(),
                            "Diamond".into(),
                            "Life".into(),
                            "Robot".into(),
                            "Dolphin".into(),
                            "AIPartner".into(),
                        ],
                    ),
                    7 => (
                        vec![
                            "Carbon".into(),
                            "Hydrogen".into(),
                            "Oxygen".into(),
                            "Silicon".into(),
                        ],
                        vec![],
                    ),
                    _ => (vec![], vec![]),
                }
            }
            let planet_id = &tmp.value;

            // Normalize layout boundaries by recycling planet configurations sequentially via a virtual modulo offset
            let virtual_id = ((planet_id - 1) % 7) + 1;

            // Polymorphic structure generation wrapping unique foreign planet creation crate factories
            match virtual_id {
                1 => {
                    planet_wrapper = rust_eze::create_planet(*planet_id, rx1, tx2, ex2);
                    planet_name = "Rust-Eze".to_string();
                }

                2 => {
                    planet_wrapper = ciuc::create_planet(rx1, tx2, ex2, *planet_id);
                    planet_name = "Ciuc".to_string()
                }

                3 => {
                    planet_wrapper = trip::trip(*planet_id, rx1, tx2, ex2).unwrap();
                    planet_name = "Trip".to_string()
                }

                5 => {
                    planet_wrapper = crabtorio::create_planet(*planet_id, rx1, tx2, ex2);
                    planet_name = "Crabtorio".to_string()
                }

                6 => {
                    planet_wrapper =
                        rusty_crab_ap2025::planet::create_planet(rx1, tx2, ex2, *planet_id);
                    planet_name = "Rusty-Crab".to_string()
                }

                7 => {
                    planet_wrapper = enterprise::create_planet(*planet_id, rx1, tx2, ex2);
                    planet_name = "Enterprise".to_string()
                }

                8 => {
                    planet_wrapper = rustrelli::create_planet(
                        *planet_id,
                        rx1,
                        tx2,
                        ex2,
                        ExplorerRequestLimit::None,
                    );
                    planet_name = "Rustrelli".to_string()
                }

                _ => {
                    planet_wrapper = rustrelli::create_planet(
                        *planet_id,
                        rx1,
                        tx2,
                        ex2,
                        ExplorerRequestLimit::None,
                    );
                    planet_name = "Rustrelli".to_string()
                }
            }

            let (base, complex) = get_known_resources(virtual_id);
            self.planet_resources.insert(*planet_id, (base, complex));

            planet_type = planet_wrapper.planet_type();

            // Run the individual planet simulation inside an isolated thread sandbox protected from external panic termination cascades
            let handle = thread::spawn(move || {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let res = planet_wrapper.run();
                    match res {
                        Ok(_) => {}
                        Err(err) => {
                            dbg!(err);
                        }
                    }
                }));
            });

            // Store the initialized communication endpoints and join handles into the central orchestrator registries
            self.planet_channels
                .write()
                .unwrap()
                .insert(tmp.value, (tx1, rx2, ex1));
            self.planet_threads.insert(tmp.value, handle);
            map_guard.add_planet(tmp.value, planet_name, planet_type);

            self.start_planet_ai(tmp.value);
        }
    }
}
