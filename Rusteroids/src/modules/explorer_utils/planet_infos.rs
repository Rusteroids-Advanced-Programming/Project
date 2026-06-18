use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use std::collections::HashSet;

/// Holds the resource-related information gathered about a planet,
/// distinguishing between raw basic resources and processed complex ones.
#[derive(Debug)]
pub struct PlanetInfos {
    pub basic_resources: HashSet<BasicResourceType>,
    pub complex_resources: HashSet<ComplexResourceType>,
}

impl PlanetInfos {
    pub fn new(
        basic_resources: HashSet<BasicResourceType>,
        complex_resources: HashSet<ComplexResourceType>,
    ) -> PlanetInfos {
        Self {
            basic_resources,
            complex_resources,
        }
    }
}
