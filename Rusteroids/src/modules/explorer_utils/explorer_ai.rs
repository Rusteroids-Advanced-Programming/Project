// use rusteroid_planet::from_generic_type_to_basic;
use crate::modules::manual_explorer::manual_explorer::ManualExplorer;
use common_game::components::resource::{BasicResource, BasicResourceType, ComplexResource, ComplexResourceRequest, ComplexResourceType, GenericResource};
use common_game::protocols::orchestrator_explorer::ExplorerToOrchestrator;
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;
use crate::modules::explorer_utils::explorer_base::ExplorerBase;

fn send_message_to_planet_with_timeout(to_planet: &Sender<ExplorerToPlanet>, from_planet:&Receiver<PlanetToExplorer>, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
    let res_sent = to_planet.send(msg);
    match res_sent {
        Ok(_) => {
            let response = from_planet.recv_timeout(Duration::from_millis(5000));
            match response {
                Ok(response) => {
                    Some(response)
                }
                Err(e) => {
                    // println!("IL pianeta #{} non ha risposto entro il timeout (per me può esplodere)", planet_id);
                    None
                }
            }
        },
        Err(_) => {
            return None;
        }
    }
}

pub trait ExplorerAI {
    fn start_ai <F> (&self, custom_handler: F) where F : Fn();
    fn reset_ai <F> (&self, custom_handler: F) where F : Fn();
    fn kill <F> (&self, custom_handler: F) where F : Fn();
    fn move_to_planet <F> (
        &self,
        to_planet: Option<Sender<ExplorerToPlanet>>,
        planet_id: ID,
        custom_handler: F
    ) where F : Fn();
    fn get_current_planet(&self);
    fn give_supported_resources(&self);
    fn ask_supported_resources(&self);
    fn give_combinations(&mut self);
    fn ask_combinations(&self);
    fn generate_resource <F> (&self, to_generate: BasicResourceType, custom_handler: F) -> Result<(), String> where F : Fn(&Option<&BasicResource>);
    fn combine_resource <F> (&self, to_generate: ComplexResourceType, custom_handler: F) where F : Fn(&Result<&ComplexResource, &(String, GenericResource, GenericResource)>);
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

impl ExplorerAI for ExplorerBase {
    fn start_ai <F> (&self, custom_handler: F) where F: Fn() {
        println!("AI STARTED {}", self.explorer_id);
        self.to_orchestrator
            .send(ExplorerToOrchestrator::StartExplorerAIResult {
                explorer_id: self.explorer_id,
            })
            .unwrap();

        custom_handler();
    }

    fn reset_ai <F> (&self, custom_handler: F) where F: Fn() {
        self.to_orchestrator
            .send(ExplorerToOrchestrator::ResetExplorerAIResult {
                explorer_id: self.explorer_id,
            })
            .unwrap();

        custom_handler();
    }

    fn kill <F> (&self, custom_handler: F) where F: Fn() {
        self.to_orchestrator
            .send(ExplorerToOrchestrator::KillExplorerResult {
                explorer_id: self.explorer_id,
            })
            .unwrap();

        custom_handler();
    }

    fn move_to_planet <F> (&self, to_planet: Option<Sender<ExplorerToPlanet>>, planet_id: ID, custom_handler : F) where F : Fn(){
        println!("MOVE TO_PLANET {}", planet_id);

        let mut to_planet_guard = self.to_planet.write().unwrap();
        println!("to_planet lock ricevuto");
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
        // sleep(Duration::from_millis(500));
        //println!("neighbors: {:?}", self.neighbours.read().unwrap());
        self.ask_combinations();
        //println!("after combinatiopns");
        self.ask_supported_resources();
        //println!("resourcces: {:?}", self.basic_resources)
        custom_handler()
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

        if let Some(to_planet) = to_planet_guard.as_ref() && let Some(from_planet) = from_planet_guard.as_ref() {
            let resp = send_message_to_planet_with_timeout(to_planet, from_planet, ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: self.explorer_id,
            });

            match resp {
                Some(msg) => {
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
                }

                None => {}
            }
        }

        // if let Some(to_planet) = to_planet_guard.as_ref() {
        //     let msg_res = to_planet.send(ExplorerToPlanet::SupportedResourceRequest {
        //             explorer_id: self.explorer_id,
        //         });
        //
        //     match msg_res {
        //         Err(e) => {
        //             println!("Pianeta esploso finchè gli chiedevo le basic resources");
        //         }
        //         Ok(()) => {
        //             println!("From_planet {:?}", self.from_planet);
        //
        //             if let Some(from_planet) = from_planet_guard.as_ref() {
        //                 //println!("Before message");
        //                 let msg = from_planet.recv().unwrap();
        //                 //println!("After message");
        //                 match msg {
        //                     PlanetToExplorer::SupportedResourceResponse { resource_list } => {
        //                         //println!("Resource list: {:?}", resource_list);
        //                         let mut guard = self.basic_resources.write().unwrap();
        //                         *guard = resource_list;
        //                     }
        //                     msg => {
        //                         println!(
        //                             "Received unexpected message: {:?} while waiting for SupportedResourceResponse",
        //                             msg
        //                         );
        //                     }
        //                 }
        //             } else {
        //                 println!("channel from_plResource Listanet has been dropped");
        //             }
        //         }
        //     }
        // } else {
        //     println!("channel to_planet has been dropped");
        // }
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

        if let Some(to_planet) = to_planet_guard.as_ref() && let Some(from_planet) = from_planet_guard.as_ref() {
            let resp = send_message_to_planet_with_timeout(to_planet, from_planet, ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: self.explorer_id,
            });

            match resp {
                Some(msg) => {
                    match msg {
                        PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                            let mut guard = self.combinations.write().unwrap();
                            *guard = combination_list;
                        }
                        msg => {
                            println!("Received unexpected message: {:?} while waiting for SupportedCombinationResponse", msg);
                        }
                    }
                }
                None => {}
            }
        }

        // if let Some(to_planet) = &*to_planet_guard {
        //     let msg_res = to_planet.send(ExplorerToPlanet::SupportedCombinationRequest {
        //             explorer_id: self.explorer_id,
        //         });
        //     match msg_res {
        //         Err(e) => {
        //             println!("Pianeta esploso finchè chiedevo le recipes")
        //         }
        //         Ok(()) => {
        //             if let Some(from_planet) = &*from_planet_guard {
        //                 //println!("askcombinations");
        //                 let msg = from_planet
        //                     .recv_timeout(Duration::from_millis(2000))
        //                     .unwrap(); // -> proviamo a mettere forzatamente il nostro pianeta e vedere se funziona( a patto che noi gestiamo bene questo messaggio)
        //                 match msg {
        //                     PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
        //                         let mut guard = self.combinations.write().unwrap();
        //                         *guard = combination_list;
        //                     }
        //                     msg => {
        //                         println!("Received unexpected message: {:?} while waiting for SupportedCombinationResponse", msg);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
        // else {
        //     println!("channel to_planet has been dropped");
        // }
    }

    fn generate_resource <F> (&self, to_generate: BasicResourceType, custom_handler: F) -> Result<(), String>
    where F: Fn(&Option<&BasicResource>){
        let to_planet_guard = self.to_planet.read().unwrap();
        let from_planet_guard = self.from_planet.read().unwrap();

        if let Some(to_planet) = to_planet_guard.as_ref() && let Some(from_planet) = from_planet_guard.as_ref() {
            let resp = send_message_to_planet_with_timeout(to_planet, from_planet, ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: self.explorer_id,
                resource: to_generate,
            });

            match resp {
                Some(msg) => {
                    let result: Result<(), String>;
                    match msg {
                        PlanetToExplorer::GenerateResourceResponse { resource } => {
                            let tmp = &resource.as_ref();
                            custom_handler(tmp);

                            println!("DEBUG: RICEVUTA RESPONSE DALLA GENERAZIONE: {:?}", resource);

                            match resource {
                                Some(resource) => {
                                    println!("Generated basic resource: {:?}", resource);
                                    let mut bag_guard = self.bag.write().unwrap();
                                    bag_guard.add_basic_resource(resource);
                                    result = Ok(());
                                }
                                None => {
                                    result = Err(format!("Failed to gen {:?}", to_generate));
                                }
                            }
                        },
                        msg => {
                            println!("Received unexpected message: {:?} while waiting for GenerateResourceResponse", msg);

                            result = Err(format!(
                                "Unexpected message while wating for Generate Resource Response {:?}",
                                msg
                            ));
                        }
                    }

                    self.to_orchestrator
                        .send(ExplorerToOrchestrator::GenerateResourceResponse {
                            explorer_id: self.explorer_id,
                            generated: result.clone(),
                        })
                        .unwrap();

                    result
                }
                None => {
                    Err(format!("Failed to generate resource {:?}, planet exceeded timeout", to_generate))
                }
            }
        }

        else {
            Err(format!("Failed to generate resource {:?}, planet channels are dead", to_generate))
        }
    }


    fn combine_resource <F> (&self, to_generate: ComplexResourceType, custom_handler: F) where F: Fn(&Result<&ComplexResource, &(String, GenericResource, GenericResource)>){
        let generate_request = self.get_complex_resource_request(to_generate);
        
        println!("Combine resource request {:?}", generate_request);
        
        let to_planet_guard = self.to_planet.read().unwrap();
        let from_planet_guard = self.from_planet.read().unwrap();

        if let Some(to_planet) = to_planet_guard.as_ref() && let Some(from_planet) = from_planet_guard.as_ref() {
            if let Some(generate_request) = generate_request {
                let resp = send_message_to_planet_with_timeout(to_planet, from_planet, ExplorerToPlanet::CombineResourceRequest {
                    explorer_id: self.explorer_id,
                    msg: generate_request,
                });

                match resp {
                    Some(msg) => {
                        let result: Result<(), String>;
                        match msg {
                            PlanetToExplorer::CombineResourceResponse { complex_response } => {

                                let tmp = &complex_response.as_ref();
                                custom_handler(tmp);

                                match complex_response {
                                    Ok(resource) => {
                                        // println!("DEBUG: Combined resource successfully: {:?}", resource);
                                        let mut bag_guard = self.bag.write().unwrap();
                                        bag_guard.add_complex_resource(resource);
                                        result = Ok(());
                                    }
                                    Err((e, res1, res2)) => {
                                        // println!("DEBUG: Could not generate using {:?} + {:?}: {}", res1, res2, e);

                                        result = Err("Couldn't combine resource".to_string());
                                        // self.bag.add_to_bag(ResourceType::Basic(from_generic_type_to_basic(res1).unwrap()));
                                        // self.bag.add_to_bag(ResourceType::Basic(from_generic_type_to_basic(res2).unwrap()));
                                    }
                                }
                            }
                            msg => {
                                println!("Received unexpected message: {:?} while waiting for CombineResourceResponse", msg);
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

                None => {}

                }
            }
        }
    }

        // if let Some(to_planet) = to_planet_guard.as_ref() {
        //     if let Some(generate_request) = generate_request {
        //         to_planet
        //             .send(ExplorerToPlanet::CombineResourceRequest {
        //                 explorer_id: self.explorer_id,
        //                 msg: generate_request,
        //             })
        //             .unwrap();
        //         if let Some(from_planet) = from_planet_guard.as_ref() {
        //             let msg = from_planet.recv().unwrap();
        //             let result: Result<(), String>;
        //             match msg {
        //                 PlanetToExplorer::CombineResourceResponse { complex_response } => {
        //
        //                     let tmp = &complex_response.as_ref();
        //                     custom_handler(tmp);
        //
        //                     match complex_response {
        //                         Ok(resource) => {
        //                             println!("DEBUG: Combined resource successfully: {:?}", resource);
        //                             let mut bag_guard = self.bag.write().unwrap();
        //                             bag_guard.add_complex_resource(resource);
        //                             result = Ok(());
        //                         }
        //                         Err((e, res1, res2)) => {
        //                             println!("DEBUG: Could not generate using {:?} + {:?}: {}", res1, res2, e);
        //
        //                             result = Err("Couldn't combine resource".to_string());
        //                             // self.bag.add_to_bag(ResourceType::Basic(from_generic_type_to_basic(res1).unwrap()));
        //                             // self.bag.add_to_bag(ResourceType::Basic(from_generic_type_to_basic(res2).unwrap()));
        //                         }
        //                     }
        //                 }
        //                 msg => {
        //                     println!("Received unexpected message: {:?} while waiting for CombineResourceResponse", msg);
        //                     result = Err("Couldn't combine resource".to_string());
        //                 }
        //             }
        //
        //             self.to_orchestrator
        //                 .send(ExplorerToOrchestrator::CombineResourceResponse {
        //                     explorer_id: self.explorer_id,
        //                     generated: result,
        //                 })
        //                 .unwrap();
    //             }
    //         }
    //     }
    // }

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

        if let Some(to_planet) = to_planet_guard.as_ref() && let Some(from_planet) = from_planet_guard.as_ref() {
            let resp = send_message_to_planet_with_timeout(to_planet, from_planet, ExplorerToPlanet::AvailableEnergyCellRequest {
                explorer_id: self.explorer_id,
            });

            match resp {
                Some(msg) => {
                    match msg {
                        PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                            available = available_cells;
                        }
                        msg => {
                            println!("Unexpected message received while waiting for AvailableEnergyCellResponse: {:?}", msg);
                        }
                    }
                }
                None => {}
            }
        }

        // if let Some(to_planet) = to_planet_guard.as_ref() {
        //     to_planet
        //         .send(ExplorerToPlanet::AvailableEnergyCellRequest {
        //             explorer_id: self.explorer_id,
        //         })
        //         .unwrap();
        //     if let Some(from_planet) = from_planet_guard.as_ref() {
        //         let msg = from_planet.recv().unwrap();
        //         match msg {
        //             PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
        //                 available = available_cells;
        //             }
        //             msg => {
        //                 println!("Unexpected message received while waiting for AvailableEnergyCellResponse: {:?}", msg);
        //             }
        //         }
        //     }
        // }

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
