use common_game::components::energy_cell::EnergyCell;
use common_game::components::resource::{
    BasicResourceType, Combinator, ComplexResource, ComplexResourceType, GenericResource,
};

/// Extracts the `BasicResourceType` out of a `GenericResource`, returning
/// `None` if the generic resource is actually a complex one.
pub fn _from_generic_type_to_basic(resource: GenericResource) -> Option<BasicResourceType> {
    match resource {
        GenericResource::BasicResources(res) => Some(res.get_type()),
        _ => None,
    }
}

/// Crafts a complex resource of the requested type using `combinator`, the two
/// ingredients `lhs`/`rhs` and the energy taken from `cell`. Each branch
/// unwraps the generic ingredients into the concrete types expected by the
/// specific recipe; failures from the combinator are turned into `None`.
pub fn _gen_complex_resource(
    combinator: &Combinator,
    complex_resource_type: ComplexResourceType,
    cell: &mut EnergyCell,
    lhs: GenericResource,
    rhs: GenericResource,
) -> Option<ComplexResource> {
    let result: Option<ComplexResource>;

    match complex_resource_type {
        ComplexResourceType::Water => {
            let tmp =
                combinator.make_water(lhs.to_hydrogen().unwrap(), rhs.to_oxygen().unwrap(), cell);
            match tmp {
                Ok(w2) => {
                    result = Some(w2.to_complex());
                }
                Err(_e) => {
                    result = None;
                }
            }
        }

        ComplexResourceType::Diamond => {
            let tmp =
                combinator.make_diamond(lhs.to_carbon().unwrap(), rhs.to_carbon().unwrap(), cell);
            match tmp {
                Ok(d2) => {
                    result = Some(d2.to_complex());
                }
                Err(_e) => {
                    result = None;
                }
            }
        }

        ComplexResourceType::Life => {
            let tmp = combinator.make_life(lhs.to_water().unwrap(), rhs.to_carbon().unwrap(), cell);
            match tmp {
                Ok(d2) => {
                    result = Some(d2.to_complex());
                }
                Err(_e) => {
                    result = None;
                }
            }
        }

        ComplexResourceType::Robot => {
            let tmp =
                combinator.make_robot(lhs.to_silicon().unwrap(), rhs.to_life().unwrap(), cell);
            match tmp {
                Ok(d2) => {
                    result = Some(d2.to_complex());
                }
                Err(_e) => {
                    result = None;
                }
            }
        }

        ComplexResourceType::Dolphin => {
            let tmp =
                combinator.make_dolphin(lhs.to_water().unwrap(), rhs.to_life().unwrap(), cell);
            match tmp {
                Ok(d2) => {
                    result = Some(d2.to_complex());
                }
                Err(_e) => {
                    result = None;
                }
            }
        }

        ComplexResourceType::AIPartner => {
            let tmp =
                combinator.make_aipartner(lhs.to_robot().unwrap(), rhs.to_diamond().unwrap(), cell);
            match tmp {
                Ok(d2) => {
                    result = Some(d2.to_complex());
                }
                Err(_e) => {
                    result = None;
                }
            }
        }
    }

    result
}