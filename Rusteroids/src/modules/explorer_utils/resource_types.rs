use std::collections::HashSet;
use common_game::components::resource::{BasicResourceType, ComplexResourceType, ResourceType};

/// Returns the set of every existing complex resource type.
pub fn get_all_complex_resource_types() -> HashSet<ComplexResourceType> {
    let mut result = HashSet::new();

    result.insert(ComplexResourceType::AIPartner);
    result.insert(ComplexResourceType::Dolphin);
    result.insert(ComplexResourceType::Robot);
    result.insert(ComplexResourceType::Life);
    result.insert(ComplexResourceType::Water);
    result.insert(ComplexResourceType::Diamond);

    result
}

/// Returns the set of every existing basic resource type.
pub fn get_all_bascic_resource_types() -> HashSet<BasicResourceType> {
    let mut result = HashSet::new();

    result.insert(BasicResourceType::Carbon);
    result.insert(BasicResourceType::Hydrogen);
    result.insert(BasicResourceType::Oxygen);
    result.insert(BasicResourceType::Silicon);

    result
}

/// Unwraps a `ResourceType` into its inner variant, using `Result` as a
/// two-way discriminator: `Ok` for basic resources, `Err` for complex ones.
/// The `Err` variant does not represent a failure, just the complex case.
pub fn resource_type_to_inner(resource_type: ResourceType) -> Result<BasicResourceType, ComplexResourceType> {
    match resource_type {
        ResourceType::Basic(resource_type) => Ok(resource_type),
        ResourceType::Complex(resource_type) => Err(resource_type),
    }
}
