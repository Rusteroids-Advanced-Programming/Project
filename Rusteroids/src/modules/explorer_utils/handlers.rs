use common_game::components::resource::{BasicResource, ComplexResource, GenericResource};

/// Defines the callbacks used to handle the outcomes of AI actions.
pub trait AIHandlers {

    /// Starts the AI execution loop.
    fn start_ai_handler(&self);

    /// Resets the AI state to its initial configuration.
    fn reset_ai_handler(&self);

    /// Stops the AI and terminates its execution.
    fn kill_handler(&self);

    /// Handles the result of a resource generation attempt.
    fn generate_resource_handler(&self, result: &Option<&BasicResource>);

    /// Handles the result of a resource combination attempt.
    /// On success, returns the generated complex resource.
    /// On failure, provides an error message and the involved resources.
    fn combine_resource_handler(
        &self,
        result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>
    );

    /// Handles the completion of a planet movement action.
    fn move_to_planet_handler(&self);
}