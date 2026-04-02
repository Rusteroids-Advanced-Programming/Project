use crate::modules::orchestrator::orchestator::Orchestrator;
use combine_resources::combine_resources_impl;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::utils::ID;
use generate_resource::generate_resource_impl;
use get_explorer_bag::get_explorer_bag_impl;
use get_explorer_neighbours::get_explorer_neighbours_impl;
use get_explorer_planet::get_explorer_planet_impl;
use get_supported_combination::get_supported_combinations_impl;
use get_supported_resources::get_supported_resources_impl;
use kill_explorer::kill_explorer_impl;
use move_explorer::move_explorer_impl;
use reset_explorer::reset_explorer_impl;
use start_explorer::start_explorer_impl;

mod combine_resources;
mod generate_resource;
mod get_explorer_bag;
mod get_explorer_neighbours;
mod get_explorer_planet;
mod get_supported_combination;
mod get_supported_resources;
mod kill_explorer;
mod move_explorer;
mod reset_explorer;
mod start_explorer;

#[allow(dead_code)]
pub trait HandlerExplorer {
    fn start_explorer(&self, expl_id: ID);
    fn reset_explorer(&self, expl_id: ID);
    fn kill_explorer(&self, expl_id: ID);
    fn move_explorer(&self, expl_id: ID, planet_id: ID);
    fn get_explorer_planet(&self, expl_id: ID);
    fn get_supported_resource(&self, expl_id: ID);
    fn get_supported_combinations(&self, expl_id: ID);
    fn generate_resource(&self, expl_id: ID, to_generate: BasicResourceType);
    fn combine_resource(&self, expl_id: ID, to_combine: ComplexResourceType);
    fn get_explorer_bag(&self, expl_id: ID);
    fn get_explorer_neighbours(&self, expl_id: ID);
}

impl HandlerExplorer for Orchestrator {
    fn start_explorer(&self, expl_id: ID) {
        start_explorer_impl(self, expl_id);
    }

    fn reset_explorer(&self, expl_id: ID) {
        reset_explorer_impl(self, expl_id);
    }

    fn kill_explorer(&self, expl_id: ID) {
        kill_explorer_impl(self, expl_id);
    }

    fn move_explorer(&self, expl_id: ID, planet_id: ID) {
        move_explorer_impl(self, expl_id, planet_id);
    }

    fn get_explorer_planet(&self, expl_id: ID) {
        get_explorer_planet_impl(self, expl_id);
    }

    fn get_supported_resource(&self, expl_id: ID) {
        get_supported_resources_impl(self, expl_id);
    }

    fn get_supported_combinations(&self, expl_id: ID) {
        get_supported_combinations_impl(self, expl_id);
    }

    fn generate_resource(&self, expl_id: ID, to_generate: BasicResourceType) {
        generate_resource_impl(self, expl_id, to_generate);
    }

    fn combine_resource(&self, expl_id: ID, to_generate: ComplexResourceType) {
        combine_resources_impl(self, expl_id, to_generate);
    }

    fn get_explorer_bag(&self, expl_id: ID) {
        get_explorer_bag_impl(self, expl_id);
    }

    fn get_explorer_neighbours(&self, expl_id: ID) {
        get_explorer_neighbours_impl(self, expl_id);
    }
}
