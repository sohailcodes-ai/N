use std::collections::HashMap;

use crate::agent::AgentInventory;
use crate::market::MarketListing;
use crate::resources::{
    FOOD_PRICE_MAX, FOOD_PRICE_MIN, FOOD_PURCHASE_HUNGER_THRESHOLD, FOOD_PURCHASE_TARGET_INVENTORY,
    SCARCITY_PRICE_MULTIPLIER, WATER_PURCHASE_TARGET_INVENTORY, WATER_PURCHASE_THIRST_THRESHOLD,
};

pub fn agent_wants_food(agent_hunger: f64, food_in_inventory: u32) -> bool {
    agent_hunger > FOOD_PURCHASE_HUNGER_THRESHOLD
        && food_in_inventory < FOOD_PURCHASE_TARGET_INVENTORY
}

pub fn compute_desired_food_quantity(agent_hunger: f64, food_in_inventory: u32) -> u32 {
    if agent_hunger <= FOOD_PURCHASE_HUNGER_THRESHOLD {
        return 0;
    }
    let target = FOOD_PURCHASE_TARGET_INVENTORY;
    if food_in_inventory >= target {
        return 0;
    }
    let deficit = target - food_in_inventory;
    let urgency = ((agent_hunger - FOOD_PURCHASE_HUNGER_THRESHOLD) * 5.0).min(1.0) as u32;
    (deficit).max(urgency).min(3)
}

pub fn agent_wants_water(agent_thirst: f64, water_in_inventory: u32) -> bool {
    agent_thirst > WATER_PURCHASE_THIRST_THRESHOLD
        && water_in_inventory < WATER_PURCHASE_TARGET_INVENTORY
}

pub fn compute_desired_water_quantity(agent_thirst: f64, water_in_inventory: u32) -> u32 {
    if agent_thirst <= WATER_PURCHASE_THIRST_THRESHOLD {
        return 0;
    }
    let target = WATER_PURCHASE_TARGET_INVENTORY;
    if water_in_inventory >= target {
        return 0;
    }
    let deficit = target - water_in_inventory;
    let urgency = ((agent_thirst - WATER_PURCHASE_THIRST_THRESHOLD) * 5.0).min(1.0) as u32;
    (deficit).max(urgency).min(3)
}

pub fn find_food_listing(listings: &[MarketListing], max_price: f64) -> Option<(usize, u32, f64)> {
    for (i, listing) in listings.iter().enumerate() {
        if listing.resource == "food" && listing.quantity > 0 && listing.price <= max_price {
            return Some((i, listing.quantity, listing.price));
        }
    }
    None
}

pub fn find_water_listing(listings: &[MarketListing], max_price: f64) -> Option<(usize, u32, f64)> {
    for (i, listing) in listings.iter().enumerate() {
        if listing.resource == "water" && listing.quantity > 0 && listing.price <= max_price {
            return Some((i, listing.quantity, listing.price));
        }
    }
    None
}

pub fn execute_purchase(
    listings: &mut Vec<MarketListing>,
    agent_inventory: &mut AgentInventory,
    agent_money: &mut f64,
    _buyer_id: &str,
    seller_id: &str,
    resource: &str,
    quantity: u32,
    price: f64,
) -> bool {
    let total_cost = price * quantity as f64;
    if *agent_money < total_cost {
        return false;
    }

    if let Some(listing) = listings
        .iter_mut()
        .find(|l| l.seller_id == seller_id && l.resource == resource && l.quantity >= quantity)
    {
        *agent_money -= total_cost;
        listing.quantity -= quantity;
        let (ok, _) = agent_inventory.add_resource(resource, quantity);
        ok
    } else {
        false
    }
}

pub fn adjust_price(
    current_price: f64,
    _sold_quantity: u32,
    remaining_supply: u32,
    demand_pressure: f64,
) -> f64 {
    let supply_pressure = if remaining_supply > 100 {
        1.0
    } else if remaining_supply > 50 {
        0.5
    } else if remaining_supply > 10 {
        0.1
    } else {
        0.0
    };

    let demand_factor = demand_pressure * 0.1;
    let supply_factor = supply_pressure * 0.05;

    let adjustment = demand_factor - supply_factor;
    let new_price = current_price + adjustment;

    new_price.clamp(FOOD_PRICE_MIN, FOOD_PRICE_MAX)
}

pub fn adjust_price_with_scarcity(
    current_price: f64,
    remaining_supply: u32,
    demand_pressure: f64,
    scarcity_threshold: u32,
    price_min: f64,
    price_max: f64,
) -> f64 {
    let scarcity_factor = if remaining_supply < scarcity_threshold {
        let scarcity_ratio = 1.0 - (remaining_supply as f64 / scarcity_threshold as f64);
        SCARCITY_PRICE_MULTIPLIER * scarcity_ratio
    } else {
        0.0
    };

    let supply_pressure = if remaining_supply > 100 {
        1.0
    } else if remaining_supply > 50 {
        0.5
    } else if remaining_supply > 10 {
        0.1
    } else {
        0.0
    };

    let demand_factor = demand_pressure * 0.1;
    let supply_factor = supply_pressure * 0.05;

    let adjustment = demand_factor - supply_factor + scarcity_factor;
    let new_price = current_price + adjustment;

    new_price.clamp(price_min, price_max)
}

pub fn compute_total_money(
    agents_money: &HashMap<String, f64>,
    companies_cash: &HashMap<String, f64>,
) -> f64 {
    let agent_total: f64 = agents_money.values().sum();
    let company_total: f64 = companies_cash.values().sum();
    agent_total + company_total
}
