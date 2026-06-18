use common_game::components::planet::PlanetType;
use common_game::utils::ID;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/// Selects which counter to increment on a planet's stats.
pub enum Counts {
    Asteroids,
    Sunrays,
    Rockets,
}

/// Serializable snapshot of a planet's full state, meant to be sent to the outside (e.g. a frontend).
#[derive(Debug, Serialize, Clone)]
pub struct PlanetDataDTO {
    pub id: u32,
    pub name: String,
    pub planet_type: String,
    pub alive: bool,
    pub energy_cells: u32,
    pub resources_base: Vec<String>,
    pub resources_complex: Vec<String>,
    pub neighbors: Vec<u32>,
    pub has_rocket: bool,
    pub stats: PlanetStatsDTO,
}

/// Serializable counters associated with a planet.
#[derive(Debug, Serialize, Clone)]
pub struct PlanetStatsDTO {
    pub asteroids: usize,
    pub sunrays: usize,
    pub rockets: usize,
}

/// Serializable snapshot of an explorer's state.
#[derive(Serialize)]
pub struct ExplorerDataDTO {
    pub id: u32,
    pub current_planet: u32,
    pub bag: Vec<String>,
    pub alive: bool,
}

/// Internal per-planet statistics tracked by the orchestrator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Stats {
    pub planet_name: String,
    pub planet_type: PlanetType,
    pub alive: bool,
    pub asteroid_count: usize,
    pub sunray_count: usize,
    pub rocket_used_count: usize, //add basic resource, option complex resource
}

impl Stats {
    /// Creates fresh stats for a planet: alive by default and all counters at zero.
    pub fn new(planet_name: String, planet_type: PlanetType) -> Self {
        Self {
            planet_name,
            planet_type,
            alive: true,
            asteroid_count: 0,
            sunray_count: 0,
            rocket_used_count: 0,
        }
    }
}

/// Owns the per-planet stats keyed by planet ID, with map-like access via Deref.
#[derive(Debug)]
pub struct StatsMap {
    map: HashMap<ID, Stats>,
}

impl StatsMap {
    pub fn new() -> Self {
        StatsMap {
            map: HashMap::new(),
        }
    }

    /// Registers a planet's stats, but only if it isn't already tracked (no overwrite).
    pub fn add_planet(&mut self, id: ID, planet_name: String, planet_type: PlanetType) {
        if !self.map.contains_key(&id) {
            self.map.insert(id, Stats::new(planet_name, planet_type));
        }
    }

    /// Increments the selected counter for a planet; does nothing if the planet is unknown.
    pub fn increase_count(&mut self, id: ID, count_type: Counts) {
        if self.map.contains_key(&id) {
            let tmp = self.map.get_mut(&id).unwrap();
            match count_type {
                Counts::Sunrays => tmp.sunray_count += 1,
                Counts::Asteroids => tmp.asteroid_count += 1,
                Counts::Rockets => tmp.rocket_used_count += 1,
            }
        }
    }

    /// Marks a planet as no longer alive; does nothing if the planet is unknown.
    pub fn planet_killed(&mut self, id: ID) {
        if self.map.contains_key(&id) {
            let tmp = self.map.get_mut(&id).unwrap();
            tmp.alive = false;
        }
    }
}

impl Deref for StatsMap {
    type Target = HashMap<ID, Stats>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for StatsMap {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.map
    }
}
