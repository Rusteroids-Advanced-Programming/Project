use crate::modules::explorer_utils::tasks::TaskState::{Finished, Pending};
use crate::modules::explorer_utils::tasks::{Task, TaskState};

/// Tracks the progression of discovering or traversing a specific number of unique map edges.
pub struct TotalEdgesVisitedTask {
    state: TaskState,
    visited: usize,
    to_visit: usize,
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
    /// Initializes the task with a target goal, counting the starting edge as already visited.
    pub fn new(to_visit: usize) -> Self {
        Self {
            state: Pending,
            visited: 0,
            to_visit,
        }
    }

    /// Increments the edge counter and automatically completes the task if the target is met.
    pub fn update_progress(&mut self) {
        self.visited += 1;

        if self.visited >= self.to_visit {
            self.update_state(Finished);
        }
    }
}
