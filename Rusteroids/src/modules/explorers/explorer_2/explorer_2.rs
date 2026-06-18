use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;
use common_game::components::resource::{BasicResource, BasicResourceType, ComplexResource, ComplexResourceType, GenericResource, ResourceType};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use rand::Rng;
use crate::modules::explorer_utils::explorer::{Explorer, ExplorerBehaviour};
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_base::ExplorerBase;
use crate::modules::explorer_utils::explorer_map::ExplorerMap;
use crate::modules::explorer_utils::handlers::AIHandlers;
use crate::modules::explorer_utils::planet_infos::PlanetInfos;
use crate::modules::explorer_utils::recipes::{get_complex_shopping_list, get_shopping_list};
use crate::modules::explorer_utils::resource_types::get_all_complex_resource_types;
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorers::common_tasks::craft_all::CraftAllTask;
use crate::modules::explorers::explorer_2::tasks::visit_all_edges::TotalEdgesVisitedTask;
use crate::modules::explorer_utils::bag_type::{BagType, DummyBag};


pub type MissingBasicResources = HashMap<BasicResourceType, usize>;
pub type MissingComplexResources = HashMap<ComplexResourceType, usize>;

//DECISION TREE:
// - 1 check if current planet can craft
// - 2 check bag
// - 3 check if I have the ingredients to craft
// - 4 if I can craft I do it, else I check if I can extract
// - 5 Move to a planet which permits to craft missing complex resources or to extract basic resources needed to craft

/// An AI-driven explorer variant focused on path/edge exploration while completing
/// full crafting checklists across different planetary systems.
pub struct Explorer2 {
    pub base: RwLock<ExplorerBase>,
    pub tot_edges_task: RwLock<TotalEdgesVisitedTask>,
    pub craft_all_task: RwLock<CraftAllTask>,
    pub dummy_bag: RwLock<DummyBag>,
    pub explorer_map: RwLock<ExplorerMap>,
    pub prev_planet: RwLock<ID>
}

impl Explorer2 {
    /// Initializes a new explorer instance with a specific targeted edge traversal quota.
    pub fn new(
        explorer_id: ID,
        current_planet_id: ID,
        from_orchestrator: Receiver<OrchestratorToExplorer>,
        to_orchestrator: Sender<ExplorerToOrchestrator<DummyBag>>,
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
            RwLock::new(HashSet::new())
        ));

        Self {
            base,
            tot_edges_task: RwLock::new(TotalEdgesVisitedTask::new(48)),
            craft_all_task: RwLock::new(CraftAllTask::new()),
            dummy_bag: RwLock::new(DummyBag::new(HashMap::new(), HashMap::new())),
            explorer_map: RwLock::new(ExplorerMap::new()),
            prev_planet: RwLock::new(current_planet_id)
        }
    }

    /// Computes the exact missing raw materials based on unfulfilled crafting requirements.
    fn get_missing_basic_resources(&self) -> MissingBasicResources {
        let mut missing_resources: MissingBasicResources = HashMap::new();
        let base_guard = self.get_base();
        let bag_guard = base_guard.bag.read().unwrap();
        let dummy_bag = bag_guard.to_dummy();

        let task_state = self.craft_all_task.read().unwrap().get_progress();

        for (resource, already_crafted) in task_state {
            if !already_crafted {
                let vec_missing = get_shopping_list(&dummy_bag, &resource);

                for missing in vec_missing {
                    let node = missing_resources.get_mut(&missing);
                    match node {
                        Some(node) => {
                            *node += 1;
                        }
                        None => {
                            missing_resources.insert(missing, 1);
                        }
                    }
                }
            }
        }

        for (resource, qty) in &mut missing_resources {
            let already_have = dummy_bag.get_basic_quantity(resource);
            if already_have <= *qty {
                *qty -= already_have;
            }
            else {
                *qty = 0;
            }
        }

        missing_resources
    }

    /// Calculates missing complex resources needed to complete the overall game task checklist.
    fn get_missing_complex_resources(&self) -> MissingComplexResources {
        let mut missing_resources: MissingComplexResources = HashMap::new();
        let base_guard = self.get_base();
        let bag_guard = base_guard.bag.read().unwrap();
        let dummy_bag = bag_guard.to_dummy();

        for resource in get_all_complex_resource_types() {
            let needed = get_complex_shopping_list(&dummy_bag, &resource);
            for needed_resource in needed {
                let node = missing_resources.get_mut(&needed_resource);
                match node {
                    Some(node) => {
                        *node += 1;
                    }
                    None => {
                        missing_resources.insert(needed_resource, 1);
                    }
                }
            }
        }

        for (resource, qty) in &mut missing_resources {
            let already_have = dummy_bag.get_complex_quantity(resource);
            if already_have <= *qty {
                *qty -= already_have;
            }
            else {
                *qty = 0;
            }
        }

        missing_resources
    }

    /// Chooses a neighboring destination based on resource matching and paths not recently traversed.
    fn change_planet(&self, missing_resources: MissingBasicResources) {
        let explorer_map_guard = self.explorer_map.read().unwrap();
        let base_guard = self.get_base();
        let neighbours = base_guard.neighbours.read().unwrap();

        if neighbours.len() == 0 {
            return;
        }

        let mut next_planet: ID = neighbours[get_random_index(neighbours.len())];

        for neighbour in &*neighbours {
            let planet_infos = explorer_map_guard.infos.get(&neighbour);
            match planet_infos {
                None => {
                    next_planet = *neighbour;
                    break;
                }
                Some(planet_infos) => {
                    let mut basic_planet_found = false;

                    for (basic_resource, qty) in &missing_resources {
                        if qty > &0 {
                            let tmp = planet_infos.basic_resources.get(basic_resource);
                            if let Some(_basic_resource) = tmp {
                                if &*self.prev_planet.read().unwrap() != neighbour {
                                    basic_planet_found = true;
                                    next_planet = neighbour.clone();
                                }
                            }
                        }
                    }

                    if !basic_planet_found {
                        // Context-dependent: complex requirements check can be integrated here
                    }
                }
            }
        }

        let mut prev_guard = self.prev_planet.write().unwrap();
        *prev_guard = base_guard.current_planet_id.read().unwrap().clone();

        base_guard.travel_request(next_planet);
    }

    /// Evaluates current available execution cells to either trigger combination procedures or gather base ingredients.
    fn decision_tree(&self) {
        let base_guard = self.get_base();
        let mut num_cells = base_guard.ask_available_cells();
        let missing_basic_resources = self.get_missing_basic_resources();
        let missing_complex_resources = self.get_missing_complex_resources();

        println!("missing_resources: {:?}", missing_basic_resources);

        if num_cells == 0 {
            self.change_planet(missing_basic_resources);
        }
        else {
            let explorer_map_guard = self.explorer_map.read().unwrap();
            let planet_infos = explorer_map_guard.infos.get(&base_guard.current_planet_id.read().unwrap()).unwrap();
            let mut resource_target: Option<BasicResourceType> = None;

            println!("COMPLEX RESOURCES CRAFTABILI NEL PIANETA {}: {:?}", base_guard.current_planet_id.read().unwrap(), planet_infos.complex_resources);

            while num_cells >= 1 {
                sleep(Duration::from_millis(1000));
                let task_state = self.craft_all_task.read().unwrap().get_progress();

                if planet_infos.complex_resources.len() > 0 {
                    for combination in &planet_infos.complex_resources {
                        println!("CONTROLLO SE HO GIà CRAFTATO {:?}", combination);
                        let qty_needed = missing_complex_resources.get(&combination).unwrap_or(&0);
                        if qty_needed > &0 {
                            num_cells = base_guard.ask_available_cells();
                            println!("DEBUG: (DI NUOVO) Celle disponibili {:?}", num_cells);
                            if num_cells > 0 {
                                println!("NON Ho già craftato {:?}", combination);
                                base_guard.combine_resource(combination.clone(), |res| self.combine_resource_handler(res))
                            }
                        }
                    }
                }

                for (resource, qty) in &missing_basic_resources {
                    if qty > &0 && planet_infos.basic_resources.contains(resource) {
                        resource_target = Some(*resource);
                        break;
                    }
                }

                if let Some(resource_target_inner) = resource_target {
                    let gen_res = base_guard.generate_resource(resource_target_inner, |result| self.generate_resource_handler(result));

                    if let Err(_) = gen_res {
                        self.change_planet(missing_basic_resources.clone());
                        break;
                    }

                    resource_target = None;
                    num_cells = base_guard.ask_available_cells();
                } else {
                    break;
                }
            }
            self.change_planet(missing_basic_resources);
        }
    }
}

impl AIHandlers for Explorer2 {
    fn start_ai_handler(&self) {}

    fn reset_ai_handler(&self) {}

    fn kill_handler(&self) {}

    fn generate_resource_handler(&self, result: &Option<&BasicResource>) {}

    fn combine_resource_handler(&self, result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>) {
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
        if !self.explorer_map.read().unwrap().is_edge_visited(&self.prev_planet.read().unwrap(), &self.get_base().current_planet_id.read().unwrap()) {
            self.tot_edges_task.write().unwrap().update_progress();
            println!("Aggiungo arco visitato {} - {}", self.prev_planet.read().unwrap().clone(), self.get_base().current_planet_id.read().unwrap().clone());
            self.explorer_map.write().unwrap().visit_edge(self.prev_planet.read().unwrap().clone(), self.get_base().current_planet_id.read().unwrap().clone());
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

            if ! *alive {
                println!("Explorer #{} è morto come un cogl", base_guard.explorer_id);
                return;
            }

            let mut explorer_map_guard = self.explorer_map.write().unwrap();
            let current_planet_id = base_guard.current_planet_id.read().unwrap();

            if !explorer_map_guard.is_planet_discovered(&current_planet_id) {
                base_guard.ask_for_neighbours();
                base_guard.ask_supported_resources();
                base_guard.ask_combinations();

                let planet_infos = PlanetInfos::new(base_guard.basic_resources.read().unwrap().clone(), base_guard.combinations.read().unwrap().clone());
                explorer_map_guard.planet_discovery(*current_planet_id, planet_infos, base_guard.neighbours.read().unwrap().clone());
            }
            else {
                base_guard.ask_for_neighbours();
            }

            explorer_map_guard.update_neighbors(&current_planet_id, &base_guard.neighbours.read().unwrap());

            drop(explorer_map_guard);
            drop(current_planet_id);

            let task_guard = self.craft_all_task.read().unwrap();
            let task_state = task_guard.get_state();
            if let TaskState::Finished = task_state {
                println!("Craftato tutto CAPO");
                let task2_guard = self.tot_edges_task .read().unwrap();
                let mut task2_state = task2_guard.get_state().clone();
                drop(task2_guard);

                if let TaskState::Finished = task2_state {
                    println!("LAVORO FINITO CAPO");
                    return;
                }
                else {
                    println!("VADO A ESPLORARE LA GALASSIA");

                    loop {
                        match task2_state {
                            TaskState::Finished => {
                                println!("LAVORO FINITO CAPO");
                                return;
                            }
                            TaskState::Uncompletable => {
                                println!("NON POSSO PIù Esplorare, uccidetemi");
                                return;
                            }
                            TaskState::Pending => {
                                let base_guard = self.get_base();
                                sleep(Duration::from_millis(1000));

                                let neighbours = base_guard.neighbours.read().unwrap();
                                if neighbours.len() == 0 {
                                    println!("NUN POSSO PIU ESPLORARE LA GALASSIA MANNAC");
                                    let mut task2_guard = self.tot_edges_task.write().unwrap();
                                    task2_guard.update_state(TaskState::Uncompletable);
                                } else {
                                    println!("STO ESPLORANDO LA GALASSIA e ho visitato {} Archi", self.explorer_map.read().unwrap().get_num_discovered_edges());

                                    let mut next_planet = neighbours[get_random_index(neighbours.len())];
                                    let current_planet = base_guard.current_planet_id.read().unwrap();

                                    // Dynamic routing strategy: Scan connected routes to prioritize unvisited edges first
                                    for neig in neighbours.iter() {
                                        let explorer_map_guard = self.explorer_map.read().unwrap();
                                        if ! explorer_map_guard.is_edge_visited(&current_planet, neig) {
                                            next_planet = neig.clone();
                                            println!("TROVATO ARCO NON VISITATO, DA {} A {}", current_planet, next_planet);
                                            break;
                                        }
                                    }

                                    drop(neighbours);
                                    drop(current_planet);

                                    *self.prev_planet.write().unwrap() = self.get_base().current_planet_id.read().unwrap().clone();
                                    self.get_base().travel_request(next_planet);
                                }

                                let task2_guard = self.tot_edges_task.read().unwrap();
                                task2_state = task2_guard.get_state().clone();
                                drop(task2_guard);
                            }
                        }
                    }
                }
            }

            drop(task_guard);

            self.decision_tree();
        }
    }

    /// Evaluates if both item-crafting checklists and topological road-mapping objectives are finished.
    fn all_tasks_finished(&self) -> bool {
        let craft_all_state = self.craft_all_task.read().unwrap().get_state().clone();
        let num_edges_state = self.tot_edges_task.read().unwrap().get_state().clone();
        match (craft_all_state, num_edges_state) {
            (TaskState::Finished, TaskState::Finished) => true,
            _ => false
        }
    }
}

fn get_random_index(length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    let mut rng = rand::rng();
    rng.random_range(0..length)
}