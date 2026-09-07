use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Zone {
    Residential,
    Commercial,
    Industrial,
    Public,
}

impl Zone {
    pub fn name(&self) -> &str {
        match self {
            Zone::Residential => "residential",
            Zone::Commercial => "commercial",
            Zone::Industrial => "industrial",
            Zone::Public => "public",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuildingType {
    House,
    Apartment,
    Factory,
    Farm,
    Shop,
    Office,
    Warehouse,
    WaterPlant,
}

impl BuildingType {
    pub fn name(&self) -> &str {
        match self {
            BuildingType::House => "house",
            BuildingType::Apartment => "apartment",
            BuildingType::Factory => "factory",
            BuildingType::Farm => "farm",
            BuildingType::Shop => "shop",
            BuildingType::Office => "office",
            BuildingType::Warehouse => "warehouse",
            BuildingType::WaterPlant => "water_plant",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "house" => Some(BuildingType::House),
            "apartment" => Some(BuildingType::Apartment),
            "factory" => Some(BuildingType::Factory),
            "farm" => Some(BuildingType::Farm),
            "shop" => Some(BuildingType::Shop),
            "office" => Some(BuildingType::Office),
            "warehouse" => Some(BuildingType::Warehouse),
            "water_plant" => Some(BuildingType::WaterPlant),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstructionState {
    Planned,
    Funded,
    UnderConstruction,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropertyOwnership {
    Agent(String),
    Company(String),
    Unowned,
}

impl PropertyOwnership {
    pub fn owner_id(&self) -> Option<&str> {
        match self {
            PropertyOwnership::Agent(id) => Some(id),
            PropertyOwnership::Company(id) => Some(id),
            PropertyOwnership::Unowned => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parcel {
    pub id: String,
    pub district_id: String,
    pub zone: Zone,
    pub area: f64,
    pub owner: PropertyOwnership,
    pub current_value: f64,
    pub occupied: bool,
    pub building_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub parcel_id: String,
    pub owner: PropertyOwnership,
    pub building_type: BuildingType,
    pub value: f64,
    pub purchase_price: f64,
    pub occupancy: u32,
    pub building_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub parcel_id: String,
    pub building_type: BuildingType,
    pub owner: PropertyOwnership,
    pub operator: Option<String>,
    pub construction_state: ConstructionState,
    pub construction_progress: f64,
    pub capacity: u32,
    pub operational: bool,
    pub construction_cost: f64,
    pub maintenance_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionProject {
    pub id: String,
    pub owner: String,
    pub parcel_id: String,
    pub target_building: BuildingType,
    pub required_resources: HashMap<String, u32>,
    pub reserved_resources: HashMap<String, u32>,
    pub labor_required: u32,
    pub total_duration: u64,
    pub progress: u64,
    pub total_cost: f64,
    pub paid_cost: f64,
    pub status: ConstructionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionRecipe {
    pub building_type: BuildingType,
    pub money_cost: f64,
    pub resource_cost: HashMap<String, u32>,
    pub labor_hours: u32,
    pub duration_ticks: u64,
    pub capacity: u32,
    pub maintenance_cost: f64,
}

impl ConstructionRecipe {
    pub fn default_recipes() -> HashMap<String, ConstructionRecipe> {
        let mut recipes = HashMap::new();

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 5);
        recipes.insert(
            "house".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::House,
                money_cost: 500.0,
                resource_cost: r,
                labor_hours: 8,
                duration_ticks: 1440,
                capacity: 4,
                maintenance_cost: 5.0,
            },
        );

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 10);
        recipes.insert(
            "apartment".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::Apartment,
                money_cost: 1000.0,
                resource_cost: r,
                labor_hours: 16,
                duration_ticks: 2880,
                capacity: 8,
                maintenance_cost: 10.0,
            },
        );

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 20);
        recipes.insert(
            "factory".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::Factory,
                money_cost: 2000.0,
                resource_cost: r,
                labor_hours: 24,
                duration_ticks: 4320,
                capacity: 10,
                maintenance_cost: 20.0,
            },
        );

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 8);
        recipes.insert(
            "farm".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::Farm,
                money_cost: 800.0,
                resource_cost: r,
                labor_hours: 12,
                duration_ticks: 2160,
                capacity: 6,
                maintenance_cost: 8.0,
            },
        );

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 6);
        recipes.insert(
            "shop".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::Shop,
                money_cost: 600.0,
                resource_cost: r,
                labor_hours: 10,
                duration_ticks: 1440,
                capacity: 4,
                maintenance_cost: 6.0,
            },
        );

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 12);
        recipes.insert(
            "office".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::Office,
                money_cost: 1200.0,
                resource_cost: r,
                labor_hours: 14,
                duration_ticks: 2880,
                capacity: 8,
                maintenance_cost: 12.0,
            },
        );

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 15);
        recipes.insert(
            "warehouse".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::Warehouse,
                money_cost: 900.0,
                resource_cost: r,
                labor_hours: 12,
                duration_ticks: 2160,
                capacity: 100,
                maintenance_cost: 8.0,
            },
        );

        let mut r = HashMap::new();
        r.insert("raw_food".to_string(), 10);
        r.insert("raw_water".to_string(), 5);
        recipes.insert(
            "water_plant".to_string(),
            ConstructionRecipe {
                building_type: BuildingType::WaterPlant,
                money_cost: 1500.0,
                resource_cost: r,
                labor_hours: 20,
                duration_ticks: 4320,
                capacity: 5,
                maintenance_cost: 15.0,
            },
        );

        recipes
    }

    pub fn recipe_key(building_type: &BuildingType) -> String {
        match building_type {
            BuildingType::House => "house",
            BuildingType::Apartment => "apartment",
            BuildingType::Factory => "factory",
            BuildingType::Farm => "farm",
            BuildingType::Shop => "shop",
            BuildingType::Office => "office",
            BuildingType::Warehouse => "warehouse",
            BuildingType::WaterPlant => "water_plant",
        }
        .to_string()
    }
}

pub fn zone_compatible_building(zone: &Zone, building_type: &BuildingType) -> bool {
    match zone {
        Zone::Residential => {
            matches!(
                building_type,
                BuildingType::House | BuildingType::Apartment
            )
        }
        Zone::Commercial => matches!(building_type, BuildingType::Shop | BuildingType::Office),
        Zone::Industrial => matches!(
            building_type,
            BuildingType::Factory
                | BuildingType::Warehouse
                | BuildingType::Farm
                | BuildingType::WaterPlant
        ),
        Zone::Public => true,
    }
}

pub fn default_parcel_value(zone: &Zone) -> f64 {
    match zone {
        Zone::Residential => 100.0,
        Zone::Commercial => 150.0,
        Zone::Industrial => 80.0,
        Zone::Public => 50.0,
    }
}

pub fn default_building_capacity(building_type: &BuildingType) -> u32 {
    match building_type {
        BuildingType::House => 4,
        BuildingType::Apartment => 8,
        BuildingType::Factory => 10,
        BuildingType::Farm => 6,
        BuildingType::Shop => 4,
        BuildingType::Office => 8,
        BuildingType::Warehouse => 100,
        BuildingType::WaterPlant => 5,
    }
}
