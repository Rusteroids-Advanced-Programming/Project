use std::any::Any;
use std::collections::{HashMap, HashSet};
use common_game::components::resource::{
    BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource, ResourceType,
};
use common_game::components::resource::ResourceType::Basic;
use crate::modules::explorer_utils::resource_types::resource_type_to_inner;
use crate::modules::explorer_utils::bag_type::DummyBag;

/// Builds a `ComplexResourceRequest` for the given complex resource type, using
/// `lhs` and `rhs` as the two ingredients required by its recipe. Each branch
/// unwraps the generic resources into the specific basic/complex variants
/// expected by that recipe.
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


/// Returns the recipe (pair of ingredients) needed to craft the given
/// complex resource. Ingredients can be either basic or other complex resources.
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

/// Returns the list of basic resources still missing from `bag` in order to
/// craft `complex_resource_type`. Resources already owned are reused (each unit
/// only once) thanks to the recursive helper that tracks virtual consumption.
pub fn get_shopping_list(bag: &DummyBag, complex_resource_type: &ComplexResourceType) -> Vec<BasicResourceType> {
    let mut consumed = HashMap::new();
    get_shopping_list_helper(bag, complex_resource_type, &mut consumed)
}


/// Recursive helper for `get_shopping_list`. `consumed` tracks how many units
/// of each resource have already been virtually taken from the bag during this
/// traversal, so the same physical item is never counted twice when it appears
/// as an ingredient on multiple branches of the recipe tree.
fn get_shopping_list_helper(
    bag: &DummyBag,
    complex_resource_type: &ComplexResourceType,
    consumed: &mut HashMap<ResourceType, usize>
) -> Vec<BasicResourceType> {
    let mut result = Vec::new();
    let (lhs, rhs) = get_recipe(complex_resource_type);

    let mut needed_left: Vec<BasicResourceType> = Vec::new();
    let mut needed_right: Vec<BasicResourceType> = Vec::new();

    // -- LHS check --
    let lhs_consumed = *consumed.get(&lhs).unwrap_or(&0);
    if bag.get_resource_quantity(&lhs) > lhs_consumed {
        // We still have an unused unit in the bag: virtually consume it.
        consumed.insert(lhs.clone(), lhs_consumed + 1);
    } else {
        // Not available (or already exhausted virtually): recurse on sub-recipe
        // if complex, otherwise add the basic resource to the shopping list.
        let left_inner = resource_type_to_inner(lhs.clone());
        if let Err(complex_res) = left_inner {
            let mut tmp = get_shopping_list_helper(bag, &complex_res, consumed);
            needed_left.append(&mut tmp);
        } else if let Ok(basic_res) = left_inner {
            needed_left.push(basic_res);
        }
    }

    // -- RHS check (same logic as LHS) --
    let rhs_consumed = *consumed.get(&rhs).unwrap_or(&0);
    if bag.get_resource_quantity(&rhs) > rhs_consumed {
        consumed.insert(rhs.clone(), rhs_consumed + 1);
    } else {
        let right_inner = resource_type_to_inner(rhs.clone());
        if let Err(complex_res) = right_inner {
            let mut tmp = get_shopping_list_helper(bag, &complex_res, consumed);
            needed_right.append(&mut tmp);
        } else if let Ok(basic_res) = right_inner {
            needed_right.push(basic_res);
        }
    }

    result.append(&mut needed_left);
    result.append(&mut needed_right);

    result
}


// pub fn get_shopping_list(bag: &DummyBag, complex_resource_type: &ComplexResourceType) -> Vec<BasicResourceType> {
//     let mut result = Vec::new();
//
//     let (lhs, rhs) = get_recipe(complex_resource_type);
//
//     let mut needed_left: Vec<BasicResourceType> = Vec::new();
//     let mut needed_right: Vec<BasicResourceType> = Vec::new();
//
//     let left_inner = resource_type_to_inner(lhs);
//     let right_inner = resource_type_to_inner(rhs);
//
//     if let Err(complex_res) = left_inner {
//         let mut tmp = get_shopping_list(bag, &complex_res);
//         needed_left.append(&mut tmp);
//     }
//     else if let Ok(basic_res) = left_inner {
//         needed_left.push(basic_res);
//     }
//
//     if let Err(complex_res) = right_inner {
//         let mut tmp = get_shopping_list(bag, &complex_res);
//         needed_right.append(&mut tmp);
//     }
//     else if let Ok(basic_res) = right_inner {
//         needed_right.push(basic_res);
//     }
//
//     // if !bag.is_in_bag(&lhs) { result.append(&mut needed_left); }
//     // if !bag.is_in_bag(&rhs) { result.append(&mut needed_right); }
//     result.append(&mut needed_left);
//     result.append(&mut needed_right);
//
//     result
// }

// pub fn get_complex_shopping_list(bag: &DummyBag, complex_resource_type: &ComplexResourceType) -> Vec<ComplexResourceType> {
//     let mut result = Vec::new();
//     let (lhs, rhs) = get_recipe(complex_resource_type);
//     let mut needed_left: Vec<ComplexResourceType> = Vec::new();
//     let mut needed_right: Vec<ComplexResourceType> = Vec::new();
//
//     let left_inner = resource_type_to_inner(lhs);
//     let right_inner = resource_type_to_inner(rhs);
//
//     if let Err(complex_res) = left_inner {
//         let mut tmp = get_complex_shopping_list(bag, &complex_res);
//         needed_left.append(&mut tmp);
//         needed_left.push(complex_res);
//     }
//
//     if let Err(complex_res) = right_inner {
//         let mut tmp = get_complex_shopping_list(bag, &complex_res);
//         needed_right.append(&mut tmp);
//         needed_right.push(complex_res);
//     }
//
//     result.append(&mut needed_left);
//     result.append(&mut needed_right);
//     result.push(complex_resource_type.clone());
//
//     result
// }

/// Returns the ordered list of complex resources that need to be crafted in
/// order to obtain `complex_resource_type`. The order is bottom-up: each
/// resource appears after all of its complex dependencies, so the list can be
/// consumed sequentially by a crafter.
pub fn get_complex_shopping_list(bag: &DummyBag, complex_resource_type: &ComplexResourceType) -> Vec<ComplexResourceType> {
    let mut consumed = HashMap::new();
    get_complex_shopping_list_helper(bag, complex_resource_type, &mut consumed)
}

/// Recursive helper for `get_complex_shopping_list`. Same virtual-consumption
/// trick as `get_shopping_list_helper`: items already in the bag are not
/// scheduled for crafting, and each unit is only reused once.
fn get_complex_shopping_list_helper(
    bag: &DummyBag,
    complex_resource_type: &ComplexResourceType,
    consumed: &mut HashMap<ResourceType, usize>
) -> Vec<ComplexResourceType> {
    let mut result = Vec::new();
    let (lhs, rhs) = get_recipe(complex_resource_type);

    let mut needed_left: Vec<ComplexResourceType> = Vec::new();
    let mut needed_right: Vec<ComplexResourceType> = Vec::new();

    // -- LHS check --
    let lhs_consumed = *consumed.get(&lhs).unwrap_or(&0);
    if bag.get_resource_quantity(&lhs) > lhs_consumed {
        consumed.insert(lhs.clone(), lhs_consumed + 1);
    } else {
        // Only recurse on complex ingredients: basic ones are not part of this list.
        let left_inner = resource_type_to_inner(lhs.clone());
        if let Err(complex_res) = left_inner {
            let mut tmp = get_complex_shopping_list_helper(bag, &complex_res, consumed);
            needed_left.append(&mut tmp);
        }
    }

    // -- RHS check (same logic as LHS) --
    let rhs_consumed = *consumed.get(&rhs).unwrap_or(&0);
    if bag.get_resource_quantity(&rhs) > rhs_consumed {
        consumed.insert(rhs.clone(), rhs_consumed + 1);
    } else {
        let right_inner = resource_type_to_inner(rhs.clone());
        if let Err(complex_res) = right_inner {
            let mut tmp = get_complex_shopping_list_helper(bag, &complex_res, consumed);
            needed_right.append(&mut tmp);
        }
    }

    result.append(&mut needed_left);
    result.append(&mut needed_right);

    // Push the current resource last so dependencies come before it.
    result.push(complex_resource_type.clone());

    result
}