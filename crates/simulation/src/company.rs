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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanyStatus {
    Operating,
    Understaffed,
    Overstaffed,
    Suspended,
    Insolvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOffer {
    pub company_id: String,
    pub role: String,
    pub wage: f64,
    pub required_skill: String,
    pub min_skill_level: f64,
    pub openings: u32,
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
    pub status: CompanyStatus,
    pub desired_workforce: usize,
    pub min_workforce: usize,
    pub max_workforce: usize,
    pub required_workers: usize,
    pub wage: f64,
    pub job_openings: Vec<JobOffer>,
    pub profit_loss: f64,
    pub cumulative_profit: f64,
    pub loss_streak: u32,
    pub last_growth_tick: u64,
    pub last_hire_tick: u64,
    pub active: bool,
    pub closed_at_tick: Option<u64>,
    pub building_ids: Vec<String>,
    pub property_ids: Vec<String>,
    pub total_capacity: u32,
    pub construction_project_ids: Vec<String>,
    pub last_expansion_tick: u64,
}
