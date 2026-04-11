use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};
use common_game::components::resource::{
    BasicResourceType, ComplexResourceRequest, ComplexResourceType,
};
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

pub trait ExplorerPlanetCommunication {
    fn set_to_planet_sender(&mut self, to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>);
    fn set_from_planet_receiver(&mut self, from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>);
}

pub struct ManualExplorer {
    pub explorer_id: ID,
    pub bag: RwLock<BagType>,
    pub dummy_bag: RwLock<DummyBag>, // aggiunto per visualizer
    pub current_planet_id: RwLock<ID>,
    pub stopped: RwLock<bool>,
    pub alive: RwLock<bool>,
    pub from_orchestrator: Receiver<OrchestratorToExplorer>,
    pub to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
    pub to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>,
    pub from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>,
    pub neighbours: RwLock<Vec<ID>>,
    pub basic_resources: RwLock<HashSet<BasicResourceType>>,
    pub combinations: RwLock<HashSet<ComplexResourceType>>,
}

impl ManualExplorer {
    const ERROR_ORCH_DISCONNECTED: &'static str = "Orchestrator disconnected from explorer";

    pub fn new(
        explorer_id: ID,
        current_planet_id: ID,
        from_orchestrator: Receiver<OrchestratorToExplorer>,
        to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
    ) -> Self {
        Self {
            explorer_id,
            bag: RwLock::new(BagType::new()),
            dummy_bag: RwLock::new(DummyBag::new(HashMap::new(), HashMap::new())),
            current_planet_id: RwLock::new(current_planet_id),
            stopped: RwLock::new(false),
            alive: RwLock::new(true),
            from_orchestrator,
            to_orchestrator,
            to_planet: RwLock::new(None),
            from_planet: RwLock::new(None),
            neighbours: RwLock::new(Vec::new()),
            basic_resources: RwLock::new(HashSet::new()),
            combinations: RwLock::new(HashSet::new()),
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let kill = self.wait_for_start()?;
        if kill {
            return Ok(());
        }

        // self.start_ai();
        println!("Running explorer");
        // self.ask_supported_resources();
        // println!("Planet #{} basic resources: {:?}", self.current_planet_id.read().unwrap(), self.basic_resources);
        // self.set_neighbours();
        // self.handle_user_input();

        loop {
            println!("Explorer {} entered main loop", self.explorer_id);
            // Se è morto, si esce dal loop
            if !*self.alive.read().unwrap() {
                println!("Explorer {} detected death, exiting...", self.explorer_id);
                return Ok(());
            }

            match self.from_orchestrator.recv() {
                Ok(OrchestratorToExplorer::StartExplorerAI) => {
                    // self.start_ai();
                }
                Ok(OrchestratorToExplorer::ResetExplorerAI) => {
                    self.reset_ai();
                }
                Ok(OrchestratorToExplorer::KillExplorer) => {
                    self.kill();
                    return Ok(());
                }

                Ok(OrchestratorToExplorer::MoveToPlanet {
                    sender_to_new_planet,
                    planet_id,
                }) => {
                    println!("Received move to planet msg");
                    self.move_to_planet(sender_to_new_planet, planet_id);
                }
                Ok(OrchestratorToExplorer::CurrentPlanetRequest) => {
                    self.get_current_planet();
                }
                Ok(OrchestratorToExplorer::SupportedResourceRequest) => {
                    self.ask_supported_resources();
                }
                Ok(OrchestratorToExplorer::SupportedCombinationRequest) => {
                    self.ask_combinations();
                }
                Ok(OrchestratorToExplorer::GenerateResourceRequest { to_generate }) => {
                    self.generate_resource(to_generate);
                }
                Ok(OrchestratorToExplorer::CombineResourceRequest {
                    to_generate: _to_generate,
                }) => {
                    let _request: ComplexResourceRequest;
                    // match to_generate {
                    //     ComplexResourceType::Dolphin => { request = ComplexResourceRequest::Dolphin() }
                    //     ComplexResourceType::AIPartner => { request = ComplexResourceRequest::AIPartner()}
                    // }
                    // self.combine_resource(to_generate);
                }
                Ok(OrchestratorToExplorer::BagContentRequest) => {
                    self.get_bag();
                }
                Ok(OrchestratorToExplorer::NeighborsResponse { neighbors }) => {
                    self.set_neighbours(neighbors);
                }
                Err(_) => {
                    return Err(Self::ERROR_ORCH_DISCONNECTED.to_string());
                }
                Ok(OrchestratorToExplorer::StopExplorerAI) => todo!(),
            }
        }
    }

    fn wait_for_start(&self) -> Result<bool, String> {
        loop {
            select_biased! {
                // orch messages
                recv(self.from_orchestrator) -> msg => match msg {
                    // if `Start` is received, return false
                    Ok(OrchestratorToExplorer::StartExplorerAI) => {
                        self.to_orchestrator
                            .send(ExplorerToOrchestrator::StartExplorerAIResult {
                                explorer_id: self.explorer_id,
                            })
                            .map_err(|_| Self::ERROR_ORCH_DISCONNECTED.to_string())?;

                        return Ok(false);
                    }

                    Ok(OrchestratorToExplorer::KillExplorer) => {
                        let mut alive_lock = self.alive.write().unwrap();
                        *alive_lock = false; // aggiunto per visaulizer
                        self.to_orchestrator
                            .send(ExplorerToOrchestrator::KillExplorerResult { explorer_id: self.explorer_id })
                            .map_err(|_| Self::ERROR_ORCH_DISCONNECTED.to_string())?;

                        return Ok(true)
                    }
                    Ok(_) => {
                        self.to_orchestrator
                            .send(ExplorerToOrchestrator::ResetExplorerAIResult {explorer_id: self.explorer_id})
                            .map_err(|_| Self::ERROR_ORCH_DISCONNECTED.to_string())?
                    }

                    Err(_) => return Err(Self::ERROR_ORCH_DISCONNECTED.to_string()),
                },

            }
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
                        let guard = self.basic_resources.read().unwrap();
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
                        self.generate_resource(**choice);
                    }

                    2 => {
                        println!("Choose Complex Resource:");
                        let mut options_list = String::new();
                        let mut options_map = HashMap::new();
                        let guard = self.combinations.read().unwrap();
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
                        self.combine_resource(**resource);
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
                    self.neighbours.read().unwrap()
                );
                io::stdin()
                    .read_line(&mut tmp)
                    .expect("Error while reading explorer input");
                let planet_id: ID = tmp.trim().parse().expect("Please insert a valid option");
                let guard = self.neighbours.read().unwrap();

                if guard.contains(&planet_id) {
                    self.to_orchestrator
                        .send(ExplorerToOrchestrator::TravelToPlanetRequest {
                            explorer_id: self.explorer_id,
                            current_planet_id: *self.current_planet_id.read().unwrap(),
                            dst_planet_id: planet_id,
                        })
                        .unwrap();
                } else {
                    println!("The planet selected is not connected to the current planet")
                }
            }

            3 => {
                println!("Show Bag {:?}", self.bag.read().unwrap().to_dummy());
            }

            _ => {
                println!("Invalid explorer input");
            }
        }
    }
}

impl ExplorerPlanetCommunication for ManualExplorer {
    fn set_to_planet_sender(&mut self, to_planet: RwLock<Option<Sender<ExplorerToPlanet>>>) {
        self.to_planet = to_planet;
    }
    fn set_from_planet_receiver(
        &mut self,
        from_planet: RwLock<Option<Receiver<PlanetToExplorer>>>,
    ) {
        self.from_planet = from_planet
    }
}
