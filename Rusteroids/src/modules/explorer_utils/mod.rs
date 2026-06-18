use rand::Rng;

pub mod bag_type;
pub mod explorer;
pub mod explorer_ai;
pub mod explorer_base;
pub mod explorer_map;
pub mod handlers;
pub mod planet_infos;
pub mod recipes;
pub mod resource_types;
pub mod tasks;

/// Thread-safe helper that generates a random index safely using the thread-local RNG.
pub fn get_random_index(length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    let mut rng = rand::rng();
    rng.random_range(0..length)
}
