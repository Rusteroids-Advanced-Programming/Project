use crate::modules::explorer_utils::explorer::ExplorerBehaviour;
use crate::modules::explorer_utils::explorer_ai::ExplorerAI;
use crate::modules::explorer_utils::explorer_map::ExplorerMap;
use crate::modules::explorer_utils::get_random_index;
use crate::modules::explorer_utils::recipes::{get_complex_shopping_list, get_shopping_list};
use crate::modules::explorer_utils::resource_types::get_all_complex_resource_types;
use crate::modules::explorer_utils::tasks::Task;
use crate::modules::explorers::common_tasks::craft_all::CraftAllTask;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::utils::ID;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};

pub type MissingBasicResources = HashMap<BasicResourceType, usize>;
pub type MissingComplexResources = HashMap<ComplexResourceType, usize>;

/// Computes the absolute amount of basic resources still needed to fulfill the remaining crafting tasks.
pub fn get_missing_basic_resources<T: ExplorerBehaviour>(
    explorer: &T,
    task_guard: RwLockReadGuard<CraftAllTask>,
) -> MissingBasicResources {
    let mut missing_resources: MissingBasicResources = HashMap::new();
    let base_guard = explorer.get_base();
    let bag_guard = base_guard.bag.read().unwrap();
    let dummy_bag = bag_guard.to_dummy();

    let task_state = task_guard.get_progress();

    // Accumulate missing materials across all uncompleted complex targets
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

    // Subtract resources currently stored in the bag from the absolute needed amount
    for (resource, qty) in &mut missing_resources {
        let already_have = dummy_bag.get_basic_quantity(resource);
        if already_have <= *qty {
            *qty -= already_have;
        } else {
            *qty = 0;
        }
    }

    missing_resources
}

/// Calculates missing complex resources required by analyzing recipe hierarchies against current bag contents.
pub fn get_missing_complex_resources<T: ExplorerBehaviour>(
    explorer: &T,
) -> MissingComplexResources {
    let mut missing_resources: MissingComplexResources = HashMap::new();
    let base_guard = explorer.get_base();
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

    // Deduct already possessed complex items from the required checklist
    for (resource, qty) in &mut missing_resources {
        let already_have = dummy_bag.get_complex_quantity(resource);
        if already_have <= *qty {
            *qty -= already_have;
        } else {
            *qty = 0;
        }
    }

    missing_resources
}

/// Selects a neighboring destination based on missing resource availability and issues a travel request.
pub fn change_planet<T: ExplorerBehaviour>(
    explorer: &T,
    missing_resources: &MissingBasicResources,
    explorer_map_guard: RwLockReadGuard<ExplorerMap>,
    prev_planet: Arc<RwLock<ID>>,
) {
    let base_guard = explorer.get_base();
    let neighbours = base_guard.neighbours.read().unwrap();

    if neighbours.len() == 0 {
        return;
    }

    let mut next_planet: ID = neighbours[get_random_index(neighbours.len())];

    // Route optimization: prioritize unvisited planets or those containing missing raw materials
    for neighbour in &*neighbours {
        let planet_infos = explorer_map_guard.infos.get(&neighbour);
        match planet_infos {
            None => {
                next_planet = *neighbour;
                break;
            }
            Some(planet_infos) => {
                let mut _basic_planet_found = false;

                for (basic_resource, qty) in missing_resources {
                    if qty > &0 {
                        let tmp = planet_infos.basic_resources.get(basic_resource);
                        if let Some(_basic_resource) = tmp {
                            // Avoid immediate backtracking to the previous planet if possible
                            if &*prev_planet.read().unwrap() != neighbour {
                                _basic_planet_found = true;
                                next_planet = neighbour.clone();
                            }
                        }
                    }
                }
            }
        }
    }

    let mut prev_guard = prev_planet.write().unwrap();
    *prev_guard = base_guard.current_planet_id.read().unwrap().clone();

    base_guard.travel_request(next_planet);
}
