use std::collections::HashMap;
use common_game::components::resource::{ComplexResourceType};
use common_game::components::resource::GenericResource::ComplexResources;
use crate::modules::explorer_utils::resource_types::get_all_complex_resource_types;
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorer_utils::tasks::TaskState::{Finished, Pending};

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
    pub fn new() -> Self {
        let all_complex = get_all_complex_resource_types();
        let mut crafted_map = HashMap::new();

        for resource in all_complex {
            crafted_map.insert(resource, false);
        }

        Self {state: Pending, crafted: crafted_map}
    }

    pub fn update_progress(&mut self, crafted_resource: ComplexResourceType) {
        self.crafted.insert(crafted_resource, true);

        //checking state of the task
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