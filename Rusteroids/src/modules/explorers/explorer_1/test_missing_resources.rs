use std::any::{Any, TypeId};
use common_game::components::resource::{BasicResourceType, ComplexResourceType, ResourceType};

#[test]
fn test_missing_resources() {
    let tmp = ResourceType::make_aipartner();
    assert_eq!(tmp.type_id(), TypeId::of::<ComplexResourceType>())
}