use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictLayout {
    pub name: String,
    pub area: f64,
    pub residents: u32,
    pub businesses: u32,
    pub average_income: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityLayout {
    pub districts: HashMap<String, DistrictLayout>,
    pub road_network: Vec<(String, String)>,
}
