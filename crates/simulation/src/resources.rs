use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const HUNGER_RATE_PER_HOUR: f64 = 0.01;
pub const THIRST_RATE_PER_HOUR: f64 = 0.02;
pub const FATIGUE_RATE_PER_HOUR_AWAKE: f64 = 0.008;
pub const FATIGUE_RECOVERY_RATE_PER_HOUR_SLEEPING: f64 = 0.08;
pub const FATIGUE_RECOVERY_RATE_PER_HOUR_RESTING: f64 = 0.04;
pub const HUNGER_REDUCTION_PER_FOOD: f64 = 0.3;
pub const THIRST_REDUCTION_PER_WATER: f64 = 0.35;
pub const CRITICAL_THIRST_THRESHOLD: f64 = 0.6;
pub const CRITICAL_HUNGER_THRESHOLD: f64 = 0.6;
pub const SLEEP_THRESHOLD: f64 = 0.7;
pub const REST_THRESHOLD: f64 = 0.5;
pub const SKILL_XP_BASE_RATE: f64 = 0.005;
pub const DEFAULT_FOOD_CAPACITY: u32 = 10;
pub const DEFAULT_WATER_CAPACITY: u32 = 5;
pub const NEEDS_CHANGE_EVENT_THRESHOLD: f64 = 0.05;

pub const FOOD_BASE_PRICE: f64 = 2.0;
pub const FOOD_PRICE_MIN: f64 = 0.5;
pub const FOOD_PRICE_MAX: f64 = 10.0;
pub const PRICE_ADJUSTMENT_RATE: f64 = 0.05;
pub const DEMAND_PRESSURE_FACTOR: f64 = 0.1;
pub const SUPPLY_PRESSURE_FACTOR: f64 = 0.05;

pub const RAW_FOOD_INPUT_PER_CYCLE: u32 = 10;
pub const FOOD_OUTPUT_PER_CYCLE: u32 = 10;
pub const PRODUCTION_COOLDOWN_TICKS: u32 = 1;
pub const BASE_PRODUCTION_RATE: f64 = 1.0;
pub const MIN_WORKERS_FOR_PRODUCTION: usize = 1;
pub const WAGE_PER_TICK: f64 = 10.0;
pub const FOOD_PURCHASE_HUNGER_THRESHOLD: f64 = 0.4;
pub const FOOD_PURCHASE_TARGET_INVENTORY: u32 = 5;

pub const WATER_BASE_PRICE: f64 = 0.5;
pub const WATER_PRICE_MIN: f64 = 0.1;
pub const WATER_PRICE_MAX: f64 = 5.0;
pub const WATER_PURCHASE_THIRST_THRESHOLD: f64 = 0.4;
pub const WATER_PURCHASE_TARGET_INVENTORY: u32 = 3;
pub const RAW_WATER_INPUT_PER_CYCLE: u32 = 10;
pub const WATER_OUTPUT_PER_CYCLE: u32 = 10;

pub const SCARCITY_THRESHOLD_FOOD: u32 = 20;
pub const SCARCITY_THRESHOLD_WATER: u32 = 15;
pub const SCARCITY_PRICE_MULTIPLIER: f64 = 1.5;
pub const REGENERATION_RATE_FOOD: u32 = 5;
pub const REGENERATION_RATE_WATER: u32 = 3;
pub const RESOURCE_RESERVE_FOOD: u32 = 200;
pub const RESOURCE_RESERVE_WATER: u32 = 150;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNeeds {
    pub hunger: f64,
    pub thirst: f64,
    pub fatigue: f64,
}

impl AgentNeeds {
    pub fn new() -> Self {
        Self {
            hunger: 0.0,
            thirst: 0.0,
            fatigue: 0.0,
        }
    }

    pub fn clamp(&mut self) {
        self.hunger = self.hunger.clamp(0.0, 1.0);
        self.thirst = self.thirst.clamp(0.0, 1.0);
        self.fatigue = self.fatigue.clamp(0.0, 1.0);
    }

    pub fn needs_drinking(&self) -> bool {
        self.thirst > CRITICAL_THIRST_THRESHOLD
    }

    pub fn needs_eating(&self) -> bool {
        self.hunger > CRITICAL_HUNGER_THRESHOLD
    }

    pub fn needs_sleeping(&self) -> bool {
        self.fatigue > SLEEP_THRESHOLD
    }

    pub fn needs_resting(&self) -> bool {
        self.fatigue > REST_THRESHOLD
    }

    pub fn is_satiated(&self) -> bool {
        self.hunger < 0.3 && self.thirst < 0.3 && self.fatigue < 0.3
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Working,
    Resting,
    Sleeping,
    Eating,
    Drinking,
    Idle,
    Unemployed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Water,
    RawFood,
}

impl ResourceType {
    pub fn name(&self) -> &'static str {
        match self {
            ResourceType::Food => "food",
            ResourceType::Water => "water",
            ResourceType::RawFood => "raw_food",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "food" => Some(ResourceType::Food),
            "water" => Some(ResourceType::Water),
            "raw_food" => Some(ResourceType::RawFood),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInventory {
    pub quantities: HashMap<String, u32>,
}

impl ResourceInventory {
    pub fn new() -> Self {
        Self {
            quantities: HashMap::new(),
        }
    }

    pub fn with_capacity(resource: ResourceType, amount: u32) -> Self {
        let mut inv = Self::new();
        inv.quantities.insert(resource.name().to_string(), amount);
        inv
    }

    pub fn quantity(&self, resource: ResourceType) -> u32 {
        self.quantities.get(resource.name()).copied().unwrap_or(0)
    }

    pub fn quantity_str(&self, resource: &str) -> u32 {
        self.quantities.get(resource).copied().unwrap_or(0)
    }

    pub fn has(&self, resource: ResourceType) -> bool {
        self.quantity(resource) > 0
    }

    pub fn has_str(&self, resource: &str) -> bool {
        self.quantity_str(resource) > 0
    }

    pub fn add(&mut self, resource: ResourceType, amount: u32) -> Result<u32, String> {
        if amount == 0 {
            return Ok(0);
        }
        let key = resource.name().to_string();
        let current = self.quantities.get(&key).copied().unwrap_or(0);
        let new_val = current.checked_add(amount).ok_or_else(|| {
            format!(
                "Inventory overflow for {}: {} + {}",
                resource.name(),
                current,
                amount
            )
        })?;
        self.quantities.insert(key, new_val);
        Ok(amount)
    }

    pub fn add_str(&mut self, resource: &str, amount: u32) -> Result<u32, String> {
        if amount == 0 {
            return Ok(0);
        }
        let current = self.quantities.get(resource).copied().unwrap_or(0);
        let new_val = current.checked_add(amount).ok_or_else(|| {
            format!(
                "Inventory overflow for {}: {} + {}",
                resource, current, amount
            )
        })?;
        self.quantities.insert(resource.to_string(), new_val);
        Ok(amount)
    }

    pub fn remove(&mut self, resource: ResourceType, amount: u32) -> Result<u32, String> {
        if amount == 0 {
            return Ok(0);
        }
        let key = resource.name().to_string();
        let current = self.quantities.get(&key).copied().unwrap_or(0);
        if current < amount {
            return Err(format!(
                "Insufficient {}: have {} need {}",
                resource.name(),
                current,
                amount
            ));
        }
        let new_val = current - amount;
        if new_val == 0 {
            self.quantities.remove(&key);
        } else {
            self.quantities.insert(key, new_val);
        }
        Ok(amount)
    }

    pub fn remove_str(&mut self, resource: &str, amount: u32) -> Result<u32, String> {
        if amount == 0 {
            return Ok(0);
        }
        let current = self.quantities.get(resource).copied().unwrap_or(0);
        if current < amount {
            return Err(format!(
                "Insufficient {}: have {} need {}",
                resource, current, amount
            ));
        }
        let new_val = current - amount;
        if new_val == 0 {
            self.quantities.remove(resource);
        } else {
            self.quantities.insert(resource.to_string(), new_val);
        }
        Ok(amount)
    }

    pub fn total(&self) -> u32 {
        self.quantities.values().sum()
    }
}
