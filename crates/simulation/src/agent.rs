use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::resources::{AgentNeeds, AgentStatus, DEFAULT_FOOD_CAPACITY, DEFAULT_WATER_CAPACITY};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInventory {
    pub resources: HashMap<String, u32>,
    pub capacity: HashMap<String, u32>,
}

impl AgentInventory {
    pub fn new(capacities: HashMap<String, u32>) -> Self {
        Self {
            resources: HashMap::new(),
            capacity: capacities,
        }
    }

    pub fn add_resource(&mut self, resource: &str, amount: u32) -> (bool, u32) {
        if amount == 0 {
            return (true, 0);
        }
        let current = self.resources.get(resource).copied().unwrap_or(0);
        let cap = self.capacity.get(resource).copied().unwrap_or(u32::MAX);
        let new_total = current.saturating_add(amount);
        if new_total > cap {
            let actual = cap.saturating_sub(current);
            self.resources.insert(resource.to_string(), cap);
            (actual > 0, actual)
        } else {
            self.resources.insert(resource.to_string(), new_total);
            (true, amount)
        }
    }

    pub fn remove_resource(&mut self, resource: &str, amount: u32) -> (bool, u32) {
        if amount == 0 {
            return (true, 0);
        }
        let current = self.resources.get(resource).copied().unwrap_or(0);
        if current < amount {
            self.resources.remove(resource);
            return (false, current);
        }
        let new_val = current - amount;
        if new_val == 0 {
            self.resources.remove(resource);
        } else {
            self.resources.insert(resource.to_string(), new_val);
        }
        (true, amount)
    }

    pub fn has_resource(&self, resource: &str) -> bool {
        self.resources.get(resource).copied().unwrap_or(0) > 0
    }

    pub fn resource_quantity(&self, resource: &str) -> u32 {
        self.resources.get(resource).copied().unwrap_or(0)
    }
}

pub fn default_inventory() -> AgentInventory {
    let mut caps = HashMap::new();
    caps.insert("food".to_string(), DEFAULT_FOOD_CAPACITY);
    caps.insert("water".to_string(), DEFAULT_WATER_CAPACITY);
    AgentInventory::new(caps)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub name: String,
    pub age: u8,
    pub skills: HashMap<String, f64>,
    pub money: f64,
    pub inventory: AgentInventory,
    pub employer: Option<String>,
    pub location: String,
    pub status: AgentStatus,
    pub needs: AgentNeeds,
    pub experience: HashMap<String, f64>,
    pub memories: Vec<String>,
    pub goals: Vec<String>,
    pub housing_id: Option<String>,
}
