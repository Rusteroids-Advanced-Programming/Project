pub trait Task <Progress> {
    fn get_state(&self) -> &TaskState;
    fn update_state(&mut self, state: TaskState);
    fn get_progress(&self) -> Progress;
}

pub enum TaskState {
    Finished,
    Pending,
    Uncompletable
}