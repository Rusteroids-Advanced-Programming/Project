use crate::modules::explorer_utils::bag_type::{BagType, DummyBag};
use crate::modules::explorer_utils::explorer::{Explorer, ExplorerBehaviour};
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_base::ExplorerBase;
use crate::modules::explorer_utils::explorer_map::ExplorerMap;
use crate::modules::explorer_utils::get_random_index;
use crate::modules::explorer_utils::handlers::AIHandlers;
use crate::modules::explorer_utils::planet_infos::PlanetInfos;
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorers::common_tasks::craft_all::CraftAllTask;
use crate::modules::explorers::explorer_2::tasks::visit_all_edges::TotalEdgesVisitedTask;
use common_game::components::resource::{BasicResource, ComplexResource, GenericResource};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;

/// An AI-driven explorer variant focused on path/edge exploration while completing
/// full crafting checklists across different planetary systems.
pub struct Explorer2 {
    pub base: RwLock<ExplorerBase>,
    pub tot_edges_task: RwLock<TotalEdgesVisitedTask>,
    pub craft_all_task: Arc<RwLock<CraftAllTask>>,
    pub dummy_bag: RwLock<DummyBag>,
    pub explorer_map: Arc<RwLock<ExplorerMap>>,
    pub prev_planet: Arc<RwLock<ID>>,
}

impl Explorer2 {
    /// Initializes a new explorer instance with a specific targeted edge traversal quota.
    pub fn new(
        explorer_id: ID,
        current_planet_id: ID,
        from_orchestrator: Receiver<OrchestratorToExplorer>,
        to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
        num_total_planets: usize,
    ) -> Self {
        let base = RwLock::new(ExplorerBase::new(
            explorer_id,
            RwLock::new(BagType::new()),
            RwLock::new(current_planet_id),
            to_orchestrator,
            from_orchestrator,
            RwLock::new(None),
            RwLock::new(None),
            RwLock::new(Vec::new()),
            RwLock::new(HashSet::new()),
            RwLock::new(HashSet::new()),
        ));

        let num_edges_to_visit = num_total_planets / 100 * 60;

        Self {
            base,
            tot_edges_task: RwLock::new(TotalEdgesVisitedTask::new(num_edges_to_visit)),
            craft_all_task: Arc::new(RwLock::new(CraftAllTask::new())),
            dummy_bag: RwLock::new(DummyBag::new(HashMap::new(), HashMap::new())),
            explorer_map: Arc::new(RwLock::new(ExplorerMap::new())),
            prev_planet: Arc::new(RwLock::new(current_planet_id)),
        }
    }
}

impl AIHandlers for Explorer2 {
    fn start_ai_handler(&self) {}

    fn reset_ai_handler(&self) {}

    fn kill_handler(&self) {}

    fn generate_resource_handler(&self, _result: &Option<&BasicResource>) {}

    fn combine_resource_handler(
        &self,
        result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>,
    ) {
        match result {
            Ok(resource) => {
                let mut task_guard = self.craft_all_task.write().unwrap();
                task_guard.update_progress(resource.get_type());
            }
            Err(_) => {}
        }
    }

    /// Handles map tracking updates when the explorer successfully switches location, evaluating if a new edge connection was traversed.
    fn move_to_planet_handler(&self) {
        // Evaluate if the path link between previous and current node has already been explored
        if !self.explorer_map.read().unwrap().is_edge_visited(
            &self.prev_planet.read().unwrap(),
            &self.get_base().current_planet_id.read().unwrap(),
        ) {
            self.tot_edges_task.write().unwrap().update_progress();
            self.explorer_map.write().unwrap().visit_edge(
                self.prev_planet.read().unwrap().clone(),
                self.get_base().current_planet_id.read().unwrap().clone(),
            );
        }
    }
}

impl ExplorerBehaviour for Explorer2 {}

impl Explorer for Explorer2 {
    fn get_base(&self) -> RwLockReadGuard<ExplorerBase> {
        self.base.read().unwrap()
    }

    fn get_base_mut(&self) -> RwLockWriteGuard<ExplorerBase> {
        self.base.write().unwrap()
    }

    fn get_dummy_bag_mut(&self) -> RwLockWriteGuard<DummyBag> {
        self.dummy_bag.write().unwrap()
    }

    fn get_dummy_bag(&self) -> RwLockReadGuard<DummyBag> {
        self.dummy_bag.read().unwrap()
    }

    /// Execution framework processing discovery tracking, environment scanning, and edge-prioritized traveling loops.
    fn handle_explorer(&self) {
        loop {
            sleep(Duration::from_millis(1000));

            let base_guard = self.get_base();
            let alive = base_guard.alive.read().unwrap();

            if !*alive {
                return;
            }

            let mut explorer_map_guard = self.explorer_map.write().unwrap();
            let current_planet_id = base_guard.current_planet_id.read().unwrap();

            if !explorer_map_guard.is_planet_discovered(&current_planet_id) {
                base_guard.ask_for_neighbours();
                base_guard.ask_supported_resources();
                base_guard.ask_combinations();

                let planet_infos = PlanetInfos::new(
                    base_guard.basic_resources.read().unwrap().clone(),
                    base_guard.combinations.read().unwrap().clone(),
                );
                explorer_map_guard.planet_discovery(
                    *current_planet_id,
                    planet_infos,
                    base_guard.neighbours.read().unwrap().clone(),
                );
            } else {
                base_guard.ask_for_neighbours();
            }

            explorer_map_guard
                .update_neighbors(&current_planet_id, &base_guard.neighbours.read().unwrap());

            drop(explorer_map_guard);
            drop(current_planet_id);

            let task_guard = self.craft_all_task.read().unwrap();
            let task_state = task_guard.get_state();
            if let TaskState::Finished = task_state {
                let task2_guard = self.tot_edges_task.read().unwrap();
                let mut task2_state = task2_guard.get_state().clone();
                drop(task2_guard);

                if let TaskState::Finished = task2_state {
                    return;
                }
                loop {
                    match task2_state {
                        TaskState::Finished => {
                            return;
                        }
                        TaskState::Uncompletable => {
                            return;
                        }
                        TaskState::Pending => {
                            let base_guard = self.get_base();
                            sleep(Duration::from_millis(1000));

                            let neighbours = base_guard.neighbours.read().unwrap();
                            if neighbours.len() == 0 {
                                let mut edges_task_guard = self.tot_edges_task.write().unwrap();
                                edges_task_guard.update_state(TaskState::Uncompletable);
                            } else {
                                let mut next_planet =
                                    neighbours[get_random_index(neighbours.len())];
                                let current_planet = base_guard.current_planet_id.read().unwrap();

                                // Dynamic routing strategy: Scan connected routes to prioritize unvisited edges first
                                for neig in neighbours.iter() {
                                    let explorer_map_guard = self.explorer_map.read().unwrap();
                                    if !explorer_map_guard.is_edge_visited(&current_planet, neig) {
                                        next_planet = neig.clone();
                                        break;
                                    }
                                }

                                drop(neighbours);
                                drop(current_planet);

                                *self.prev_planet.write().unwrap() =
                                    self.get_base().current_planet_id.read().unwrap().clone();
                                self.get_base().travel_request(next_planet);
                            }

                            let task2_guard = self.tot_edges_task.read().unwrap();
                            task2_state = task2_guard.get_state().clone();
                            drop(task2_guard);
                        }
                    }
                }
            }

            drop(task_guard);

            CraftAllTask::resolve_task(
                self,
                self.craft_all_task.clone(),
                self.explorer_map.clone(),
                self.prev_planet.clone(),
            );
        }
    }

    /// Evaluates if both item-crafting checklists and topological road-mapping objectives are finished.
    fn all_tasks_finished(&self) -> bool {
        let craft_all_state = self.craft_all_task.read().unwrap().get_state().clone();
        let num_edges_state = self.tot_edges_task.read().unwrap().get_state().clone();
        match (craft_all_state, num_edges_state) {
            (TaskState::Finished, TaskState::Finished) => true,
            _ => false,
        }
    }
}
