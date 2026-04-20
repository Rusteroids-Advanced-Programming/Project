use std::collections::HashMap;
use common_game::components::resource::{ComplexResourceType};
use common_game::components::resource::GenericResource::ComplexResources;
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
        Self {state: Pending, crafted: HashMap::new()}
    }

    pub fn update_progress(&mut self, crafted_resource: ComplexResourceType) {
        self.crafted.insert(crafted_resource, true);
    }
}