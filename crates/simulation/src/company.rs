use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanyStrategy {
    Growth,
    Stability,
    ProfitMaximization,
    Innovation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanyType {
    FoodProducer,
    WaterProducer,
    Retailer,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductOffering {
    pub name: String,
    pub price: f64,
    pub supply: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyState {
    pub id: String,
    pub name: String,
    pub founder: String,
    pub owners: HashMap<String, f64>,
    pub employees: HashMap<String, u32>,
    pub cash: f64,
    pub revenue: f64,
    pub expenses: f64,
    pub assets: f64,
    pub liabilities: f64,
    pub products: Vec<ProductOffering>,
    pub location: String,
    pub strategy: CompanyStrategy,
    pub goals: Vec<String>,
    pub company_type: CompanyType,
    pub inventory: HashMap<String, u32>,
    pub recipe_name: Option<String>,
    pub production_cooldown: u32,
}
