use crate::modules::explorer_utils::tasks::TaskState::{Finished, Pending};
use crate::modules::explorer_utils::tasks::{Task, TaskState};
use common_game::utils::ID;
use std::collections::HashMap;

/// Minimum number of times each distinct planet must be visited to count toward the goal.
const REQUIRED_VISITS: usize = 3;

/// Tracks visiting a target number of *distinct* planets, each at least REQUIRED_VISITS times.
pub struct TotalPlanetsVisitedTask {
    state: TaskState,
    // Per-planet visit count
    visits: HashMap<ID, usize>,
    to_visit: usize,
}

impl Task<usize> for TotalPlanetsVisitedTask {
    fn get_state(&self) -> &TaskState {
        &self.state
    }

    fn update_state(&mut self, state: TaskState) {
        self.state = state;
    }

    /// Progress is the number of planets that have reached the required visit count.
    fn get_progress(&self) -> usize {
        self.count_satisfied()
    }
}

impl TotalPlanetsVisitedTask {
    /// Initializes the task, recording the explorer's starting planet as visited once.
    pub fn new(to_visit: usize, start_planet: ID) -> Self {
        let mut visits = HashMap::new();
        visits.insert(start_planet, 1);

        let mut task = Self {
            state: Pending,
            visits,
            to_visit,
        };
        task.refresh_state();
        task
    }

    /// Records a visit to a planet and re-evaluates completion.
    pub fn update_progress(&mut self, planet_id: ID) {
        *self.visits.entry(planet_id).or_insert(0) += 1;
        self.refresh_state();
    }

    /// Whether a planet has been visited enough times to count toward the goal.
    pub fn is_satisfied(&self, planet_id: &ID) -> bool {
        self.visits.get(planet_id).copied().unwrap_or(0) >= REQUIRED_VISITS
    }

    /// Number of distinct planets that have reached the required visit count.
    fn count_satisfied(&self) -> usize {
        self.visits
            .values()
            .filter(|&&v| v >= REQUIRED_VISITS)
            .count()
    }

    /// Marks the task Finished once enough planets are satisfied.
    fn refresh_state(&mut self) {
        if self.count_satisfied() >= self.to_visit {
            self.update_state(Finished);
        }
    }
}
