use common_game::components::resource::{BasicResource, ComplexResource, GenericResource};

/// Defines the custom handlers called in the corresponding methods of the explorer.
pub trait AIHandlers {
    fn start_ai_handler(&self);

    fn reset_ai_handler(&self);

    fn kill_handler(&self);

    fn generate_resource_handler(&self, result: &Option<&BasicResource>);

    fn combine_resource_handler(
        &self,
        result: &Result<&ComplexResource, &(String, GenericResource, GenericResource)>,
    );
    fn move_to_planet_handler(&self);
}
