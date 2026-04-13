use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};
use common_game::components::resource::{BasicResource, BasicResourceType, ComplexResource, ComplexResourceRequest, ComplexResourceType, GenericResource};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender, select_biased};
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;
use crate::modules::explorer_utils::explorer::Explorer;
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_base::ExplorerBase;
use crate::modules::explorer_utils::handlers::AIHandlers;

pub trait ExplorerPlanetCommunication {
    fn set_to_planet_sender(&mut self, to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>);
    fn set_from_planet_receiver(&mut self, from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>);
}

impl AIHandlers for ManualExplorer {
    fn start_ai_handler(&self) {

    }

    fn reset_ai_handler(&self) {
    }

    fn kill_handler(&self) {
    }

    fn generate_resource_handler(&self, result: &Option<&BasicResource>) {
    }

    fn combine_resource_handler(&self, result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>) {
    }

    fn move_to_planet_handler(&self) {

    }
}

pub struct ManualExplorer {
    // pub explorer_id: ID,
    // pub bag: RwLock<BagType>,
    pub dummy_bag: RwLock<DummyBag>, // aggiunto per visualizer
    // pub current_planet_id: RwLock<ID>,
    // pub stopped: RwLock<bool>,
    // pub alive: RwLock<bool>,
    // pub from_orchestrator: Receiver<OrchestratorToExplorer>,
    // pub to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
    // pub to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>,
    // pub from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>,
    // pub neighbours: RwLock<Vec<ID>>,
    // pub basic_resources: RwLock<HashSet<BasicResourceType>>,
    // pub combinations: RwLock<HashSet<ComplexResourceType>>,
    pub base: ExplorerBase
}

impl ManualExplorer {
    const ERROR_ORCH_DISCONNECTED: &'static str = "Orchestrator disconnected from explorer";

    pub fn new(
        explorer_id: ID,
        current_planet_id: ID,
        from_orchestrator: Receiver<OrchestratorToExplorer>,
        to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
    ) -> Self {
        let base = ExplorerBase::new(
            explorer_id,
            RwLock::new(BagType::new()),
            RwLock::new(current_planet_id),
            to_orchestrator,
            from_orchestrator,
            RwLock::new(None),
            RwLock::new(None),
            RwLock::new(Vec::new()),
            RwLock::new(HashSet::new()),
            RwLock::new(HashSet::new())
        );

        Self {

            dummy_bag: RwLock::new(DummyBag::new(HashMap::new(), HashMap::new())),
            base
        }
    }


    pub fn handle_user_input(&self) {
        sleep(Duration::from_millis(2000));
        println!(
            "Choose what explorer should do:\n[1] Interact with planet     [2] Move to another one       [3]Show Bag"
        );
        let mut tmp = String::new();
        io::stdin()
            .read_line(&mut tmp)
            .expect("Error while reading explorer input");

        let input: u8 = tmp.trim().parse().expect("Please insert a valid option");

        match input {
            1 => {
                println!("[1] Extract Resource\n[2] Craft Resource");
                tmp = String::new();
                io::stdin()
                    .read_line(&mut tmp)
                    .expect("Error while reading explorer input");
                println!("HAI INSERITO {}", tmp);
                let input2: u8 = tmp.trim().parse().expect("Please insert a valid option");
                match input2 {
                    1 => {
                        println!("Choose Basic Resource:");
                        let mut options_list = String::new();
                        let mut options_map = HashMap::new();
                        let guard = self.base.basic_resources.read().unwrap();
                        let mut choices: HashMap<usize, &BasicResourceType> = HashMap::new();

                        for (i, resource) in guard.iter().enumerate() {
                            options_map.insert(i, resource);
                            options_list.push_str(&format!("[{}] {:?}\n", i + 1, resource));
                            choices.insert(i + 1, resource);
                        }

                        println!("{}", options_list);
                        tmp = String::new();
                        io::stdin()
                            .read_line(&mut tmp)
                            .expect("Error while reading explorer input");
                        let choice: usize =
                            tmp.trim().parse().expect("Please insert a valid option");
                        let choice = choices.get(&choice).unwrap();
                        println!("Generating {:?}", choice);
                        self.base.generate_resource(**choice, |arg: &Option<&BasicResource>| self.generate_resource_handler(arg));
                    }

                    2 => {
                        println!("Choose Complex Resource:");
                        let mut options_list = String::new();
                        let mut options_map = HashMap::new();
                        let guard = self.base.combinations.read().unwrap();
                        for (i, resource) in guard.iter().enumerate() {
                            options_map.insert(i + 1, resource);
                            options_list.push_str(&format!("[{}] {:?}\n", i + 1, resource));
                        }
                        println!("{}", options_list);
                        tmp = String::new();
                        io::stdin()
                            .read_line(&mut tmp)
                            .expect("Error while reading explorer input");
                        let input3: usize =
                            tmp.trim().parse().expect("Please insert a valid option");
                        let resource = options_map.get(&input3).unwrap();
                        self.base.combine_resource(**resource, |arg: &Result<&ComplexResource, &(String, GenericResource, GenericResource)> | self.combine_resource_handler(arg));
                    }

                    _ => {
                        println!("CHOOSE A VALID OPTION");
                    }
                }
            }

            2 => {
                tmp = String::new();
                println!(
                    "Decide neighbour to visit: {:?}",
                    self.base.neighbours.read().unwrap()
                );
                io::stdin()
                    .read_line(&mut tmp)
                    .expect("Error while reading explorer input");
                let planet_id: ID = tmp.trim().parse().expect("Please insert a valid option");
                let guard = self.base.neighbours.read().unwrap();

                if guard.contains(&planet_id) {
                    self.base.to_orchestrator
                        .send(ExplorerToOrchestrator::TravelToPlanetRequest {
                            explorer_id: self.base.explorer_id,
                            current_planet_id: *self.base.current_planet_id.read().unwrap(),
                            dst_planet_id: planet_id,
                        })
                        .unwrap();
                } else {
                    println!("The planet selected is not connected to the current planet")
                }
            }

            3 => {
                println!("Show Bag {:?}", self.base.bag.read().unwrap().to_dummy());
            }

            _ => {
                println!("Invalid explorer input");
            }
        }
    }
}

impl ExplorerPlanetCommunication for ManualExplorer {
    fn set_to_planet_sender(&mut self, to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>) {
        self.base.to_planet = to_planet;
    }
    fn set_from_planet_receiver(
        &mut self,
        from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>,
    ) {
        self.base.from_planet = from_planet
    }
}


impl Explorer for ManualExplorer {
    fn get_base(&self) -> &ExplorerBase {
        &self.base
    }
}
