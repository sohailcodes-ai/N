use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictLayout {
    pub name: String,
    pub area: f64,
    pub residents: u32,
    pub businesses: u32,
    pub average_income: f64,
    pub parcel_ids: Vec<String>,
    pub residential_capacity: u32,
    pub commercial_capacity: u32,
    pub industrial_capacity: u32,
    pub active_construction: u32,
    pub completed_construction: u32,
    pub total_property_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityLayout {
    pub districts: HashMap<String, DistrictLayout>,
    pub road_network: Vec<(String, String)>,
}
