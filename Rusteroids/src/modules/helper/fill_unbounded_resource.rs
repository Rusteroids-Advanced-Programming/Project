use common_game::components::resource::BasicResourceType::{Carbon, Hydrogen, Oxygen, Silicon};
use common_game::components::resource::ComplexResourceType::{
    AIPartner, Diamond, Dolphin, Life, Robot, Water,
};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};

const ALL_BASIC_RESOURCES: [BasicResourceType; 4] = [Hydrogen, Carbon, Silicon, Oxygen];
const ALL_COMPLEX_RESOURCES: [ComplexResourceType; 6] =
    [Diamond, Life, AIPartner, Robot, Water, Dolphin];

pub fn _get_unbounded_basic_resource_vec() -> Vec<BasicResourceType> {
    let mut result = Vec::new();
    for resource in ALL_BASIC_RESOURCES.iter().cloned() {
        result.push(resource)
    }
    result
}

pub fn _get_unbounded_complex_resource_vec() -> Vec<ComplexResourceType> {
    let mut result = Vec::new();
    for resource in ALL_COMPLEX_RESOURCES.iter().cloned() {
        result.push(resource)
    }
    result
}
