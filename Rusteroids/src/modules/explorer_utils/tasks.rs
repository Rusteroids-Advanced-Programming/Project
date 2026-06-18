/// Generic interface for any long-running task whose progress can be inspected
/// and whose state can be queried and updated externally.
/// The `Progress` type parameter lets each implementor expose its own
/// progress representation (percentage, step count, custom struct, ...).
pub trait Task<Progress> {
    fn get_state(&self) -> &TaskState;
    fn update_state(&mut self, state: TaskState);
    fn get_progress(&self) -> Progress;
}

/// Lifecycle states a `Task` can be in.
/// `Uncompletable` marks tasks that cannot terminate successfully (e.g. missing
/// prerequisites) and should be dropped or retried instead of being polled.
#[derive(Clone)]
pub enum TaskState {
    Finished,
    Pending,
    Uncompletable,
}
