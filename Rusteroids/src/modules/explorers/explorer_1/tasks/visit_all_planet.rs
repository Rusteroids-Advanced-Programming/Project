use crate::modules::explorer_utils::tasks::{Task, TaskState};
use crate::modules::explorer_utils::tasks::TaskState::{Finished, Pending};

/// Tracks the progression of visiting a specified number of planets.
pub struct TotalPlanetsVisitedTask {
    state: TaskState,
    visited: usize,
    to_visit: usize
}

impl Task<usize> for TotalPlanetsVisitedTask {
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

impl TotalPlanetsVisitedTask {
    /// Creates a new task instance, initializing the number of visited planets to 1.
    pub fn new(to_visit: usize) -> Self {
        Self {state: Pending, visited: 1, to_visit}
    }

    /// Increments visited planets count and checks if the entire task is completed.
    pub fn update_progress(&mut self) {
        self.visited += 1;
        if self.visited >= self.to_visit {
            self.update_state(Finished);
        }
    }
}