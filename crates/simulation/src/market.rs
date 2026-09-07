use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketListing {
    pub seller_id: String,
    pub resource: String,
    pub quantity: u32,
    pub price: f64,
    pub original_quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTransaction {
    pub buyer_id: String,
    pub seller_id: String,
    pub resource: String,
    pub quantity: u32,
    pub price: f64,
    pub total_cost: f64,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    pub prices: HashMap<String, f64>,
    pub buy_orders: HashMap<String, f64>,
    pub sell_orders: HashMap<String, f64>,
    pub listings: Vec<MarketListing>,
    pub transactions: Vec<MarketTransaction>,
    pub previous_demand: HashMap<String, f64>,
    pub previous_supply: HashMap<String, f64>,
}
