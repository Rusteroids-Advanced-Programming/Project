use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorer_utils::tasks::TaskState::{Finished, Pending};

pub struct TotalEdgesVisitedTask {
    state: TaskState,
    visited: usize,
    to_visit: usize
}

impl Task<usize> for TotalEdgesVisitedTask {
    fn get_state(&self) -> &TaskState {
        &self.state
    }

    fn update_state(&mut self, state: TaskState) {
        self.state = state;
    }

    fn get_progress(&self) -> usize {
        self.visited
    }
}

impl TotalEdgesVisitedTask {
    pub fn new(to_visit: usize) -> Self {
        Self {state: Pending, visited: 1, to_visit}
    }

    pub fn update_progress(&mut self) {
        self.visited += 1;
        if self.visited >= self.to_visit {
            self.update_state(Finished);
        }
    }
}