use std::collections::HashSet;
use common_game::components::planet::PlanetType;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::utils::ID;

#[derive(Debug)]
pub struct PlanetInfos {
    // pub id: ID,
    // pub planet_type: PlanetType,
    pub basic_resources: HashSet<BasicResourceType>,
    pub complex_resources: HashSet<ComplexResourceType>,
}

impl PlanetInfos {
    pub fn new(basic_resources: HashSet<BasicResourceType>, complex_resources: HashSet<ComplexResourceType>) -> PlanetInfos {
        Self {basic_resources, complex_resources}
    }
}
