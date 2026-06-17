use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};
use common_game::components::resource::{BasicResource, ComplexResourceRequest};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use crossbeam_channel::select_biased;
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_base::ExplorerBase;
use crate::modules::explorer_utils::handlers::AIHandlers;
use crate::modules::manual_explorer::bag_type::DummyBag;

const ERROR_ORCH_DISCONNECTED: &'static str = "Orchestrator disconnected from explorer";

pub trait Explorer {
    
    fn get_base (&self) -> RwLockReadGuard<ExplorerBase>;
    fn get_base_mut(&self) -> RwLockWriteGuard<ExplorerBase>;
    
    fn get_dummy_bag_mut(&self) -> RwLockWriteGuard<DummyBag>;
    fn get_dummy_bag(&self) -> RwLockReadGuard<DummyBag>;
    
    fn run(&self, container: Arc<dyn AIHandlers> ) -> Result<(), String> {
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
            println!("Explorer {} entered main loop", self.get_base().explorer_id);
            // Se è morto, si esce dal loop
            if !*self.get_base().alive.read().unwrap() {
                println!("Explorer {} detected death, exiting...", self.get_base().explorer_id);
                return Ok(());
            }

            match self.get_base().from_orchestrator.recv() {
                Ok(OrchestratorToExplorer::StartExplorerAI) => {
                    // self.start_ai();
                }
                Ok(OrchestratorToExplorer::ResetExplorerAI) => {
                    self.get_base().reset_ai(|| container.reset_ai_handler());
                }
                Ok(OrchestratorToExplorer::KillExplorer) => {
                    self.get_base().kill(|| container.kill_handler());
                    return Ok(());
                }

                Ok(OrchestratorToExplorer::MoveToPlanet {
                       sender_to_new_planet,
                       planet_id,
                   }) => {
                    println!("Received move to planet msg");
                    self.get_base().move_to_planet(sender_to_new_planet, planet_id, || container.move_to_planet_handler());
                }
                Ok(OrchestratorToExplorer::CurrentPlanetRequest) => {
                    self.get_base().get_current_planet();
                }
                Ok(OrchestratorToExplorer::SupportedResourceRequest) => {
                    self.get_base().ask_supported_resources();
                }
                Ok(OrchestratorToExplorer::SupportedCombinationRequest) => {
                    self.get_base().ask_combinations();
                }
                Ok(OrchestratorToExplorer::GenerateResourceRequest { to_generate }) => {
                    self.get_base().generate_resource(to_generate, |arg: &Option<&BasicResource> | container.generate_resource_handler(arg));
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
                    self.get_base().get_bag();
                }
                Ok(OrchestratorToExplorer::NeighborsResponse { neighbors }) => {
                    println!("Received neighbors response: {:?}", neighbors);
                    self.get_base().set_neighbours(neighbors);
                }
                Err(_) => {
                    return Err(ERROR_ORCH_DISCONNECTED.to_string());
                }
                Ok(OrchestratorToExplorer::StopExplorerAI) => todo!(),
            }
        }
    }

    fn wait_for_start(&self) -> Result<bool, String> {
        loop {
            select_biased! {
                // orch messages
                recv(self.get_base().from_orchestrator) -> msg => match msg {
                    // if `Start` is received, return false
                    Ok(OrchestratorToExplorer::StartExplorerAI) => {
                        self.get_base().to_orchestrator
                            .send(ExplorerToOrchestrator::StartExplorerAIResult {
                                explorer_id: self.get_base().explorer_id,
                            })
                            .map_err(|_| ERROR_ORCH_DISCONNECTED.to_string())?;

                        return Ok(false);
                    }

                    Ok(OrchestratorToExplorer::KillExplorer) => {
                        let base_lock = self.get_base();
                        let mut alive_lock = base_lock.alive.write().unwrap();
                        *alive_lock = false; // aggiunto per visaulizer
                        self.get_base().to_orchestrator
                            .send(ExplorerToOrchestrator::KillExplorerResult { explorer_id: self.get_base().explorer_id })
                            .map_err(|_| ERROR_ORCH_DISCONNECTED.to_string())?;

                        return Ok(true)
                    }
                    Ok(_) => {
                        self.get_base().to_orchestrator
                            .send(ExplorerToOrchestrator::ResetExplorerAIResult {explorer_id: self.get_base().explorer_id})
                            .map_err(|_| ERROR_ORCH_DISCONNECTED.to_string())?
                    }

                    Err(_) => return Err(ERROR_ORCH_DISCONNECTED.to_string()),
                },

            }
        }
    }
    
    fn handle_explorer(&self);

    fn all_tasks_finished(&self) -> bool;
}

pub trait ExplorerBehaviour: Explorer + AIHandlers + Send + Sync{}