use crate::modules::explorer_utils::explorer::ExplorerBehaviour;
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_map::ExplorerMap;
use crate::modules::explorer_utils::resource_types::get_all_complex_resource_types;
use crate::modules::explorer_utils::tasks::TaskState::{Finished, Pending};
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorers::common_tasks::utils::{
    change_planet, get_missing_basic_resources, get_missing_complex_resources,
};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::utils::ID;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

/// Tracks the progression of crafting every available complex resource at least once.
pub struct CraftAllTask {
    state: TaskState,
    crafted: HashMap<ComplexResourceType, bool>,
}

impl Task<HashMap<ComplexResourceType, bool>> for CraftAllTask {
    fn get_state(&self) -> &TaskState {
        &self.state
    }

    fn update_state(&mut self, state: TaskState) {
        self.state = state;
    }

    fn get_progress(&self) -> HashMap<ComplexResourceType, bool> {
        self.crafted.clone()
    }
}

impl CraftAllTask {
    /// Creates a new task instance, initializing all available complex resources to `false`.
    pub fn new() -> Self {
        // Dynamically fetch all existing complex resource types to populate the progress map
        let all_complex = get_all_complex_resource_types();
        let mut crafted_map = HashMap::new();

        for resource in all_complex {
            crafted_map.insert(resource, false);
        }

        Self {
            state: Pending,
            crafted: crafted_map,
        }
    }

    /// Marks a resource as crafted and checks if the entire task is completed.
    pub fn update_progress(&mut self, crafted_resource: ComplexResourceType) {
        self.crafted.insert(crafted_resource, true);

        // Scan the map to verify if there is any resource left uncrafted
        let mut completed = true;
        for resource in self.crafted.values() {
            if !resource {
                completed = false;
            }
        }

        if completed {
            self.update_state(Finished);
        }
    }

    /// Evaluates environmental conditions (energy cells, recipes) to trigger either crafting, gathering, or relocation.
    pub fn resolve_task<T: ExplorerBehaviour>(
        explorer: &T,
        craft_all_task: Arc<RwLock<CraftAllTask>>,
        explorer_map: Arc<RwLock<ExplorerMap>>,
        prev_planet: Arc<RwLock<ID>>,
    ) {
        let base_guard = explorer.get_base();
        let mut num_cells = base_guard.ask_available_cells();
        let missing_basic_resources =
            get_missing_basic_resources(explorer, craft_all_task.read().unwrap());
        let missing_complex_resources = get_missing_complex_resources(explorer);

        //DECISION TREE:
        // - 1 check if current planet can craft
        // - 2 check bag
        // - 3 check if I have the ingredients to craft
        // - 4 if I can craft I do it, else I check if I can extract
        // - 5 Move to a planet which permits to craft missing complex resources or to extract basic resources needed to craft

        if num_cells == 0 {
            change_planet(
                explorer,
                &missing_basic_resources,
                explorer_map.read().unwrap(),
                prev_planet,
            );
        } else {
            let explorer_map_guard = explorer_map.read().unwrap();
            let planet_infos = explorer_map_guard
                .infos
                .get(&base_guard.current_planet_id.read().unwrap())
                .unwrap();
            let mut resource_target: Option<BasicResourceType> = None;

            while num_cells >= 1 {
                sleep(Duration::from_millis(1000));
                let _task_state = craft_all_task.read().unwrap().get_progress();

                if planet_infos.complex_resources.len() > 0 {
                    for combination in &planet_infos.complex_resources {
                        let qty_needed = missing_complex_resources.get(&combination).unwrap_or(&0);
                        if qty_needed > &0 {
                            num_cells = base_guard.ask_available_cells();
                            if num_cells > 0 {
                                base_guard.combine_resource(combination.clone(), |res| {
                                    explorer.combine_resource_handler(res)
                                })
                            }
                        }
                    }
                }

                // If crafting targets are unmet, find an extractable basic resource currently needed
                for (resource, qty) in &missing_basic_resources {
                    if qty > &0 && planet_infos.basic_resources.contains(resource) {
                        resource_target = Some(*resource);
                        break;
                    }
                }

                if let Some(resource_target_inner) = resource_target {
                    let gen_res = base_guard.generate_resource(resource_target_inner, |result| {
                        explorer.generate_resource_handler(result)
                    });

                    // Change planet immediately if extraction fails (e.g. source depleted)
                    if let Err(_) = gen_res {
                        change_planet(
                            explorer,
                            &missing_basic_resources,
                            explorer_map.read().unwrap(),
                            prev_planet.clone(),
                        );
                        break;
                    }

                    resource_target = None;
                    num_cells = base_guard.ask_available_cells();
                } else {
                    break;
                }
            }
            change_planet(
                explorer,
                &missing_basic_resources,
                explorer_map.read().unwrap(),
                prev_planet,
            );
        }
    }
}
