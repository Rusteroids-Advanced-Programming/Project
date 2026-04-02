// use rusteroid_planet::from_generic_type_to_basic;
use crate::modules::manual_explorer::manual_explorer::ManualExplorer;
use common_game::components::resource::{
    BasicResourceType, ComplexResourceRequest, ComplexResourceType,
};
use common_game::protocols::orchestrator_explorer::ExplorerToOrchestrator;
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::Sender;
use std::thread::sleep;
use std::time::Duration;

pub trait ExplorerAI {
    fn start_ai(&self);
    fn reset_ai(&self);
    fn kill(&self);
    fn move_to_planet(&self, to_planet: Option<Sender<ExplorerToPlanet>>, planet_id: ID);
    fn get_current_planet(&self);
    fn give_supported_resources(&self);
    fn ask_supported_resources(&self);
    fn give_combinations(&mut self);
    fn ask_combinations(&self);
    fn generate_resource(&self, to_generate: BasicResourceType);
    fn combine_resource(&self, to_generate: ComplexResourceType);
    fn get_bag(&self);
    fn ask_for_neighbours(&self);
    fn set_neighbours(&self, neighbors: Vec<ID>);
    fn travel_request(&self, dst: u32);
    fn ask_available_cells(&self) -> u32;
    fn get_complex_resource_request(
        &self,
        complex_resource_type: ComplexResourceType,
    ) -> Option<ComplexResourceRequest>;
}

impl ExplorerAI for ManualExplorer {
    fn start_ai(&self) {
        println!("AI STARTED {}", self.explorer_id);
        self.to_orchestrator
            .send(ExplorerToOrchestrator::StartExplorerAIResult {
                explorer_id: self.explorer_id,
            })
            .unwrap();
    }

    fn reset_ai(&self) {
        self.to_orchestrator
            .send(ExplorerToOrchestrator::ResetExplorerAIResult {
                explorer_id: self.explorer_id,
            })
            .unwrap();
    }

    fn kill(&self) {
        self.to_orchestrator
            .send(ExplorerToOrchestrator::KillExplorerResult {
                explorer_id: self.explorer_id,
            })
            .unwrap();
    }

    fn move_to_planet(&self, to_planet: Option<Sender<ExplorerToPlanet>>, planet_id: ID) {
        let mut to_planet_guard = self.to_planet.write().unwrap();
        let mut current_planet_guard = self.current_planet_id.write().unwrap();
        *to_planet_guard = to_planet;
        *current_planet_guard = planet_id;

        println!(
            "Explorer #{} moved to planet #{}",
            self.explorer_id, planet_id
        );
        self.to_orchestrator
            .send(ExplorerToOrchestrator::MovedToPlanetResult {
                explorer_id: self.explorer_id,
                planet_id,
            })
            .unwrap();

        drop(current_planet_guard);
        drop(to_planet_guard);

        self.ask_for_neighbours();
        sleep(Duration::from_millis(500));
        //println!("neighbors: {:?}", self.neighbours.read().unwrap());
        self.ask_combinations();
        //println!("after combinatiopns");
        self.ask_supported_resources();
        //println!("resourcces: {:?}", self.basic_resources)
    }

    fn get_current_planet(&self) {
        let current_planet_id_guard = self.current_planet_id.read().unwrap();
        self.to_orchestrator
            .send(ExplorerToOrchestrator::CurrentPlanetResult {
                explorer_id: self.explorer_id,
                planet_id: *current_planet_id_guard,
            })
            .unwrap();
    }

    fn give_supported_resources(&self) {
        let guard = self.basic_resources.read().unwrap();
        self.ask_supported_resources();
        self.to_orchestrator
            .send(ExplorerToOrchestrator::SupportedResourceResult {
                explorer_id: self.explorer_id,
                supported_resources: guard.clone(),
            })
            .unwrap();
    }

    fn ask_supported_resources(&self) {
        let to_planet_guard = self.to_planet.read().unwrap();
        let from_planet_guard = self.from_planet.read().unwrap();

        if let Some(to_planet) = to_planet_guard.as_ref() {
            to_planet
                .send(ExplorerToPlanet::SupportedResourceRequest {
                    explorer_id: self.explorer_id,
                })
                .unwrap();
            println!("From_planet {:?}", self.from_planet);

            if let Some(from_planet) = from_planet_guard.as_ref() {
                //println!("Before message");
                let msg = from_planet.recv().unwrap();
                //println!("After message");
                match msg {
                    PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                        //println!("Resource list: {:?}", resource_list);
                        let mut guard = self.basic_resources.write().unwrap();
                        *guard = resource_list;
                    }
                    msg => {
                        println!(
                            "Received unexpected message: {:?} while waiting for SupportedResourceResponse",
                            msg
                        );
                    }
                }
            } else {
                println!("channel from_plResource Listanet has been dropped");
            }
        } else {
            println!("channel to_planet has been dropped");
        }
    }

    fn give_combinations(&mut self) {
        self.ask_combinations();
        let combinations_guard = self.combinations.read().unwrap();
        self.to_orchestrator
            .send(ExplorerToOrchestrator::SupportedCombinationResult {
                explorer_id: self.explorer_id,
                combination_list: combinations_guard.clone(),
            })
            .unwrap();
    }

    fn ask_combinations(&self) {
        let to_planet_guard = self.to_planet.read().unwrap();
        let from_planet_guard = self.from_planet.read().unwrap();

        if let Some(to_planet) = &*to_planet_guard {
            to_planet
                .send(ExplorerToPlanet::SupportedCombinationRequest {
                    explorer_id: self.explorer_id,
                })
                .unwrap();
            if let Some(from_planet) = &*from_planet_guard {
                //println!("askcombinations");
                let msg = from_planet
                    .recv_timeout(Duration::from_millis(2000))
                    .unwrap(); // -> proviamo a mettere forzatamente il nostro pianeta e vedere se funziona( a patto che noi gestiamo bene questo messaggio)
                //println!("FATTOASKEDDIOBELLO");
                match msg {
                    PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                        let mut guard = self.combinations.write().unwrap();
                        *guard = combination_list;
                    }
                    _ => {}
                }
            }
        }
    }

    fn generate_resource(&self, to_generate: BasicResourceType) {
        let to_planet_guard = self.to_planet.read().unwrap();
        let from_planet_guard = self.from_planet.read().unwrap();
        if let Some(to_planet) = to_planet_guard.as_ref() {
            to_planet
                .send(ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id: self.explorer_id,
                    resource: to_generate,
                })
                .unwrap();
            if let Some(from_planet) = from_planet_guard.as_ref() {
                let msg = from_planet.recv().unwrap();
                let result: Result<(), String>;
                match msg {
                    PlanetToExplorer::GenerateResourceResponse { resource } => match resource {
                        Some(resource) => {
                            println!("Generated basic resource: {:?}", resource);
                            let mut bag_guard = self.bag.write().unwrap();
                            bag_guard.add_basic_resource(resource);
                            result = Ok(());
                        }
                        None => {
                            result = Err(format!("Failed to gen {:?}", to_generate));
                        }
                    },
                    msg => {
                        result = Err(format!(
                            "Unexpected message while wating for Generate Resource Response {:?}",
                            msg
                        ));
                    }
                }

                self.to_orchestrator
                    .send(ExplorerToOrchestrator::GenerateResourceResponse {
                        explorer_id: self.explorer_id,
                        generated: result,
                    })
                    .unwrap();
            }
        }
    }

    fn combine_resource(&self, to_generate: ComplexResourceType) {
        let generate_request = self.get_complex_resource_request(to_generate);
        let to_planet_guard = self.to_planet.read().unwrap();
        let from_planet_guard = self.from_planet.read().unwrap();
        if let Some(to_planet) = to_planet_guard.as_ref() {
            to_planet
                .send(ExplorerToPlanet::CombineResourceRequest {
                    explorer_id: self.explorer_id,
                    msg: generate_request.unwrap(),
                })
                .unwrap();
            if let Some(from_planet) = from_planet_guard.as_ref() {
                let msg = from_planet.recv().unwrap();
                let result: Result<(), String>;
                match msg {
                    PlanetToExplorer::CombineResourceResponse { complex_response } => {
                        match complex_response {
                            Ok(resource) => {
                                let mut bag_guard = self.bag.write().unwrap();
                                bag_guard.add_complex_resource(resource);
                                result = Ok(());
                            }
                            Err((_e, _res1, _res2)) => {
                                result = Err("Couldn't combine resource".to_string());
                                // self.bag.add_to_bag(ResourceType::Basic(from_generic_type_to_basic(res1).unwrap()));
                                // self.bag.add_to_bag(ResourceType::Basic(from_generic_type_to_basic(res2).unwrap()));
                            }
                        }
                    }
                    _ => {
                        result = Err("Couldn't combine resource".to_string());
                    }
                }

                self.to_orchestrator
                    .send(ExplorerToOrchestrator::CombineResourceResponse {
                        explorer_id: self.explorer_id,
                        generated: result,
                    })
                    .unwrap();
            }
        }
    }

    fn get_bag(&self) {
        self.to_orchestrator
            .send(ExplorerToOrchestrator::BagContentResponse {
                explorer_id: self.explorer_id,
                bag_content: self.bag.read().unwrap().to_dummy(),
            })
            .unwrap();
    }

    fn ask_for_neighbours(&self) {
        //necessario debug todo()
        let current_planet_id_guard = self.current_planet_id.read().unwrap();
        self.to_orchestrator
            .send(ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: self.explorer_id,
                current_planet_id: *current_planet_id_guard,
            })
            .unwrap();
        // let msg = self.from_orchestrator.recv_timeout(Duration::from_millis(2000)).unwrap();// panica
        // match msg {
        //     OrchestratorToExplorer::NeighborsResponse {neighbors} => {
        //         let mut guard = self.neighbours.write().unwrap();
        //         *guard = neighbors;
        //     }
        //     _ => {}
        // }
    }

    fn set_neighbours(&self, neighbors: Vec<ID>) {
        //println!("Setting neighbors: {:?}",neighbors);
        let mut neighbours_guard = self.neighbours.write().unwrap();
        *neighbours_guard = neighbors;
    }

    fn travel_request(&self, dst: u32) {
        let current_planet_id_guard = self.current_planet_id.read().unwrap();
        self.to_orchestrator
            .send(ExplorerToOrchestrator::TravelToPlanetRequest {
                explorer_id: self.explorer_id,
                current_planet_id: *current_planet_id_guard,
                dst_planet_id: dst,
            })
            .unwrap();
    }

    fn ask_available_cells(&self) -> u32 {
        let mut available = 0;

        let to_planet_guard = self.to_planet.read().unwrap();
        let from_planet_guard = self.from_planet.read().unwrap();
        if let Some(to_planet) = to_planet_guard.as_ref() {
            to_planet
                .send(ExplorerToPlanet::AvailableEnergyCellRequest {
                    explorer_id: self.explorer_id,
                })
                .unwrap();
            if let Some(from_planet) = from_planet_guard.as_ref() {
                let msg = from_planet.recv().unwrap();
                match msg {
                    PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                        available = available_cells;
                    }
                    _ => {}
                }
            }
        }

        available
    }

    fn get_complex_resource_request(
        &self,
        complex_resource_type: ComplexResourceType,
    ) -> Option<ComplexResourceRequest> {
        let mut guard = self.bag.write().unwrap();
        match complex_resource_type {
            ComplexResourceType::AIPartner => {
                let tmp = guard.get_complex_ingredients(
                    ComplexResourceType::Diamond,
                    ComplexResourceType::Robot,
                );
                match tmp {
                    Some((lhs, rhs)) => Some(ComplexResourceRequest::AIPartner(
                        rhs.to_robot().unwrap(),
                        lhs.to_diamond().unwrap(),
                    )),
                    None => None,
                }
            }

            ComplexResourceType::Robot => {
                let tmp = guard.get_diff_type_ingredients(
                    BasicResourceType::Silicon,
                    ComplexResourceType::Life,
                );
                match tmp {
                    Some((lhs, rhs)) => Some(ComplexResourceRequest::Robot(
                        lhs.to_silicon().unwrap(),
                        rhs.to_life().unwrap(),
                    )),
                    None => None,
                }
            }

            ComplexResourceType::Diamond => {
                let tmp = guard
                    .get_basic_ingredients(BasicResourceType::Carbon, BasicResourceType::Carbon);
                match tmp {
                    Some((lhs, rhs)) => Some(ComplexResourceRequest::Diamond(
                        lhs.to_carbon().unwrap(),
                        rhs.to_carbon().unwrap(),
                    )),
                    None => None,
                }
            }

            ComplexResourceType::Water => {
                let tmp = guard
                    .get_basic_ingredients(BasicResourceType::Hydrogen, BasicResourceType::Oxygen);
                match tmp {
                    Some((lhs, rhs)) => Some(ComplexResourceRequest::Water(
                        lhs.to_hydrogen().unwrap(),
                        rhs.to_oxygen().unwrap(),
                    )),
                    None => None,
                }
            }

            ComplexResourceType::Life => {
                let tmp = guard.get_diff_type_ingredients(
                    BasicResourceType::Carbon,
                    ComplexResourceType::Water,
                );
                match tmp {
                    Some((lhs, rhs)) => Some(ComplexResourceRequest::Life(
                        rhs.to_water().unwrap(),
                        lhs.to_carbon().unwrap(),
                    )),
                    None => None,
                }
            }

            ComplexResourceType::Dolphin => {
                let tmp = guard
                    .get_complex_ingredients(ComplexResourceType::Water, ComplexResourceType::Life);
                match tmp {
                    Some((lhs, rhs)) => Some(ComplexResourceRequest::Dolphin(
                        lhs.to_water().unwrap(),
                        rhs.to_life().unwrap(),
                    )),
                    None => None,
                }
            }
        }
    }
}
