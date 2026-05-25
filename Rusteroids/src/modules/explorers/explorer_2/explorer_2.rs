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
use crate::modules::manual_explorer::bag_type::{BagType, DummyBag};


pub type MissingBasicResources = HashMap<BasicResourceType, usize>;
pub type MissingComplexResources = HashMap<ComplexResourceType, usize>;

//STEP DECISIONE EXPLORER
// - 1 check se il pianeta corrente può craftare
// - 2 check bag
// - 3 check se ho la recipe per craftare e in base a quello che manca nella task
// - 4 Se posso craftare crafto, se no check se posso estrarre, se puo estrae
// - 5 scelgo di muovermi sul pianeta che permette o il crafting o l'estrazione di risorse che servono a completare la task


pub struct Explorer2 {
    pub base: RwLock<ExplorerBase>,
    pub tot_edges_task: RwLock<TotalEdgesVisitedTask>,
    pub craft_all_task: RwLock<CraftAllTask>,
    pub dummy_bag: RwLock<DummyBag>,
    pub explorer_map: RwLock<ExplorerMap>,
    pub prev_planet: RwLock<ID>
}

impl Explorer2 {
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

    fn get_missing_basic_resources(&self) -> MissingBasicResources {
        let mut missing_resources: MissingBasicResources = HashMap::new();
        let base_guard = self.get_base();
        let bag_guard = base_guard.bag.read().unwrap();
        let dummy_bag = bag_guard.to_dummy();

        let task_state = self.craft_all_task.read().unwrap().get_progress();

        for (resource, already_crafted) in task_state {
            // println!("DEBUG: Calculating resources needed to craft {:?}", resource);

            if !already_crafted {
                let vec_missing = get_shopping_list(&dummy_bag, &resource);
                // println!("DEBUG: resources needed to craft {:?} SHOPPING LIST: {:?}", resource, vec_missing);

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


    fn change_planet(&self, missing_resources: MissingBasicResources) {
        // println!("DEBUG: Change planet");

        let explorer_map_guard = self.explorer_map.read().unwrap();
        let base_guard = self.get_base();
        let neighbours = base_guard.neighbours.read().unwrap();

        if neighbours.len() == 0 {
            return;
        }

        let mut next_planet: ID = neighbours[get_random_index(neighbours.len())];

        //check vicini inesplorati
        for neighbour in &*neighbours {
            let planet_infos = explorer_map_guard.infos.get(&neighbour);
            match planet_infos {
                None => {
                    // println!("DEBUG: No planet infos for {:?}", neighbour);
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
                        //implementare check degli ingredienti e delle complex che mancano
                    }
                }
            }
        }

        let mut prev_guard = self.prev_planet.write().unwrap();
        *prev_guard = base_guard.current_planet_id.read().unwrap().clone();

        base_guard.travel_request(next_planet);
        // println!("DEBUG: Changing planet to {}", next_planet);
    }


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

            // let combinations = base_guard.combinations.read().unwrap();

            //println!("DECISION_TREE: CI SONO {:?} CELLE DI ENERGIA", num_cells);
            //println!("BASIC RESOURCES GENERABILI NEL PAINETA {}: {:?}", base_guard.current_planet_id.read().unwrap(), planet_infos.basic_resources);
            println!("COMPLEX RESOURCES CRAFTABILI NEL PIANETA {}: {:?}", base_guard.current_planet_id.read().unwrap(), planet_infos.complex_resources);

            while num_cells >= 1 {
                //println!("DEBUG: Celle disponibili su pianeta {:?}: {:?}", base_guard.current_planet_id.read().unwrap(), num_cells);

                sleep(Duration::from_millis(1000));
                let task_state = self.craft_all_task.read().unwrap().get_progress();

                //println!("LEN COMPLEX RESOURCES: {}", planet_infos.complex_resources.len());
                if planet_infos.complex_resources.len() > 0 {
                    for combination in &planet_infos.complex_resources {
                        println!("CONTROLLO SE HO GIà CRAFTATO {:?}", combination);
                        let qty_needed = missing_complex_resources.get(&combination).unwrap_or(&0);
                        if qty_needed > &0 {
                            //if !task_state.get(combination).unwrap() {
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
                        //println!("VOGLIO ESTRARRE {:?}", resource_target);
                        break;
                    }
                }

                if let Some(resource_target_inner) = resource_target {
                    //println!("STO PER GENERARE {:?}", resource_target_inner);
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


    // let bag_guard = base_guard.bag.read().unwrap();
    // let combinations_guard = base_guard.combinations.read().unwrap();
    // if combinations_guard.is_empty() {
    // }
    // }
}

impl AIHandlers for Explorer2 {
    fn start_ai_handler(&self) {}

    fn reset_ai_handler(&self) {}

    fn kill_handler(&self) {}

    fn generate_resource_handler(&self, result: &Option<&BasicResource>) {
        // println!("DEBUG: Generated resource {:?}", result);
    }

    fn combine_resource_handler(&self, result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>) {
        match result {
            Ok(resource) => {
                let mut task_guard = self.craft_all_task.write().unwrap();
                task_guard.update_progress(resource.get_type());
            }
            Err(_) => {}
        }
    }

    fn move_to_planet_handler(&self) {
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
                                // let map_guard = self.explorer_map.read().unwrap();

                                let neighbours = base_guard.neighbours.read().unwrap();
                                if neighbours.len() == 0 {
                                    println!("NUN POSSO PIU ESPLORARE LA GALASSIA MANNAC");
                                    let mut task2_guard = self.tot_edges_task.write().unwrap();
                                    task2_guard.update_state(TaskState::Uncompletable);
                                } else {
                                    println!("STO ESPLORANDO LA GALASSIA e ho visitato {} Archi", self.explorer_map.read().unwrap().get_num_discovered_edges());

                                    let mut next_planet = neighbours[get_random_index(neighbours.len())];
                                    let current_planet = base_guard.current_planet_id.read().unwrap();

                                    for neig in neighbours.iter() {
                                        let explorer_map_guard = self.explorer_map.read().unwrap();
                                        if ! explorer_map_guard.is_edge_visited(&current_planet, neig) {
                                            next_planet = neig.clone();
                                            println!("TROVATO ARCO NON VISITATO, DA {} A {}", current_planet, next_planet);
                                            break;
                                        }
                                    }

                                    drop(neighbours);

                                    //println!("VADO SU PIANETA : {:?} e intanto sono su Pianeta #{}", next_planet, current_planet);
                                    drop(current_planet);
                                    // drop(base_guard);
                                    *self.prev_planet.write().unwrap() = self.get_base().current_planet_id.read().unwrap().clone();
                                    self.get_base().travel_request(next_planet);
                                    // base_guard.travel_request(next_planet);
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

            // println!("neighbours of explorer #{}: {:?}", base_guard.explorer_id, base_guard.neighbours.read().unwrap());
            //
            // println!("EXPLORER #{} MAP = {:?}", self.get_base().explorer_id, explorer_map_guard);

            // let planet_ids = base_guard.neighbours.read().unwrap();


            //TASK VISITE TOTALI

            // let current_node = explorer_map_guard.graph.get_node(&current_planet_id).unwrap();
            // let current_node_guard = current_node.read().unwrap();
            // let planet_ids = &current_node_guard.adjacent_nodes;
            //
            //
            //
            // if planet_ids.len() > 0 {
            //     match self.tot_visits_task.read().unwrap().get_state() {
            //         TaskState::Finished => {
            //             println!("LAVORO FINITO CAPO");
            //             break;
            //         }
            //         TaskState::Pending => {
            //             let rand_index = get_random_index(planet_ids.len());
            //             let target_planet = &planet_ids[rand_index];
            //
            //             println!("Explorer #{} is starting to think", self.get_base().explorer_id);
            //             self.get_base().to_orchestrator.send(ExplorerToOrchestrator::TravelToPlanetRequest {
            //                 explorer_id: self.get_base().explorer_id,
            //                 current_planet_id: current_planet_id.clone(),
            //                 dst_planet_id: target_planet.read().unwrap().value,
            //             }).unwrap();
            //         }
            //         _ => {
            //             println!("Task uncompletable for explorer #{}", self.get_base().explorer_id);
            //         }
            //     }
            // }
            //
            // else {
            //     println!("Explorer #{} non ha vicini in cui spostarsi", self.get_base().explorer_id);
            // }
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
