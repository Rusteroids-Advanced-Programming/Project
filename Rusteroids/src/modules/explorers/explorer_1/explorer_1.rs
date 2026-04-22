use std::collections::{HashMap, HashSet};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;
use common_game::components::resource::{BasicResource, BasicResourceType, ComplexResource, ComplexResourceType, GenericResource};
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
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorers::explorer_1::tasks::visit_all_planet::TotalPlanetsVisitedTask;
use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};

pub struct Explorer1 {
    pub base: RwLock<ExplorerBase>,
    pub tot_visits_task: RwLock<TotalPlanetsVisitedTask>,
    pub dummy_bag: RwLock<DummyBag>,
    pub explorer_map: RwLock<ExplorerMap>,
}

impl Explorer1 {
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
            tot_visits_task: RwLock::new(TotalPlanetsVisitedTask::new(500)),
            dummy_bag: RwLock::new(DummyBag::new(HashMap::new(), HashMap::new())),
            explorer_map: RwLock::new(ExplorerMap::new())
        }
    }
}

impl AIHandlers for Explorer1 {
    fn start_ai_handler(&self) {}

    fn reset_ai_handler(&self) {}

    fn kill_handler(&self) {}

    fn generate_resource_handler(&self, result: &Option<&BasicResource>) {}

    fn combine_resource_handler(&self, result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>) {}

    fn move_to_planet_handler(&self) {
        self.tot_visits_task.write().unwrap().update_progress()
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


            println!("neighbours of explorer #{}: {:?}", base_guard.explorer_id, base_guard.neighbours.read().unwrap());
            
            println!("EXPLORER #{} MAP = {:?}", self.get_base().explorer_id, explorer_map_guard);
            // let planet_ids = base_guard.neighbours.read().unwrap();
            let current_node = explorer_map_guard.graph.get_node(&current_planet_id).unwrap();
            let current_node_guard = current_node.read().unwrap();
            let planet_ids = &current_node_guard.adjacent_nodes;
            
            if planet_ids.len() > 0 {
                match self.tot_visits_task.read().unwrap().get_state() {
                    TaskState::Finished => {
                        println!("LAVORO FINITO CAPO");
                        break;
                    }
                    TaskState::Pending => {
                        let rand_index = get_random_index(planet_ids.len());
                        let target_planet = &planet_ids[rand_index];

                        println!("Explorer #{} is starting to think", self.get_base().explorer_id);
                        self.get_base().to_orchestrator.send(ExplorerToOrchestrator::TravelToPlanetRequest {
                            explorer_id: self.get_base().explorer_id,
                            current_planet_id: current_planet_id.clone(),
                            dst_planet_id: target_planet.read().unwrap().value,
                        }).unwrap();
                    }
                    _ => {
                        println!("Task uncompletable for explorer #{}", self.get_base().explorer_id);
                    }
                }
            }

            else {
                println!("Explorer #{} non ha vicini in cui spostarsi", self.get_base().explorer_id);
            }
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
