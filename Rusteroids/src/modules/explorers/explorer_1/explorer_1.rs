use crate::modules::explorer_utils::bag_type::{BagType, DummyBag};
use crate::modules::explorer_utils::explorer::{Explorer, ExplorerBehaviour};
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_base::ExplorerBase;
use crate::modules::explorer_utils::explorer_map::ExplorerMap;
use crate::modules::explorer_utils::handlers::AIHandlers;
use crate::modules::explorer_utils::planet_infos::PlanetInfos;
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorers::common_tasks::craft_all::CraftAllTask;
use crate::modules::explorers::explorer_1::tasks::visit_all_planet::TotalPlanetsVisitedTask;
use common_game::components::resource::{BasicResource, ComplexResource, GenericResource};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;

/// An AI-driven explorer variant focused on visiting as many distinct planets as possible
/// while completing full crafting checklists.
pub struct Explorer1 {
    pub base: RwLock<ExplorerBase>,
    pub tot_planets_task: RwLock<TotalPlanetsVisitedTask>,
    pub craft_all_task: Arc<RwLock<CraftAllTask>>,
    pub dummy_bag: RwLock<DummyBag>,
    pub explorer_map: Arc<RwLock<ExplorerMap>>,
    // Kept only because the shared resolve_task signature threads it into change_planet;
    // the planet-counting logic never reads it.
    pub prev_planet: Arc<RwLock<ID>>,
}

impl Explorer1 {
    /// Initializes a new explorer instance with a specific target of distinct planets to visit.
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

        let num_planets_to_visit = num_total_planets / 100 * 80;

        Self {
            base,
            tot_planets_task: RwLock::new(TotalPlanetsVisitedTask::new(
                num_planets_to_visit,
                current_planet_id,
            )),
            craft_all_task: Arc::new(RwLock::new(CraftAllTask::new())),
            dummy_bag: RwLock::new(DummyBag::new(HashMap::new(), HashMap::new())),
            explorer_map: Arc::new(RwLock::new(ExplorerMap::new())),
            prev_planet: Arc::new(RwLock::new(current_planet_id)),
        }
    }
}

impl AIHandlers for Explorer1 {
    fn start_ai_handler(&self) {}

    fn reset_ai_handler(&self) {}

    fn kill_handler(&self) {}

    fn generate_resource_handler(&self, _result: &Option<&BasicResource>) {}

    fn combine_resource_handler(
        &self,
        result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>,
    ) {
        // Crafting a complex resource advances the crafting checklist
        if let Ok(resource) = result {
            self.craft_all_task
                .write()
                .unwrap()
                .update_progress(resource.get_type());
        }
    }

    /// Records the newly reached planet so the distinct-planet task can advance.
    fn move_to_planet_handler(&self) {
        let planet_id = *self.get_base().current_planet_id.read().unwrap();
        self.tot_planets_task
            .write()
            .unwrap()
            .update_progress(planet_id);
    }
}

impl ExplorerBehaviour for Explorer1 {}

impl Explorer for Explorer1 {
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

    /// Main loop: keeps the local map up to date, lets the shared resolver drive movement
    /// during crafting, then deliberately routes toward unvisited planets until the target is met.
    fn handle_explorer(&self) {
        loop {
            sleep(Duration::from_millis(1000));

            let base_guard = self.get_base();
            if !*base_guard.alive.read().unwrap() {
                return;
            }

            // --- Map maintenance: discover the current planet (once) and refresh adjacency ---
            let current_planet: ID = {
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
                *current_planet_id
            }; // explorer_map_guard + current_planet_id released here

            let craft_finished = matches!(
                self.craft_all_task.read().unwrap().get_state(),
                TaskState::Finished
            );
            drop(base_guard);

            // --- Phase 1: crafting not done yet -> shared resolver drives movement ---
            if !craft_finished {
                CraftAllTask::resolve_task(
                    self,
                    self.craft_all_task.clone(),
                    self.explorer_map.clone(),
                    self.prev_planet.clone(),
                );
                continue;
            }

            // --- Phase 2: crafting done -> deliberately chase unvisited planets ---
            match self.tot_planets_task.read().unwrap().get_state() {
                TaskState::Finished => return, // both tasks done
                TaskState::Uncompletable => return,
                TaskState::Pending => {
                    let next = {
                        let map = self.explorer_map.read().unwrap();
                        let task = self.tot_planets_task.read().unwrap();
                        next_hop_to_unsatisfied(&map, current_planet, &task)
                    };

                    match next {
                        Some(next_planet) => {
                            // Travel acknowledgement fires move_to_planet_handler -> task progresses
                            self.get_base().travel_request(next_planet);
                        }
                        None => {
                            // No reachable unvisited planet remains = give up
                            self.tot_planets_task
                                .write()
                                .unwrap()
                                .update_state(TaskState::Uncompletable);
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Both objectives complete only when crafting and the distinct-planet target are finished.
    fn all_tasks_finished(&self) -> bool {
        let craft_state = self.craft_all_task.read().unwrap().get_state().clone();
        let planets_state = self.tot_planets_task.read().unwrap().get_state().clone();
        matches!(
            (craft_state, planets_state),
            (TaskState::Finished, TaskState::Finished)
        )
    }
}

/// Returns the next planet to travel to: the first hop on the shortest path
/// (through the *known* graph) from `current` to the nearest unvisited planet.
///
/// Returns None when every planet reachable through already-visited territory is
/// itself visited — i.e. the explorer is boxed in and the task can't progress.
fn next_hop_to_unsatisfied(
    explorer_map: &ExplorerMap,
    current: ID,
    task: &TotalPlanetsVisitedTask,
) -> Option<ID> {
    let mut parent: HashMap<ID, ID> = HashMap::new();
    let mut seen: HashSet<ID> = HashSet::new();
    let mut queue: VecDeque<ID> = VecDeque::new();

    seen.insert(current);
    queue.push_back(current);

    let mut target: Option<ID> = None;

    while let Some(node_id) = queue.pop_front() {
        // First planet BFS reaches that still needs more visits
        if node_id != current && !task.is_satisfied(&node_id) {
            target = Some(node_id);
            break;
        }

        if let Some(node) = explorer_map.graph.get_node(&node_id) {
            let node_guard = node.read().unwrap();
            for adj in &node_guard.adjacent_nodes {
                let adj_id = adj.read().unwrap().value;
                if seen.insert(adj_id) {
                    parent.insert(adj_id, node_id);
                    queue.push_back(adj_id);
                }
            }
        }
    }

    let mut step = target?;
    loop {
        let p = *parent.get(&step)?;
        if p == current {
            return Some(step);
        }
        step = p;
    }
}
