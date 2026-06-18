use crate::modules::helper::printable::Printable;
use common_game::components::planet::Planet;
use std::collections::HashMap;
use std::fmt::Debug;

#[allow(dead_code)]
/// A synchronized map storage layer caching and mapping unique planet IDs to their active domain objects.
pub struct PlanetMap {
    map: HashMap<u32, Planet>,
}

#[allow(dead_code)]
impl PlanetMap {
    /// Constructs a clean, unallocated planetary lookup registry map shell.
    pub fn new() -> Self {
        PlanetMap {
            map: HashMap::new(),
        }
    }

    /// Registers or overrides an operational planet structure bound to a specific global numerical identification key.
    pub fn add_planet(&mut self, planet: Planet, id: u32) {
        self.map.insert(id, planet);
    }

    /// Retreives an immutable reference placeholder linking back to the matching entity model inside the storage array.
    pub fn get_planet_by_id(&self, planet_id: u32) -> Option<&Planet> {
        self.map.get(&planet_id)
    }
}

impl Debug for PlanetMap {
    /// Iterates through the raw internal key-value mapping to construct a human-readable aggregate tracking dump.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut result = String::new();

        // Loop over the key-value storage records and format them via their custom native string representation
        for (id, planet) in self.map.iter() {
            result.push_str(format!("{}: {}\n", id, planet.to_string()).as_str());
        }

        write!(f, "{}", result)
    }
}