use std::any::Any;
use common_game::components::resource::{
    BasicResource, BasicResourceType, ComplexResource, ComplexResourceType, GenericResource,
    ResourceType,
};
use std::collections::HashMap;
use std::hash::Hash;

#[allow(dead_code)]
#[derive(Debug)]
pub struct DummyBag {
    pub complex: HashMap<ComplexResourceType, usize>, //rese pub, va bene?
    pub basic: HashMap<BasicResourceType, usize>,
}

#[allow(dead_code)]
impl DummyBag {
    pub fn new(
        basic: HashMap<BasicResourceType, usize>,
        complex: HashMap<ComplexResourceType, usize>,
    ) -> Self {
        Self { complex, basic }
    }

    pub fn is_empty(&self) -> bool {
        self.complex.is_empty() && self.basic.is_empty()
    }

    pub fn get_basic_quantity(&self, basic_resource_type: &BasicResourceType) -> usize {
        let tmp = self.basic.get(basic_resource_type);
        match tmp {
            Some(qty) => *qty,
            None => 0,
        }
    }

    pub fn get_complex_quantity(&self, complex_resource_type: &ComplexResourceType) -> usize {
        let tmp = self.complex.get(complex_resource_type);
        match tmp {
            Some(qty) => *qty,
            None => 0,
        }
    }

    pub fn get_resource_quantity(&self, resource: &ResourceType) -> usize {
        match resource {
            ResourceType::Basic(basic_type) => {
                self.get_basic_quantity(basic_type)
            },
            ResourceType::Complex(complex_type) => {
                self.get_complex_quantity(complex_type)
            }
        }
    }

    pub fn is_basic_in_bag(&self, basic_resource_type: &BasicResourceType) -> bool {
        let tmp = self.basic.get(basic_resource_type);
        match tmp {
            None => false,
            Some(qty) => {
                qty >= &1
            }
        }
    }

    pub fn is_complex_in_bag(&self, complex_resource_type: &ComplexResourceType) -> bool {
        let tmp = self.complex.get(complex_resource_type);
        match tmp {
            None => false,
            Some(qty) => {
                qty >= &1
            }
        }
    }

    pub fn is_in_bag(&self, resource_type: &ResourceType) -> bool {
        let mut check: bool = false;

        if resource_type.is_carbon() {
            check = self.is_basic_in_bag(&BasicResourceType::Carbon);
        }
        else if resource_type.is_hydrogen() {
            check = self.is_basic_in_bag(&BasicResourceType::Hydrogen);
        }
        else if resource_type.is_silicon() {
            check = self.is_basic_in_bag(&BasicResourceType::Silicon);
        }
        else if resource_type.is_oxygen() {
            check = self.is_basic_in_bag(&BasicResourceType::Oxygen);
        }

        else if resource_type.is_diamond() {
            check = self.is_complex_in_bag(&ComplexResourceType::Diamond);
        }
        else if resource_type.is_dolphin() {
            check = self.is_complex_in_bag(&ComplexResourceType::Dolphin);
        }
        else if resource_type.is_aipartner() {
            check = self.is_complex_in_bag(&ComplexResourceType::AIPartner);
        }
        else if resource_type.is_robot() {
            check = self.is_complex_in_bag(&ComplexResourceType::Robot);
        }
        else if resource_type.is_water() {
            check = self.is_complex_in_bag(&ComplexResourceType::Water);
        }
        else if resource_type.is_life() {
            check = self.is_complex_in_bag(&ComplexResourceType::Life);
        }

        check
    }

    /*  pub fn get_all_resources_as_strings(&self) -> Vec<String> {//aggiunta in caso per non rendere complex e basic public
        let mut list = Vec::new();
        for (res_type, count) in &self.basic {
            for _ in 0..*count {
                list.push(format!("{:?}", res_type));
            }
        }
        for (res_type, count) in &self.complex {
            for _ in 0..*count {
                list.push(format!("{:?}", res_type));
            }
        }
        list
    }*/
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct BagType {
    complex: HashMap<ComplexResourceType, Vec<ComplexResource>>,
    basic: HashMap<BasicResourceType, Vec<BasicResource>>,
}

#[allow(dead_code)]
trait ResourceCustom<T> {
    fn get_custom_type(&self) -> T;
}

impl ResourceCustom<BasicResourceType> for BasicResource {
    fn get_custom_type(&self) -> BasicResourceType {
        self.get_type()
    }
}

impl ResourceCustom<ComplexResourceType> for ComplexResource {
    fn get_custom_type(&self) -> ComplexResourceType {
        self.get_type()
    }
}

impl ResourceCustom<ResourceType> for GenericResource {
    fn get_custom_type(&self) -> ResourceType {
        self.get_type()
    }
}

#[allow(dead_code)]
impl BagType {
    pub fn new() -> BagType {
        BagType {
            complex: HashMap::new(),
            basic: HashMap::new(),
        }
    }

    pub fn to_dummy(&self) -> DummyBag {
        let mut basic_map = HashMap::new();
        for (key, value) in &self.basic {
            basic_map.insert(key.clone(), value.len());
        }

        let mut complex_map = HashMap::new();
        for (key, value) in &self.complex {
            complex_map.insert(key.clone(), value.len());
        }

        DummyBag::new(basic_map, complex_map)
    }

    fn add_to_bag<U: Hash + Eq, T: ResourceCustom<U>>(resource: T, map: &mut HashMap<U, Vec<T>>) {
        let key = resource.get_custom_type();

        map.entry(key).or_insert_with(Vec::new).push(resource);
    }

    fn remove_from_bag<U: Hash + Eq, T: ResourceCustom<U> + Hash + Eq>(
        resource_type: U,
        map: &mut HashMap<U, Vec<T>>,
    ) -> Option<T> {
        match map.get_mut(&resource_type) {
            Some(set) => set.pop(),
            None => None,
        }
    }

    pub fn add_basic_resource(&mut self, basic_res: BasicResource) {
        Self::add_to_bag(basic_res, &mut self.basic);
    }

    pub fn add_complex_resource(&mut self, complex_res: ComplexResource) {
        Self::add_to_bag(complex_res, &mut self.complex);
    }

    pub fn remove_basic_resource(&mut self, basic_res: BasicResourceType) -> Option<BasicResource> {
        Self::remove_from_bag(basic_res, &mut self.basic)
    }
    pub fn remove_complex_resource(
        &mut self,
        complex_res: ComplexResourceType,
    ) -> Option<ComplexResource> {
        Self::remove_from_bag(complex_res, &mut self.complex)
    }

    fn check_same_type_ingredients<T: Hash + Eq, U: ResourceCustom<T>>(
        res1: Option<U>,
        res2: Option<U>,
        map: &mut HashMap<T, Vec<U>>,
    ) -> Option<(U, U)> {
        match (res1, res2) {
            (Some(u1), Some(u2)) => Some((u1, u2)),
            (Some(u1), None) => {
                Self::add_to_bag(u1, map);
                None
            }
            (None, Some(u2)) => {
                Self::add_to_bag(u2, map);
                None
            }
            (None, None) => None,
        }
    }

    fn check_diff_type_ingredients(
        res1: Option<BasicResource>,
        res2: Option<ComplexResource>,
        basic_map: &mut HashMap<BasicResourceType, Vec<BasicResource>>,
        complex_map: &mut HashMap<ComplexResourceType, Vec<ComplexResource>>,
    ) -> Option<(BasicResource, ComplexResource)> {
        match (res1, res2) {
            (Some(u1), Some(u2)) => Some((u1, u2)),
            (Some(u1), None) => {
                Self::add_to_bag(u1, basic_map);
                None
            }
            (None, Some(u2)) => {
                Self::add_to_bag(u2, complex_map);
                None
            }
            (None, None) => None,
        }
    }

    pub fn get_diff_type_ingredients(
        &mut self,
        type_res1: BasicResourceType,
        type_res2: ComplexResourceType,
    ) -> Option<(BasicResource, ComplexResource)> {
        let lhs = self.remove_basic_resource(type_res1);
        let rhs = self.remove_complex_resource(type_res2);
        Self::check_diff_type_ingredients(lhs, rhs, &mut self.basic, &mut self.complex)
    }

    pub fn get_basic_ingredients(
        &mut self,
        type_res1: BasicResourceType,
        type_res2: BasicResourceType,
    ) -> Option<(BasicResource, BasicResource)> {
        let lhs = self.remove_basic_resource(type_res1);
        let rhs = self.remove_basic_resource(type_res2);
        Self::check_same_type_ingredients(lhs, rhs, &mut self.basic)
    }
    pub fn get_complex_ingredients(
        &mut self,
        type_res1: ComplexResourceType,
        type_res2: ComplexResourceType,
    ) -> Option<(ComplexResource, ComplexResource)> {
        let lhs = self.remove_complex_resource(type_res1);
        let rhs = self.remove_complex_resource(type_res2);
        Self::check_same_type_ingredients(lhs, rhs, &mut self.complex)
    }
}
