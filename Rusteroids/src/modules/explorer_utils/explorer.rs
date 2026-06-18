//! Defines the [`Explorer`] trait: the core run-loop and message-handling
//! contract shared by all explorer implementations.
//!
//! An explorer is driven entirely by messages coming from the orchestrator
//! (see [`OrchestratorToExplorer`]). It starts paused, waits for an explicit
//! start/kill signal, then enters a loop that dispatches each incoming
//! message to the matching handler on [`ExplorerBase`].

use crate::modules::explorer_utils::bag_type::DummyBag;
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_base::ExplorerBase;
use crate::modules::explorer_utils::handlers::AIHandlers;
use common_game::components::resource::BasicResource;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use crossbeam_channel::select_biased;
use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};

const ERROR_ORCH_DISCONNECTED: &str = "Orchestrator disconnected from explorer";

/// Shared behaviour for every explorer type.
///
/// Implementors must provide read/write access to their [`ExplorerBase`]
/// (shared state and orchestrator channels) and to their [`DummyBag`]
/// (resource inventory). The trait then supplies the default run-loop
/// (`run`) and startup gate (`wait_for_start`) that drive the explorer.
pub trait Explorer {
    fn get_base(&self) -> RwLockReadGuard<ExplorerBase>;
    fn get_base_mut(&self) -> RwLockWriteGuard<ExplorerBase>;

    fn get_dummy_bag_mut(&self) -> RwLockWriteGuard<DummyBag>;
    fn get_dummy_bag(&self) -> RwLockReadGuard<DummyBag>;

    /// Main entry point: blocks waiting for a start signal, then runs the
    /// explorer's message loop until it is killed or the orchestrator
    /// disconnects. `container` supplies the AI-specific callbacks invoked
    /// by each [`ExplorerBase`] handler (e.g. `kill_handler`).
    fn run(&self, container: Arc<dyn AIHandlers>) -> Result<(), String> {
        let kill = self.wait_for_start()?;
        if kill {
            return Ok(());
        }

        loop {
            if !*self.get_base().alive.read().unwrap() {
                return Ok(());
            }

            match self.get_base().from_orchestrator.recv() {
                Ok(OrchestratorToExplorer::StartExplorerAI) => {}
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
                    self.get_base()
                        .move_to_planet(sender_to_new_planet, planet_id, || {
                            container.move_to_planet_handler()
                        });
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
                    let _res = self
                        .get_base()
                        .generate_resource(to_generate, |arg: &Option<&BasicResource>| {
                            container.generate_resource_handler(arg)
                        });
                }
                Ok(OrchestratorToExplorer::CombineResourceRequest {
                    to_generate: _to_generate,
                }) => {}
                Ok(OrchestratorToExplorer::BagContentRequest) => {
                    self.get_base().get_bag();
                }
                Ok(OrchestratorToExplorer::NeighborsResponse { neighbors }) => {
                    self.get_base().set_neighbours(neighbors);
                }
                Err(_) => {
                    return Err(ERROR_ORCH_DISCONNECTED.to_string());
                }
                Ok(OrchestratorToExplorer::StopExplorerAI) => {}
            }
        }
    }

    /// Blocks until the orchestrator sends either `StartExplorerAI` or
    /// `KillExplorer`. Any other message received while waiting is treated
    /// as an implicit reset (acknowledged, then waiting continues).
    /// Returns `Ok(true)` if killed before starting, `Ok(false)` once
    /// started, or `Err` if the orchestrator channel disconnects.
    fn wait_for_start(&self) -> Result<bool, String> {
        loop {
            select_biased! {
                recv(self.get_base().from_orchestrator) -> msg => match msg {
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
                        *alive_lock = false; // marks explorer as dead before acking, so the visualizer reflects it immediately
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

/// Marker trait combining everything a concrete explorer needs to be
/// usable by the orchestrator: the message loop ([`Explorer`]), the AI
/// callbacks ([`AIHandlers`]), and thread-safety bounds.
pub trait ExplorerBehaviour: Explorer + AIHandlers + Send + Sync {}
