use crate::modules::orchestrator::orchestrator::Orchestrator;
use common_game::components::planet::DummyPlanetState;
use common_game::utils::ID;

mod get_planet_state;
mod send_asteroid;
mod send_ingoing_explorer;
pub mod send_outgoing_explorer;
mod send_sunray;
mod start_planet_ai;
mod stop_planet_ai;

use get_planet_state::get_planet_state_impl;
use send_asteroid::send_asteroid_impl;
use send_ingoing_explorer::send_ingoing_explorer_impl;
use send_outgoing_explorer::send_outgoing_explorer_impl;
use send_sunray::send_sunray_impl;
use start_planet_ai::start_planet_ai_impl;
use stop_planet_ai::stop_planet_ai_impl;

#[allow(dead_code)]
pub trait OrchestratorAI {
    fn send_sunray(&self, target: u32) -> Option<u32>;
    fn start_planet_ai(&self, target: u32);
    fn get_planet_state(&self, target: u32) -> Option<DummyPlanetState>;
    fn send_asteroid(&self, target: u32);
    fn stop_planet_ai(&self, target: u32);
    fn send_ingoing_explorer(&self, planet_target: ID, explorer_target: ID) -> bool;
    fn send_outgoing_explorer(&self, target: u32) -> bool;
}

impl OrchestratorAI for Orchestrator {
    fn send_sunray(&self, target: u32) -> Option<u32> {
        send_sunray_impl(self, target)
    }

    fn start_planet_ai(&self, target: u32) {
        start_planet_ai_impl(self, target);
    }

    fn get_planet_state(&self, target: u32) -> Option<DummyPlanetState> {
        get_planet_state_impl(self, target)
    }
    fn send_asteroid(&self, target: u32) {
        send_asteroid_impl(self, target);
    }
    fn stop_planet_ai(&self, target: u32) {
        stop_planet_ai_impl(self, target);
    }

    fn send_ingoing_explorer(&self, planet_target: ID, explorer_target: ID) -> bool {
        send_ingoing_explorer_impl(self, planet_target, explorer_target)
    }

    fn send_outgoing_explorer(&self, target: u32) -> bool {
        send_outgoing_explorer_impl(self, target)
    }
}
