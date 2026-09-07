use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::resources::{
    BASE_PRODUCTION_RATE, FOOD_OUTPUT_PER_CYCLE, MIN_WORKERS_FOR_PRODUCTION,
    PRODUCTION_COOLDOWN_TICKS, RAW_FOOD_INPUT_PER_CYCLE, RAW_WATER_INPUT_PER_CYCLE,
    WATER_OUTPUT_PER_CYCLE,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInput {
    pub resource: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOutput {
    pub resource: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub inputs: Vec<RecipeInput>,
    pub outputs: Vec<RecipeOutput>,
    pub labor_required: usize,
    pub cooldown_ticks: u32,
}

impl Recipe {
    pub fn food_production() -> Self {
        Self {
            name: "food_production".to_string(),
            inputs: vec![RecipeInput {
                resource: "raw_food".to_string(),
                quantity: RAW_FOOD_INPUT_PER_CYCLE,
            }],
            outputs: vec![RecipeOutput {
                resource: "food".to_string(),
                quantity: FOOD_OUTPUT_PER_CYCLE,
            }],
            labor_required: MIN_WORKERS_FOR_PRODUCTION,
            cooldown_ticks: PRODUCTION_COOLDOWN_TICKS,
        }
    }

    pub fn water_production() -> Self {
        Self {
            name: "water_production".to_string(),
            inputs: vec![RecipeInput {
                resource: "raw_water".to_string(),
                quantity: RAW_WATER_INPUT_PER_CYCLE,
            }],
            outputs: vec![RecipeOutput {
                resource: "water".to_string(),
                quantity: WATER_OUTPUT_PER_CYCLE,
            }],
            labor_required: MIN_WORKERS_FOR_PRODUCTION,
            cooldown_ticks: PRODUCTION_COOLDOWN_TICKS,
        }
    }

    pub fn default_recipes() -> HashMap<String, Recipe> {
        let mut recipes = HashMap::new();
        let food_recipe = Recipe::food_production();
        let water_recipe = Recipe::water_production();
        recipes.insert(food_recipe.name.clone(), food_recipe);
        recipes.insert(water_recipe.name.clone(), water_recipe);
        recipes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOutput {
    pub resource: String,
    pub quantity: u32,
}

pub fn compute_productivity_multiplier(average_skill: f64) -> f64 {
    let base = BASE_PRODUCTION_RATE;
    let skill_bonus = average_skill * 0.3;
    (base + skill_bonus).min(1.5)
}

pub fn can_produce(
    recipe: &Recipe,
    inventory: &HashMap<String, u32>,
    worker_count: usize,
    cooldown: u32,
) -> bool {
    if cooldown > 0 {
        return false;
    }
    if worker_count < recipe.labor_required {
        return false;
    }
    for input in &recipe.inputs {
        let available = inventory.get(&input.resource).copied().unwrap_or(0);
        if available < input.quantity {
            return false;
        }
    }
    true
}

pub fn execute_production(
    recipe: &Recipe,
    inventory: &mut HashMap<String, u32>,
    average_skill: f64,
) -> Result<Vec<ProductionOutput>, String> {
    let multiplier = compute_productivity_multiplier(average_skill);

    for input in &recipe.inputs {
        let available = inventory.get(&input.resource).copied().unwrap_or(0);
        if available < input.quantity {
            return Err(format!(
                "Insufficient {}: have {} need {}",
                input.resource, available, input.quantity
            ));
        }
        let new_val = available - input.quantity;
        if new_val == 0 {
            inventory.remove(&input.resource);
        } else {
            inventory.insert(input.resource.clone(), new_val);
        }
    }

    let mut outputs = Vec::new();
    for output in &recipe.outputs {
        let produced = (output.quantity as f64 * multiplier) as u32;
        let current = inventory.get(&output.resource).copied().unwrap_or(0);
        let new_val = current + produced;
        inventory.insert(output.resource.clone(), new_val);
        outputs.push(ProductionOutput {
            resource: output.resource.clone(),
            quantity: produced,
        });
    }

    Ok(outputs)
}

pub fn can_produce_partial(
    recipe: &Recipe,
    inventory: &HashMap<String, u32>,
    worker_count: usize,
    cooldown: u32,
) -> bool {
    if cooldown > 0 || worker_count < recipe.labor_required {
        return false;
    }
    for input in &recipe.inputs {
        let available = inventory.get(&input.resource).copied().unwrap_or(0);
        if available > 0 {
            return true;
        }
    }
    false
}

pub fn execute_partial_production(
    recipe: &Recipe,
    inventory: &mut HashMap<String, u32>,
    average_skill: f64,
) -> Vec<ProductionOutput> {
    let multiplier = compute_productivity_multiplier(average_skill);

    let mut min_ratio: f64 = 1.0;
    for input in &recipe.inputs {
        let available = inventory.get(&input.resource).copied().unwrap_or(0);
        if input.quantity > 0 {
            let ratio = available as f64 / input.quantity as f64;
            if ratio < min_ratio {
                min_ratio = ratio;
            }
        }
    }

    if min_ratio <= 0.0 {
        return Vec::new();
    }

    for input in &recipe.inputs {
        let available = inventory.get(&input.resource).copied().unwrap_or(0);
        let consumed = ((input.quantity as f64 * min_ratio) as u32).min(available);
        let new_val = available - consumed;
        if new_val == 0 {
            inventory.remove(&input.resource);
        } else {
            inventory.insert(input.resource.clone(), new_val);
        }
    }

    let mut outputs = Vec::new();
    for output in &recipe.outputs {
        let produced = ((output.quantity as f64 * multiplier * min_ratio) as u32).max(1);
        let current = inventory.get(&output.resource).copied().unwrap_or(0);
        let new_val = current + produced;
        inventory.insert(output.resource.clone(), new_val);
        outputs.push(ProductionOutput {
            resource: output.resource.clone(),
            quantity: produced,
        });
    }

    outputs
}
