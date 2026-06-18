use std::collections::HashMap;
use common_game::components::resource::{ComplexResourceType};
use common_game::components::resource::GenericResource::ComplexResources;
use crate::modules::explorer_utils::resource_types::get_all_complex_resource_types;
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorer_utils::tasks::TaskState::{Finished, Pending};

/// Tracks the progression of crafting every available complex resource at least once.
pub struct CraftAllTask {
    state: TaskState,
    crafted: HashMap<ComplexResourceType, bool>
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

        Self {state: Pending, crafted: crafted_map}
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
            self.update_state(TaskState::Finished);
        }
    }
}