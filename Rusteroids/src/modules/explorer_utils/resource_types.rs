use std::collections::HashSet;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};

pub fn get_all_complex_resource_types() -> HashSet<ComplexResourceType> {
    let mut result = HashSet::new();
    
    result.insert(ComplexResourceType::Diamond);
    result.insert(ComplexResourceType::Water);
    result.insert(ComplexResourceType::Life);
    result.insert(ComplexResourceType::AIPartner);
    result.insert(ComplexResourceType::Dolphin);
    result.insert(ComplexResourceType::Robot);
    
    result
}

pub fn get_all_bascic_resource_types() -> HashSet<BasicResourceType> {
    let mut result = HashSet::new();
    
    result.insert(BasicResourceType::Carbon);
    result.insert(BasicResourceType::Hydrogen);
    result.insert(BasicResourceType::Oxygen);
    result.insert(BasicResourceType::Silicon);
    
    result
}

