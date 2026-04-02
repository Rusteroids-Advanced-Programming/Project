// impl PlanetAI for RusteroidAI {
//     fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
//
//         // Initialization of Payload for OrchestratorToPlanet logs
//         let mut payload: BTreeMap<String, String> = BTreeMap::new();
//         payload.insert("planet_id".into(), state.id().to_string().into());
//         payload.insert("system".into(), "planet_ai".into());
//
//         match msg {
//             OrchestratorToPlanet::Sunray(sunray) => {
//
//                 payload.insert("event".into(), "sunray_ack".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Warning, payload);
//                     Some(PlanetToOrchestrator::Stopped {planet_id: state.id()})
//                 }
//                 else {
//                     self.sunray_count += 1;
//                     println!("Sunray count: {}", self.sunray_count);
//
//                     state.charge_cell(sunray);
//                     let (cell, i) = state.full_cell().unwrap();
//                     state.build_rocket(i);
//                     println!("Has rocket: {:?}", state.has_rocket());
//                     let mut charged = 0;
//                     for cell in state.cells_iter(){
//                         if cell.is_charged() {
//                             charged += 1;
//                         }
//                     }
//                     println!("Charged: {}", charged);
//
//                     payload.insert("result".into(), "success".into());
//                     payload.insert("resource".into(), "energy_cell".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Debug, payload);
//                     Some(PlanetToOrchestrator::SunrayAck {planet_id: state.id()})
//                 }
//             }
//
//             OrchestratorToPlanet::Asteroid(asteroid) => {
//                 // let destroyed:bool;
//                 // match state.take_rocket() {      MAYBE CAN BE USEFUL
//                 //     Some(rocket) => {
//                 //         destroyed = false;
//                 //     }
//                 //     None => {
//                 //         destroyed = true;
//                 //         self.destroyed = true
//                 //     }
//                 // }
//
//                 payload.insert("event".into(), "asteroid_ack".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Warning, payload);
//                     Some(PlanetToOrchestrator::Stopped {planet_id: state.id()})
//                 }
//                 else {
//
//                     self.asteroid_count+=1;
//                     let rocket = self.handle_asteroid(state, generator, combinator);
//
//                     payload.insert("result".into(), "success".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Debug, payload);
//                     Some(PlanetToOrchestrator::AsteroidAck {planet_id: state.id(), rocket})
//                 }
//             }
//
//             OrchestratorToPlanet::StartPlanetAI => {
//
//                 self.start(state);
//
//                 payload.insert("event".into(), "start_planet".into());
//                 payload.insert("result".into(), "success".into());
//                 create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Debug, payload);
//
//                 return Some(PlanetToOrchestrator::StartPlanetAIResult {planet_id: state.id()})
//             }
//
//             OrchestratorToPlanet::StopPlanetAI => {
//                 self.stop(state);
//
//                 payload.insert("event".into(), "stop_planet".into());
//                 payload.insert("result".into(), "success".into());
//                 create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Debug, payload);
//
//                 return Some(PlanetToOrchestrator::StopPlanetAIResult {planet_id: state.id()})
//             }
//
//             OrchestratorToPlanet::InternalStateRequest => {
//
//                 payload.insert("event".into(), "InternalStateRequest".into());
//
//                 if self.stopped {
//                     payload.insert("state".into(), "stopped".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Warning, payload);
//                     Some(PlanetToOrchestrator::Stopped {planet_id: state.id()})
//                 }
//                 else {
//                     payload.insert("state".into(), "running".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Trace, payload);
//                     return Some(PlanetToOrchestrator::InternalStateResponse {planet_id: state.id(), planet_state: state.to_dummy()})
//                 }
//             }
//
//             OrchestratorToPlanet::IncomingExplorerRequest {explorer_id, new_mpsc_sender} => {
//
//                 payload.insert("event".into(), "explorer_request_ack".into());
//
//                 let res: Result<(), String>;
//                 if !self.destroyed {
//                     res = Ok(());
//                 }
//                 else {
//                     res = Err("Planet is destroyed, could not reach it".to_string());
//                 }
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Warning, payload);
//
//                     Some(PlanetToOrchestrator::Stopped {planet_id: state.id()})
//                 }
//                 else {
//                     payload.insert("result".into(), "success".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Debug, payload);
//
//                     return Some(PlanetToOrchestrator::IncomingExplorerResponse { planet_id: state.id(), res })
//                 }
//             }
//
//             OrchestratorToPlanet::OutgoingExplorerRequest {explorer_id} => {
//
//                 payload.insert("event".into(), "OutgoingExplorerRequest_ack".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Warning, payload);
//
//                     Some(PlanetToOrchestrator::Stopped {planet_id: state.id()})
//                 }
//                 else {
//                     payload.insert("result".into(), "success".into());
//                     create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Debug, payload);
//
//                     return Some(PlanetToOrchestrator::OutgoingExplorerResponse {planet_id: state.id(), res: Ok(())})
//                 }
//
//             }
//
//             OrchestratorToPlanet::KillPlanet => {
//
//                 println!("destroyed {}", self.destroyed);
//
//                 payload.insert("event".into(), "kill_planet".into());
//                 payload.insert("result".into(), "success".into());
//                 create_and_send_log(state.id() as u64,Orchestrator,"Orchestrator".to_string(), MessagePlanetToOrchestrator, Debug, payload);
//
//                 return Some(PlanetToOrchestrator::KillPlanetResult {planet_id: state.id()})
//             }
//
//         }
//     }
//
//     fn handle_explorer_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
//
//         // Initialization of Payload for ExplorerToPlanet logs
//         let mut payload: BTreeMap<String, String> = BTreeMap::new();
//         payload.insert("planet_id".into(), state.id().to_string().into());
//         payload.insert("system".into(), "planet_ai".into());
//
//         match msg {
//             ExplorerToPlanet::SupportedResourceRequest {explorer_id} => {
//                 payload.insert("event".into(), "supported_resource_req".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Warning, payload);
//
//                     Some(PlanetToExplorer::Stopped)
//                 }
//                 else {
//                     payload.insert("result".into(), "success".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                     Some(PlanetToExplorer::SupportedResourceResponse { resource_list: generator.all_available_recipes()})
//                 }
//             }
//
//             ExplorerToPlanet::SupportedCombinationRequest {explorer_id} => {
//                 payload.insert("event".into(), "supported_combination_req".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Warning, payload);
//
//                     Some(PlanetToExplorer::Stopped)
//                 } else {
//                     payload.insert("result".into(), "success".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                     Some(PlanetToExplorer::SupportedCombinationResponse { combination_list: combinator.all_available_recipes() })
//                 }
//             }
//
//             ExplorerToPlanet::GenerateResourceRequest {explorer_id, resource} => {
//                 payload.insert("event".into(), "gen_resource_req".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Warning, payload);
//
//                     Some(PlanetToExplorer::Stopped)
//                 } else {
//                     let resource_list = generator.all_available_recipes();
//                     let result: Option<BasicResource>;
//
//                     match resource_list.get(&resource) {
//                         None => {
//                             payload.insert("result".into(), "error".into());
//                             payload.insert("reason".into(), "no_such_recipe".into());
//                             create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//                             result = None;
//                         }
//                         Some(resource) => {
//                             let cell = state.full_cell();
//                             match cell {
//                                 None => {
//                                     payload.insert("result".into(), "error".into());
//                                     payload.insert("reason".into(), "no_energy".into());
//                                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                                     result = None;
//                                 }
//                                 Some((c, i)) => {
//                                     payload.insert("result".into(), "success".into());
//                                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                                     result = gen_basic_resource(generator, *resource, c);
//                                 }
//                             }
//                         }
//                     }
//
//                     Some(PlanetToExplorer::GenerateResourceResponse {resource: result})
//                 }
//             }
//
//             ExplorerToPlanet::CombineResourceRequest {explorer_id, msg} => {
//                 payload.insert("event".into(), "comb_resource_req".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Warning, payload);
//
//                     Some(PlanetToExplorer::Stopped)
//                 } else {
//                     let error_message = "This awesome planet can't combine resources!".to_string();
//
//                     let var_lhs: GenericResource;
//                     let var_rhs: GenericResource;
//                     let complex_type: ComplexResourceType;
//
//                     (var_lhs, var_rhs, complex_type) = match msg {
//                         ComplexResourceRequest::Water(lhs, rhs) => (lhs.to_generic(), rhs.to_generic(), ComplexResourceType::Water),
//                         ComplexResourceRequest::Diamond(lhs, rhs) => (lhs.to_generic(), rhs.to_generic(), ComplexResourceType::Diamond),
//                         ComplexResourceRequest::Life(lhs, rhs) => (lhs.to_generic(), rhs.to_generic(), ComplexResourceType::Life),
//                         ComplexResourceRequest::Robot(lhs, rhs) => (lhs.to_generic(), rhs.to_generic(), ComplexResourceType::Robot),
//                         ComplexResourceRequest::Dolphin(lhs, rhs) => (lhs.to_generic(), rhs.to_generic(), ComplexResourceType::Dolphin),
//                         ComplexResourceRequest::AIPartner(lhs, rhs) => (lhs.to_generic(), rhs.to_generic(), ComplexResourceType::AIPartner),
//                     };
//
//                     if combinator.all_available_recipes().len() == 0 {
//                         payload.insert("result".into(), "error".into());
//                         payload.insert("reason".into(), "no_available_recipes".into());
//                         create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                         Some(PlanetToExplorer::CombineResourceResponse { complex_response: Err((error_message, var_lhs, var_rhs)) })
//                     } else {
//                         if combinator.contains(complex_type) {
//                             let cell = state.full_cell();
//                             match cell {
//                                 None => {
//                                     payload.insert("result".into(), "error".into());
//                                     payload.insert("reason".into(), "no_energy".into());
//                                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                                     let error_no_energy = "Not available energy to craft".to_string();
//                                     Some(PlanetToExplorer::CombineResourceResponse { complex_response: Err((error_no_energy, var_lhs, var_rhs)) })
//                                 }
//                                 Some((c, i)) => {
//                                     payload.insert("result".into(), "success".into());
//                                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                                     let result = gen_complex_resource(combinator, complex_type, c, var_lhs, var_rhs).unwrap();
//                                     Some(PlanetToExplorer::CombineResourceResponse { complex_response: Ok(result) })
//                                     // match result {
//                                     //     Some(r) => {
//                                     //         Some(PlanetToExplorer::CombineResourceResponse {complex_response: Ok(r)})
//                                     //     }
//                                     //     None => {
//                                     //         let error_craft = "Error while crafting complex resource".to_string();
//                                     //         Some(PlanetToExplorer::CombineResourceResponse {complex_response: Err((error_craft, var_lhs, var_rhs))})
//                                     //     }
//                                     // }
//                                 }
//                             }
//                         } else {
//                             payload.insert("result".into(), "error".into());
//                             payload.insert("reason".into(), "no_such_recipe".into());
//                             create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Debug, payload);
//
//                             let error_no_recipe = "This planet can't craft that resource".to_string();
//                             Some(PlanetToExplorer::CombineResourceResponse {complex_response: Err((error_no_recipe, var_lhs, var_rhs))})
//                         }
//                     }
//                 }
//             }
//
//             ExplorerToPlanet::AvailableEnergyCellRequest {explorer_id} => {
//                 payload.insert("event".into(), "available_e_cell_req".into());
//
//                 if self.stopped {
//                     payload.insert("result".into(), "error".into());
//                     payload.insert("reason".into(), "planet_stopped".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Warning, payload);
//
//                     Some(PlanetToExplorer::Stopped)
//                 } else {
//                     payload.insert("result".into(), "success".into());
//                     create_and_send_log(state.id() as u64, Explorer, "Explorer".to_string(), MessagePlanetToExplorer, Trace, payload);
//
//                     let cell_amount: u32 = state.cells_count() as u32;
//                     Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: cell_amount })
//                 }
//             }
//         }
//     }
//
//     /// Handles asteroids by sending Some(Rocket) if the planet has a rocket or if not it sends None
//     fn handle_asteroid(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator) -> Option<Rocket> {
//
//         // Initialization of Payload for handle_asteroid logs
//         let mut payload: BTreeMap<String, String> = BTreeMap::new();
//         payload.insert("planet_id".into(), state.id().to_string().into());
//         payload.insert("system".into(), "planet_ai".into());
//         payload.insert("event".into(), "handle_asteroid".into());
//
//         self.asteroid_count += 1;
//         println!("Asteroid received, asteroid num {}", self.asteroid_count);
//
//         if state.has_rocket() {
//             let rocket = state.take_rocket();
//
//             payload.insert("action".into(), "use_rocket".into());
//             payload.insert("result".into(), "success".into());
//
//             println!("Planet {} used a rocket to defend from asteroid {}", state.id(), self.asteroid_count);
//
//             let res = state.full_cell();
//
//             match res {
//                 None => {
//                     payload.insert("event".into(), "build_new_rocket".into());
//                     payload.insert("result".into(),"failure".into());
//                     payload.insert("reason".into(), "no_energy".into());
//                     create_and_send_log(state.id() as u64, Planet, "Planet".to_string(), InternalPlanetAction, Debug, payload);
//                 }
//                 Some((c, i)) => {
//                     payload.insert("event".into(), "build_new_rocket".into());
//                     payload.insert("result".into(), "success".into());
//                     create_and_send_log(state.id() as u64, Planet, "Planet".to_string(), InternalPlanetAction, Debug, payload);
//
//                     state.build_rocket(i);
//                 }
//             }
//
//             rocket
//
//         } else {
//             payload.insert("action".into(), "build_rocket".into());
//
//             let res = state.full_cell();
//             match res {
//                 None => {
//                     payload.insert("result".into(), "failure".into());
//                     payload.insert("reason".into(), "no_energy".into());
//                     payload.insert("result".into(), "no_rocket".into());
//                     payload.insert("result".into(), "planet_destroyed".into());
//                     create_and_send_log(state.id() as u64, Planet, "Planet".to_string(), InternalPlanetAction, Debug, payload);
//
//                     self.destroyed = true;
//                     println!("planet {} destroyed", state.id());
//                     None
//                 }
//                 Some((c, i)) => {
//                     match state.build_rocket(i) {
//                         Err(e) => {
//                             payload.insert("result".into(), "failure".into());
//                             payload.insert("reason".into(), "planet_not_able_to".into());
//                             payload.insert("result".into(), "no_rocket".into());
//                             payload.insert("result".into(), "planet_destroyed".into());
//                             create_and_send_log(state.id() as u64, Planet, "Planet".to_string(), InternalPlanetAction, Debug, payload);
//
//                             self.destroyed = true;
//                             println!("planet {} destroyed", state.id());
//                             None
//                         }
//                         Ok(rocket) => {
//                             payload.insert("result".into(), "success".into());
//                             payload.insert("action".into(), "use_rocket".into());
//                             payload.insert("result".into(), "success".into());
//                             create_and_send_log(state.id() as u64, Planet, "Planet".to_string(), InternalPlanetAction, Debug, payload);
//
//                             println!("Planet {} used a rocket to defend from asteroid {}", state.id(), self.asteroid_count);
//                             state.take_rocket()
//                         }
//                     }
//
//                 }
//             }
//
//         }
//     }

// fn start(&mut self, state: &PlanetState) {
//
//     // Planet start log
//     let mut payload: BTreeMap<String, String> = BTreeMap::new();
//     payload.insert("planet_id".into(), state.id().to_string().into());
//     payload.insert("system".into(), "planet_ai".into());
//     payload.insert("event".into(), "start_planet".into());
//     payload.insert("result".into(), "success".into());
//     create_and_send_log(state.id() as u64, Planet, "Planet".to_string(), InternalPlanetAction, Debug, payload);
//
//     println!("Starting planet :) ");
//     self.stopped = false;
// }
//
// fn stop(&mut self, state: &PlanetState) {
//
//     // Planet stop log
//     let mut payload: BTreeMap<String, String> = BTreeMap::new();
//     payload.insert("planet_id".into(), state.id().to_string().into());
//     payload.insert("system".into(), "planet_ai".into());
//     payload.insert("event".into(), "stop_planet".into());
//     payload.insert("result".into(), "success".into());
//     create_and_send_log(state.id() as u64, Planet, "Planet".to_string(), InternalPlanetAction, Debug, payload);
//
//     self.destroyed = true;
// }
// }