use std::collections::HashSet;
use common_game::components::resource::{
    BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource, ResourceType,
};
use crate::modules::manual_explorer::bag_type::DummyBag;

pub fn complex_resource_type_to_request(
    complex_type: &ComplexResourceType,
    lhs: GenericResource,
    rhs: GenericResource,
) -> ComplexResourceRequest {
    let result: ComplexResourceRequest;

    match complex_type {
        ComplexResourceType::AIPartner => {
            result = ComplexResourceRequest::AIPartner(
                lhs.to_robot().unwrap(),
                rhs.to_diamond().unwrap(),
            )
        }
        ComplexResourceType::Robot => {
            result =
                ComplexResourceRequest::Robot(lhs.to_silicon().unwrap(), rhs.to_life().unwrap())
        }
        ComplexResourceType::Diamond => {
            result =
                ComplexResourceRequest::Diamond(lhs.to_carbon().unwrap(), rhs.to_carbon().unwrap())
        }
        ComplexResourceType::Dolphin => {
            result =
                ComplexResourceRequest::Dolphin(lhs.to_water().unwrap(), rhs.to_life().unwrap())
        }
        ComplexResourceType::Water => {
            result =
                ComplexResourceRequest::Water(lhs.to_hydrogen().unwrap(), rhs.to_oxygen().unwrap())
        }
        ComplexResourceType::Life => {
            result = ComplexResourceRequest::Life(lhs.to_water().unwrap(), rhs.to_carbon().unwrap())
        }
    }

    result
}


pub fn get_recipe(complex_resource_type: &ComplexResourceType) -> (ResourceType, ResourceType) {
    let result: (ResourceType, ResourceType);
    match complex_resource_type {
        ComplexResourceType::AIPartner => {
            result = (
                ResourceType::Complex(ComplexResourceType::Robot),
                ResourceType::Complex(ComplexResourceType::Diamond),
            )
        }
        ComplexResourceType::Diamond => {
            result = (
                ResourceType::Basic(BasicResourceType::Carbon),
                ResourceType::Basic(BasicResourceType::Carbon),
            )
        }
        ComplexResourceType::Dolphin => {
            result = (
                ResourceType::Complex(ComplexResourceType::Water),
                ResourceType::Complex(ComplexResourceType::Life),
            )
        }
        ComplexResourceType::Water => {
            result = (
                ResourceType::Basic(BasicResourceType::Hydrogen),
                ResourceType::Basic(BasicResourceType::Oxygen),
            )
        }
        ComplexResourceType::Life => {
            result = (
                ResourceType::Complex(ComplexResourceType::Water),
                ResourceType::Basic(BasicResourceType::Carbon),
            )
        }
        ComplexResourceType::Robot => {
            result = (
                ResourceType::Basic(BasicResourceType::Silicon),
                ResourceType::Complex(ComplexResourceType::Life),
            )
        }
    }
    result
}


pub fn get_shopping_list(bag: &DummyBag, complex_resource_type: &ComplexResourceType) -> HashSet<ResourceType> {
    let mut result = HashSet::new();

    let (lhs, rhs) = get_recipe(complex_resource_type);
    if !bag.is_in_bag(&lhs) { result.insert(lhs); }
    if !bag.is_in_bag(&rhs) { result.insert(rhs); }

    result
}