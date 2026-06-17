use crate::modules::helper::printable::Printable;
use common_game::components::planet::Planet;
use std::collections::HashMap;
use std::fmt::Debug;

#[allow(dead_code)]
pub struct PlanetMap {
    map: HashMap<u32, Planet>,
}

#[allow(dead_code)]
impl PlanetMap {
    pub fn new() -> Self {
        PlanetMap {
            map: HashMap::new(),
        }
    }

    pub fn add_planet(&mut self, planet: Planet, id: u32) {
        self.map.insert(id, planet);
    }

    pub fn get_planet_by_id(&self, planet_id: u32) -> Option<&Planet> {
        self.map.get(&planet_id)
    }
}

impl Debug for PlanetMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut result = String::new();
        for (id, planet) in self.map.iter() {
            result.push_str(format!("{}: {}", id, planet.to_string()).as_str());
        }
        write!(f, "{}", result)
    }
}