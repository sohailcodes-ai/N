pub mod agent;
pub mod city;
pub mod clock;
pub mod company;
pub mod economy;
pub mod employment;
pub mod events;
pub mod market;
pub mod production;
pub mod property;
pub mod resources;

pub use agent::*;
pub use city::*;
pub use clock::*;
pub use company::*;
pub use economy::*;
pub use employment::*;
pub use events::*;
pub use market::*;
pub use production::*;
pub use property::*;
pub use resources::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    pub name: String,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub metadata: SimulationMetadata,
    pub agents: HashMap<String, AgentState>,
    pub companies: HashMap<String, CompanyState>,
    pub events: Vec<Event>,
    pub city: CityLayout,
    pub markets: MarketState,
    pub tick: u64,
    pub production_recipes: HashMap<String, Recipe>,
    pub labor_market: LaborMarketState,
    pub parcels: HashMap<String, Parcel>,
    pub buildings: HashMap<String, Building>,
    pub properties: HashMap<String, Property>,
    pub construction_recipes: HashMap<String, ConstructionRecipe>,
    pub construction_projects: HashMap<String, ConstructionProject>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub state: SimulationState,
    pub clock: SimulationClock,
}

impl World {
    pub fn initialize(seed: u64, num_agents: u32, num_companies: u32) -> Self {
        let clock = SimulationClock::new(seed, 1.0);
        let mut agents = HashMap::new();
        let mut companies = HashMap::new();
        let mut events = Vec::new();
        let mut labor_market = LaborMarketState::new();

        for i in 0..num_agents {
            let agent_id = format!("agent-{:04}", i);
            let mut skills = HashMap::new();
            let base = (seed as f64 * ((i + 1) as f64)).fract();
            skills.insert("productivity".to_string(), 0.3 + base * 0.4);
            skills.insert("social".to_string(), 0.2 + base * 0.5);

            let mut experience = HashMap::new();
            experience.insert("productivity".to_string(), 0.0);
            experience.insert("social".to_string(), 0.0);

            let agent = AgentState {
                id: agent_id.clone(),
                name: format!("Agent-{}", i),
                age: 20 + (i % 50) as u8,
                skills,
                money: 100.0,
                inventory: default_inventory(),
                employer: None,
                location: format!("district-{}", i % 5),
                status: AgentStatus::Active,
                needs: AgentNeeds::new(),
                experience,
                memories: Vec::new(),
                goals: vec!["survive".to_string(), "thrive".to_string()],
                housing_id: None,
            };

            events.push(Event {
                id: format!("evt-created-{:04}", i),
                tick: 0,
                event_type: EventType::AgentCreated,
                actor: None,
                cause: None,
                entities: vec![agent_id.clone()],
                state_snapshot: format!("Agent {} created", agent_id),
            });

            agents.insert(agent_id, agent);
        }

        for i in 0..num_companies {
            let company_id = format!("company-{:03}", i);
            let founder_id = format!("agent-{:04}", i);

            let mut owners = HashMap::new();
            owners.insert(founder_id.clone(), 1.0);

            let mut products = Vec::new();
            products.push(ProductOffering {
                name: format!("product-{:03}", i),
                price: 10.0 + (i as f64 * 5.0),
                supply: 100.0,
            });

            let company_type = if i < 3 {
                CompanyType::FoodProducer
            } else if i < 5 {
                CompanyType::WaterProducer
            } else {
                CompanyType::Service
            };

            let recipe_name = match company_type {
                CompanyType::FoodProducer => Some("food_production".to_string()),
                CompanyType::WaterProducer => Some("water_production".to_string()),
                _ => None,
            };

            let mut inventory = HashMap::new();
            match company_type {
                CompanyType::FoodProducer => {
                    inventory.insert("raw_food".to_string(), 500);
                    inventory.insert("food".to_string(), 50);
                }
                CompanyType::WaterProducer => {
                    inventory.insert("raw_water".to_string(), 400);
                    inventory.insert("water".to_string(), 40);
                }
                _ => {}
            }

            let (desired_workforce, required_workers, wage) = match company_type {
                CompanyType::FoodProducer => (3, REQUIRED_WORKERS_FOOD, DEFAULT_WAGE),
                CompanyType::WaterProducer => (3, REQUIRED_WORKERS_WATER, DEFAULT_WAGE),
                CompanyType::Service => (2, REQUIRED_WORKERS_SERVICE, DEFAULT_WAGE * 0.8),
                CompanyType::Retailer => (2, REQUIRED_WORKERS_RETAILER, DEFAULT_WAGE * 0.8),
            };

            let company = CompanyState {
                id: company_id.clone(),
                name: format!("Company-{}", i),
                founder: founder_id,
                owners,
                employees: HashMap::new(),
                cash: 1000.0,
                revenue: 0.0,
                expenses: 0.0,
                assets: 500.0,
                liabilities: 0.0,
                products,
                location: format!("district-{}", i % 5),
                strategy: CompanyStrategy::Growth,
                goals: vec!["expand".to_string()],
                company_type,
                inventory,
                recipe_name,
                production_cooldown: 0,
                status: CompanyStatus::Operating,
                desired_workforce,
                min_workforce: 1,
                max_workforce: 8,
                required_workers,
                wage,
                job_openings: Vec::new(),
                profit_loss: 0.0,
                cumulative_profit: 0.0,
                loss_streak: 0,
                last_growth_tick: 0,
                last_hire_tick: 0,
                active: true,
                closed_at_tick: None,
                building_ids: Vec::new(),
                property_ids: Vec::new(),
                total_capacity: 0,
                construction_project_ids: Vec::new(),
                last_expansion_tick: 0,
            };

            events.push(Event {
                id: format!("evt-founded-{:03}", i),
                tick: 0,
                event_type: EventType::CompanyFounded,
                actor: None,
                cause: None,
                entities: vec![company_id.clone()],
                state_snapshot: format!("Company {} founded", company_id),
            });

            companies.insert(company_id, company);
        }

        // Bootstrap imperfect employment: some companies get initial workers, some agents stay unemployed
        // This creates initial labor market dynamics
        let mut company_ids: Vec<String> = companies.keys().cloned().collect();
        company_ids.sort();
        for (idx, cid) in company_ids.iter().enumerate() {
            let initial_count = match idx {
                0 => 2,
                1 => 2,
                2 => 1,
                3 => 2,
                4 => 1,
                5 => 0,
                6 => 0,
                7 => 0,
                8 => 0,
                9 => 0,
                _ => 1,
            };

            let company = companies.get(cid).unwrap();
            let company_type = company.company_type.clone();
            let role = match company_type {
                CompanyType::FoodProducer => "Production Worker",
                CompanyType::WaterProducer => "Water Technician",
                CompanyType::Retailer => "Sales Associate",
                CompanyType::Service => "Service Worker",
            };
            let wage = company.wage;

            for j in 0..initial_count {
                let agent_idx = (idx * 5 + j) as u32;
                let agent_id = format!("agent-{:04}", agent_idx);
                if agents.contains_key(&agent_id) && !labor_market.is_agent_employed(&agent_id) {
                    let hired = labor_market.create_employment(
                        agent_id.clone(),
                        cid.clone(),
                        role.to_string(),
                        wage,
                        0,
                    );
                    if hired {
                        let agent = agents.get_mut(&agent_id).unwrap();
                        agent.employer = Some(cid.clone());

                        let company = companies.get_mut(cid).unwrap();
                        company.employees.insert(agent_id.clone(), 1);

                        events.push(Event {
                            id: format!("evt-init-hire-{}-{}-{}", cid, agent_id, 0),
                            tick: 0,
                            event_type: EventType::AgentHired,
                            actor: Some(cid.clone()),
                            cause: Some("genesis_bootstrap".to_string()),
                            entities: vec![agent_id, cid.clone()],
                            state_snapshot: format!("Initial hire at genesis"),
                        });
                    }
                }
            }
        }

        let mut districts = HashMap::new();
        for i in 0..5u32 {
            districts.insert(
                format!("district-{}", i),
                DistrictLayout {
                    name: format!("district-{}", i),
                    area: 100.0,
                    residents: num_agents / 5,
                    businesses: num_companies / 5,
                    average_income: 50.0,
                    parcel_ids: Vec::new(),
                    residential_capacity: 0,
                    commercial_capacity: 0,
                    industrial_capacity: 0,
                    active_construction: 0,
                    completed_construction: 0,
                    total_property_value: 0.0,
                },
            );
        }

        let mut parcels = HashMap::new();
        let mut buildings = HashMap::new();
        let mut properties = HashMap::new();
        let parcel_zones: Vec<(Zone, u32)> = vec![
            (Zone::Residential, 2),
            (Zone::Commercial, 1),
            (Zone::Industrial, 1),
        ];

        let mut parcel_counter = 0u32;
        for i in 0..5u32 {
            let district_id = format!("district-{}", i);
            let district = districts.get_mut(&district_id).unwrap();
            for &(ref zone, count) in &parcel_zones {
                for _ in 0..count {
                    let pid = format!("parcel-{:04}", parcel_counter);
                    let value = default_parcel_value(zone);
                    let parcel = Parcel {
                        id: pid.clone(),
                        district_id: district_id.clone(),
                        zone: zone.clone(),
                        area: 25.0,
                        owner: PropertyOwnership::Unowned,
                        current_value: value,
                        occupied: false,
                        building_id: None,
                    };
                    district.parcel_ids.push(pid.clone());
                    parcels.insert(pid, parcel);
                    parcel_counter += 1;
                }
            }
            district.total_property_value = district
                .parcel_ids
                .iter()
                .filter_map(|pid| parcels.get(pid))
                .map(|p| p.current_value)
                .sum();
        }

        let mut building_counter = 0u32;
        let district_ids: Vec<String> = districts.keys().cloned().collect();
        for district_id in &district_ids {
            let district = districts.get(district_id).unwrap();
            let parcel_ids: Vec<String> = district.parcel_ids.clone();
            for pid in &parcel_ids {
                let parcel = parcels.get(pid).unwrap();
                let (bt, _bkey) = match parcel.zone {
                    Zone::Residential => (BuildingType::House, "house"),
                    Zone::Commercial => (BuildingType::Shop, "shop"),
                    Zone::Industrial => (BuildingType::Factory, "factory"),
                    Zone::Public => (BuildingType::Warehouse, "warehouse"),
                };
                let bid = format!("building-{:04}", building_counter);
                let capacity = default_building_capacity(&bt);
                let owner_company = if num_companies > 0 {
                    format!("company-{:03}", building_counter % num_companies)
                } else {
                    "company-000".to_string()
                };
                let building = Building {
                    id: bid.clone(),
                    parcel_id: pid.clone(),
                    building_type: bt.clone(),
                    owner: PropertyOwnership::Company(owner_company.clone()),
                    operator: Some(owner_company),
                    construction_state: ConstructionState::Completed,
                    construction_progress: 1.0,
                    capacity,
                    operational: true,
                    construction_cost: 0.0,
                    maintenance_cost: 0.0,
                };
                let parcel = parcels.get_mut(pid).unwrap();
                parcel.occupied = true;
                parcel.building_id = Some(bid.clone());

                let recipe_key = ConstructionRecipe::recipe_key(&bt);
                let recipe = ConstructionRecipe::default_recipes()
                    .get(&recipe_key)
                    .cloned()
                    .unwrap_or(ConstructionRecipe {
                        building_type: bt.clone(),
                        money_cost: 500.0,
                        resource_cost: HashMap::new(),
                        labor_hours: 8,
                        duration_ticks: 1440,
                        capacity,
                        maintenance_cost: 5.0,
                    });

                let prop_id = format!("property-{:04}", building_counter);
                let property = Property {
                    id: prop_id,
                    parcel_id: pid.clone(),
                    owner: building.owner.clone(),
                    building_type: bt,
                    value: recipe.money_cost,
                    purchase_price: recipe.money_cost,
                    occupancy: 0,
                    building_id: Some(bid.clone()),
                };
                properties.insert(property.id.clone(), property);
                let building_capacity = building.capacity;
                let building_operator = building.operator.clone();
                buildings.insert(bid.clone(), building);
                if let Some(op_cid) = &building_operator {
                    if let Some(comp) = companies.get_mut(op_cid) {
                        comp.building_ids.push(bid.clone());
                        comp.total_capacity += building_capacity;
                    }
                }
                building_counter += 1;
            }
        }

        for district in districts.values_mut() {
            for pid in &district.parcel_ids {
                if let Some(parcel) = parcels.get(pid) {
                    if parcel.occupied {
                        if let Some(bid) = &parcel.building_id {
                            if let Some(b) = buildings.get(bid) {
                                if b.operational {
                                    match b.building_type {
                                        BuildingType::House | BuildingType::Apartment => {
                                            district.residential_capacity += b.capacity
                                        }
                                        BuildingType::Shop | BuildingType::Office => {
                                            district.commercial_capacity += b.capacity
                                        }
                                        _ => district.industrial_capacity += b.capacity,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let construction_recipes = ConstructionRecipe::default_recipes();

        let mut prices = HashMap::new();
        prices.insert("food".to_string(), FOOD_BASE_PRICE);
        prices.insert("water".to_string(), 0.5);
        prices.insert("energy".to_string(), 2.0);
        prices.insert("raw_food".to_string(), 1.0);

        let state = SimulationState {
            metadata: SimulationMetadata {
                name: "N Genesis".to_string(),
                seed,
            },
            agents,
            companies,
            events,
            city: CityLayout {
                districts,
                road_network: vec![
                    ("district-0".to_string(), "district-1".to_string()),
                    ("district-1".to_string(), "district-2".to_string()),
                    ("district-2".to_string(), "district-3".to_string()),
                    ("district-3".to_string(), "district-4".to_string()),
                ],
            },
            markets: MarketState {
                prices,
                buy_orders: HashMap::new(),
                sell_orders: HashMap::new(),
                listings: Vec::new(),
                transactions: Vec::new(),
                previous_demand: HashMap::new(),
                previous_supply: HashMap::new(),
                total_volume: HashMap::new(),
                depth: MarketDepth::new(),
            },
            tick: 0,
            production_recipes: Recipe::default_recipes(),
            labor_market,
            parcels,
            buildings,
            properties,
            construction_recipes,
            construction_projects: HashMap::new(),
        };

        World { state, clock }
    }

    pub fn tick(&mut self) {
        self.clock.advance_tick();
        let tick = self.clock.tick;
        self.state.tick = tick;

        self.process_calendar_transitions(tick);
        let is_night = self.clock.is_night();
        let is_work_hours = self.clock.is_work_hours();

        self.process_needs(tick);
        self.process_job_evaluation(tick);
        self.process_hiring(tick);
        self.process_firing(tick);
        self.process_purchasing(tick);
        self.process_water_purchasing(tick);
        self.process_routines(tick, is_night, is_work_hours);
        self.process_labor(tick);
        self.process_production(tick);
        self.process_market(tick);
        self.process_wages(tick);
        self.process_company_accounting(tick);
        self.process_workforce_adjustment(tick);
        self.process_construction(tick);
        self.process_housing(tick);
        self.process_property_updates(tick);
        self.process_resource_regeneration(tick);

        if self.state.events.len() > 50000 {
            let excess = self.state.events.len() - 25000;
            self.state.events.drain(..excess);
        }

        self.state.labor_market.employments.retain(|e| e.active);
    }

    pub fn advance(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    pub fn tick_count(&self) -> u64 {
        self.clock.tick
    }

    pub fn random(&mut self) -> f64 {
        self.clock.random()
    }

    fn process_needs(&mut self, tick: u64) {
        let mut agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        agent_ids.sort();
        for agent_id in &agent_ids {
            let agent = self.state.agents.get_mut(agent_id).unwrap();

            let prev_hunger = agent.needs.hunger;
            let prev_thirst = agent.needs.thirst;
            let prev_fatigue = agent.needs.fatigue;

            agent.needs.hunger += HUNGER_RATE_PER_HOUR;
            agent.needs.thirst += THIRST_RATE_PER_HOUR;

            match agent.status {
                AgentStatus::Working | AgentStatus::Active | AgentStatus::Idle => {
                    agent.needs.fatigue += FATIGUE_RATE_PER_HOUR_AWAKE;
                }
                AgentStatus::Sleeping => {
                    agent.needs.fatigue -= FATIGUE_RECOVERY_RATE_PER_HOUR_SLEEPING;
                }
                AgentStatus::Resting => {
                    agent.needs.fatigue -= FATIGUE_RECOVERY_RATE_PER_HOUR_RESTING;
                }
                _ => {}
            }

            agent.needs.clamp();

            let hunger_delta = (agent.needs.hunger - prev_hunger).abs();
            let thirst_delta = (agent.needs.thirst - prev_thirst).abs();
            let fatigue_delta = (agent.needs.fatigue - prev_fatigue).abs();

            if hunger_delta > NEEDS_CHANGE_EVENT_THRESHOLD
                || thirst_delta > NEEDS_CHANGE_EVENT_THRESHOLD
                || fatigue_delta > NEEDS_CHANGE_EVENT_THRESHOLD
            {
                self.state.events.push(Event {
                    id: format!("evt-need-{}-{}", agent_id, tick),
                    tick,
                    event_type: EventType::AgentNeedChanged,
                    actor: Some(agent_id.clone()),
                    cause: Some("time_passage".to_string()),
                    entities: vec![agent_id.clone()],
                    state_snapshot: format!(
                        "hunger={:.2} thirst={:.2} fatigue={:.2}",
                        agent.needs.hunger, agent.needs.thirst, agent.needs.fatigue
                    ),
                });
            }
        }
    }

    fn process_purchasing(&mut self, tick: u64) {
        let food_price = self
            .state
            .markets
            .prices
            .get("food")
            .copied()
            .unwrap_or(FOOD_BASE_PRICE);

        let mut agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        agent_ids.sort();
        for agent_id in &agent_ids {
            let (hunger, food_qty, money, agent_location) = {
                let agent = self.state.agents.get(agent_id).unwrap();
                (
                    agent.needs.hunger,
                    agent.inventory.resource_quantity("food"),
                    agent.money,
                    agent.location.clone(),
                )
            };

            if !agent_wants_food(hunger, food_qty) {
                continue;
            }

            let desired_qty = compute_desired_food_quantity(hunger, food_qty);
            if desired_qty == 0 {
                continue;
            }

            let max_affordable = (money / food_price) as u32;
            let buy_qty = desired_qty.min(max_affordable);
            if buy_qty == 0 {
                continue;
            }

            let mut best_listing_idx = None;
            let mut best_price = f64::MAX;
            for (i, listing) in self.state.markets.listings.iter().enumerate() {
                if listing.resource == "food"
                    && listing.quantity >= buy_qty
                    && listing.price <= money / buy_qty as f64
                    && listing.price < best_price
                {
                    let same_location = self
                        .state
                        .companies
                        .get(&listing.seller_id)
                        .map(|c| c.location == agent_location)
                        .unwrap_or(false);
                    if same_location || best_listing_idx.is_none() {
                        best_listing_idx = Some(i);
                        best_price = listing.price;
                    }
                }
            }

            if let Some(idx) = best_listing_idx {
                let listing = &self.state.markets.listings[idx];
                let seller_id = listing.seller_id.clone();
                let resource = listing.resource.clone();
                let price = listing.price;
                let total_cost = price * buy_qty as f64;

                let agent = self.state.agents.get_mut(agent_id).unwrap();
                if agent.money >= total_cost {
                    agent.money -= total_cost;
                    let (ok, actual) = agent.inventory.add_resource(&resource, buy_qty);
                    if ok && actual > 0 {
                        self.state.markets.listings[idx].quantity -= actual;

                        let company = self.state.companies.get_mut(&seller_id).unwrap();
                        company.cash += total_cost;
                        company.revenue += total_cost;

                        self.state.markets.transactions.push(MarketTransaction {
                            buyer_id: agent_id.clone(),
                            seller_id: seller_id.clone(),
                            resource: resource.clone(),
                            quantity: actual,
                            price,
                            total_cost,
                            tick,
                        });

                        self.state.events.push(Event {
                            id: format!("evt-purchase-{}-{}", agent_id, tick),
                            tick,
                            event_type: EventType::FoodPurchased,
                            actor: Some(agent_id.clone()),
                            cause: Some("hunger_driven".to_string()),
                            entities: vec![agent_id.clone(), seller_id],
                            state_snapshot: format!(
                                "Bought {} {} at {:.2} N each, total {:.2} N",
                                actual, resource, price, total_cost
                            ),
                        });
                    }
                }
            }
        }
    }

    fn process_routines(&mut self, tick: u64, is_night: bool, is_work_hours: bool) {
        let mut agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        agent_ids.sort();
        for agent_id in &agent_ids {
            let agent = self.state.agents.get(agent_id).unwrap();
            let current_status = agent.status.clone();
            let has_food = agent.inventory.has_resource("food");
            let has_water = agent.inventory.has_resource("water");

            let new_status = if agent.needs.needs_drinking() && has_water {
                AgentStatus::Drinking
            } else if agent.needs.needs_eating() && has_food {
                AgentStatus::Eating
            } else if agent.needs.needs_sleeping() && is_night {
                AgentStatus::Sleeping
            } else if agent.needs.needs_resting() {
                AgentStatus::Resting
            } else if agent.employer.is_some() && is_work_hours {
                AgentStatus::Working
            } else if agent.needs.needs_sleeping() {
                AgentStatus::Sleeping
            } else {
                AgentStatus::Idle
            };

            if new_status != current_status {
                let event_type = match &new_status {
                    AgentStatus::Working => Some(EventType::AgentStartedWork),
                    AgentStatus::Sleeping => Some(EventType::AgentStartedSleep),
                    AgentStatus::Resting => Some(EventType::AgentStartedRest),
                    _ => None,
                };

                if let Some(evt) = event_type {
                    self.state.events.push(Event {
                        id: format!("evt-status-{}-{}", agent_id, tick),
                        tick,
                        event_type: evt,
                        actor: Some(agent_id.clone()),
                        cause: Some(format!("status_change_{:?}", current_status)),
                        entities: vec![agent_id.clone()],
                        state_snapshot: format!("{:?} -> {:?}", current_status, new_status),
                    });
                }

                if current_status == AgentStatus::Working {
                    self.state.events.push(Event {
                        id: format!("evt-stopwork-{}-{}", agent_id, tick),
                        tick,
                        event_type: EventType::AgentStoppedWork,
                        actor: Some(agent_id.clone()),
                        cause: Some("routine_change".to_string()),
                        entities: vec![agent_id.clone()],
                        state_snapshot: format!("Stopped working, now {:?}", new_status),
                    });
                }

                let agent = self.state.agents.get_mut(agent_id).unwrap();
                agent.status = new_status;
            }

            let agent = self.state.agents.get_mut(agent_id).unwrap();
            match agent.status {
                AgentStatus::Drinking => {
                    let removed = agent.inventory.remove_resource("water", 1);
                    if removed.0 {
                        agent.needs.thirst -= THIRST_REDUCTION_PER_WATER;
                        agent.needs.clamp();
                        self.state.events.push(Event {
                            id: format!("evt-drink-{}-{}", agent_id, tick),
                            tick,
                            event_type: EventType::AgentDrank,
                            actor: Some(agent_id.clone()),
                            cause: Some("thirst_driven".to_string()),
                            entities: vec![agent_id.clone()],
                            state_snapshot: format!(
                                "Drank water, thirst now {:.2}",
                                agent.needs.thirst
                            ),
                        });
                    }
                }
                AgentStatus::Eating => {
                    let removed = agent.inventory.remove_resource("food", 1);
                    if removed.0 {
                        agent.needs.hunger -= HUNGER_REDUCTION_PER_FOOD;
                        agent.needs.clamp();
                        self.state.events.push(Event {
                            id: format!("evt-eat-{}-{}", agent_id, tick),
                            tick,
                            event_type: EventType::AgentAte,
                            actor: Some(agent_id.clone()),
                            cause: Some("hunger_driven".to_string()),
                            entities: vec![agent_id.clone()],
                            state_snapshot: format!(
                                "Ate food, hunger now {:.2}",
                                agent.needs.hunger
                            ),
                        });
                    }
                }
                AgentStatus::Working => {
                    let agent = self.state.agents.get_mut(agent_id).unwrap();
                    let skill_key = "productivity".to_string();
                    let current_skill = agent.skills.get("productivity").copied().unwrap_or(0.0);
                    let current_xp = agent.experience.get("productivity").copied().unwrap_or(0.0);

                    let xp_gain = SKILL_XP_BASE_RATE * (1.0 - current_skill);
                    let new_xp = current_xp + xp_gain;
                    let new_skill = (current_skill + xp_gain).min(1.0);

                    if (new_skill - current_skill) > 0.0001 {
                        self.state.events.push(Event {
                            id: format!("evt-skill-{}-{}", agent_id, tick),
                            tick,
                            event_type: EventType::SkillImproved,
                            actor: Some(agent_id.clone()),
                            cause: Some("work_experience".to_string()),
                            entities: vec![agent_id.clone()],
                            state_snapshot: format!(
                                "productivity: {:.4} -> {:.4}",
                                current_skill, new_skill
                            ),
                        });
                    }

                    let agent = self.state.agents.get_mut(agent_id).unwrap();
                    agent.skills.insert(skill_key, new_skill);
                    agent.experience.insert("productivity".to_string(), new_xp);
                }
                _ => {}
            }
        }
    }

    fn process_labor(&mut self, tick: u64) {
        let mut company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
        company_ids.sort();

        for company_id in &company_ids {
            let employments: Vec<_> = self
                .state
                .labor_market
                .employments_for_company(company_id)
                .into_iter()
                .map(|e| e.agent_id.clone())
                .collect();

            let working_agents: Vec<String> = employments
                .iter()
                .filter(|aid| {
                    self.state
                        .agents
                        .get(*aid)
                        .map(|a| a.status == AgentStatus::Working)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            if working_agents.is_empty() {
                continue;
            }

            let working_count = working_agents.len();
            let average_skill = {
                let mut total_skill = 0.0;
                let mut skill_count = 0u32;
                for aid in &working_agents {
                    if let Some(agent) = self.state.agents.get(aid) {
                        if let Some(skill) = agent.skills.get("productivity") {
                            total_skill += skill;
                            skill_count += 1;
                        }
                    }
                }
                if skill_count > 0 {
                    total_skill / skill_count as f64
                } else {
                    0.5
                }
            };

            self.state.events.push(Event {
                id: format!("evt-labor-{}-{}", company_id, tick),
                tick,
                event_type: EventType::AgentStartedWork,
                actor: Some(company_id.clone()),
                cause: Some(format!("{} workers contributing labor", working_count)),
                entities: working_agents,
                state_snapshot: format!(
                    "Company {} received {} units of labor, avg skill {:.2}",
                    company_id, working_count, average_skill
                ),
            });
        }
    }

    fn process_production(&mut self, tick: u64) {
        let mut company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
        company_ids.sort();
        let recipe_names: Vec<Option<String>> = company_ids
            .iter()
            .map(|cid| {
                self.state
                    .companies
                    .get(cid)
                    .and_then(|c| c.recipe_name.clone())
            })
            .collect();

        for (i, company_id) in company_ids.iter().enumerate() {
            let recipe_name = match &recipe_names[i] {
                Some(name) => name.clone(),
                None => continue,
            };

            let recipe = match self.state.production_recipes.get(&recipe_name) {
                Some(r) => r.clone(),
                None => continue,
            };

            let company = self.state.companies.get(company_id).unwrap();
            if !company.active {
                continue;
            }
            let cooldown = company.production_cooldown;
            let required_workers = company.required_workers;

            let employments = self
                .state
                .labor_market
                .employments_for_company(company_id)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();

            let mut working_count = 0usize;
            let mut skill_sum = 0.0f64;
            for emp in &employments {
                if let Some(a) = self.state.agents.get(&emp.agent_id) {
                    if a.status == AgentStatus::Working {
                        working_count += 1;
                        skill_sum += a.skills.get("productivity").copied().unwrap_or(0.5);
                    }
                }
            }

            let labor_ratio = if required_workers > 0 {
                (working_count as f64 / required_workers as f64).min(1.0)
            } else {
                1.0
            };

            let average_skill = if working_count > 0 {
                skill_sum / working_count as f64
            } else {
                0.5
            };

            let can = {
                let inv = &self.state.companies.get(company_id).unwrap().inventory;
                can_produce(&recipe, inv, working_count, cooldown)
            };

            if can {
                self.state.events.push(Event {
                    id: format!("evt-prodstart-{}-{}", company_id, tick),
                    tick,
                    event_type: EventType::ProductionStarted,
                    actor: Some(company_id.clone()),
                    cause: Some(format!("recipe: {}", recipe_name)),
                    entities: vec![company_id.clone()],
                    state_snapshot: format!(
                        "Starting production of {} with {} workers ({:.0}% labor capacity), avg skill {:.2}",
                        recipe_name, working_count, labor_ratio * 100.0, average_skill
                    ),
                });

                let company = self.state.companies.get_mut(company_id).unwrap();
                let result = execute_production(&recipe, &mut company.inventory, average_skill);

                match result {
                    Ok(outputs) => {
                        for output in &outputs {
                            self.state.events.push(Event {
                                id: format!(
                                    "evt-prodcomp-{}-{}-{}",
                                    company_id, output.resource, tick
                                ),
                                tick,
                                event_type: EventType::ProductionCompleted,
                                actor: Some(company_id.clone()),
                                cause: Some(format!(
                                    "produced {} {}",
                                    output.quantity, output.resource
                                )),
                                entities: vec![company_id.clone()],
                                state_snapshot: format!(
                                    "Produced {} {} from {} (labor: {:.0}%)",
                                    output.quantity,
                                    output.resource,
                                    recipe_name,
                                    labor_ratio * 100.0
                                ),
                            });
                        }
                        let company = self.state.companies.get_mut(company_id).unwrap();
                        company.production_cooldown = recipe.cooldown_ticks;
                    }
                    Err(_) => {
                        let company = self.state.companies.get_mut(company_id).unwrap();
                        company.production_cooldown = recipe.cooldown_ticks;
                    }
                }
            } else if cooldown > 0 {
                let company = self.state.companies.get_mut(company_id).unwrap();
                company.production_cooldown = cooldown - 1;
            }
        }
    }

    fn process_market(&mut self, tick: u64) {
        let mut food_listing_data: Vec<(String, u32)> = self
            .state
            .companies
            .values()
            .filter(|c| c.active)
            .filter_map(|c| {
                c.inventory
                    .get("food")
                    .copied()
                    .filter(|q| *q > 0)
                    .map(|q| (c.id.clone(), q))
            })
            .collect();
        food_listing_data.sort_by(|a, b| a.0.cmp(&b.0));

        let mut water_listing_data: Vec<(String, u32)> = self
            .state
            .companies
            .values()
            .filter(|c| c.active)
            .filter_map(|c| {
                c.inventory
                    .get("water")
                    .copied()
                    .filter(|q| *q > 0)
                    .map(|q| (c.id.clone(), q))
            })
            .collect();
        water_listing_data.sort_by(|a, b| a.0.cmp(&b.0));

        let food_price = self
            .state
            .markets
            .prices
            .get("food")
            .copied()
            .unwrap_or(FOOD_BASE_PRICE);

        let water_price = self
            .state
            .markets
            .prices
            .get("water")
            .copied()
            .unwrap_or(WATER_BASE_PRICE);

        for (company_id, food_qty) in &food_listing_data {
            let existing = self
                .state
                .markets
                .listings
                .iter()
                .position(|l| l.seller_id == *company_id && l.resource == "food");

            match existing {
                Some(idx) => {
                    self.state.markets.listings[idx].quantity += food_qty;
                    self.state.markets.listings[idx].original_quantity += food_qty;
                    self.state.markets.listings[idx].price = food_price;
                }
                None => {
                    self.state.markets.listings.push(MarketListing {
                        seller_id: company_id.clone(),
                        resource: "food".to_string(),
                        quantity: *food_qty,
                        price: food_price,
                        original_quantity: *food_qty,
                    });

                    self.state.events.push(Event {
                        id: format!("evt-listing-{}-{}", company_id, tick),
                        tick,
                        event_type: EventType::MarketListingCreated,
                        actor: Some(company_id.clone()),
                        cause: Some("market_listing".to_string()),
                        entities: vec![company_id.clone()],
                        state_snapshot: format!("Listed {} food at {:.2} N", food_qty, food_price),
                    });
                }
            }

            if let Some(company) = self.state.companies.get_mut(company_id) {
                company.inventory.remove("food");
            }
        }

        for (company_id, water_qty) in &water_listing_data {
            let existing = self
                .state
                .markets
                .listings
                .iter()
                .position(|l| l.seller_id == *company_id && l.resource == "water");

            match existing {
                Some(idx) => {
                    self.state.markets.listings[idx].quantity += water_qty;
                    self.state.markets.listings[idx].original_quantity += water_qty;
                    self.state.markets.listings[idx].price = water_price;
                }
                None => {
                    self.state.markets.listings.push(MarketListing {
                        seller_id: company_id.clone(),
                        resource: "water".to_string(),
                        quantity: *water_qty,
                        price: water_price,
                        original_quantity: *water_qty,
                    });

                    self.state.events.push(Event {
                        id: format!("evt-listing-water-{}-{}", company_id, tick),
                        tick,
                        event_type: EventType::MarketListingCreated,
                        actor: Some(company_id.clone()),
                        cause: Some("market_listing".to_string()),
                        entities: vec![company_id.clone()],
                        state_snapshot: format!(
                            "Listed {} water at {:.2} N",
                            water_qty, water_price
                        ),
                    });
                }
            }

            if let Some(company) = self.state.companies.get_mut(company_id) {
                company.inventory.remove("water");
            }
        }

        let mut demand: HashMap<String, f64> = HashMap::new();
        let mut supply: HashMap<String, f64> = HashMap::new();

        for listing in &self.state.markets.listings {
            *supply.entry(listing.resource.clone()).or_insert(0.0) += listing.quantity as f64;
        }

        let mut food_sold: u32 = 0;
        let mut water_sold: u32 = 0;
        for tx in self
            .state
            .markets
            .transactions
            .iter()
            .filter(|t| t.tick + 24 >= tick)
        {
            *demand.entry(tx.resource.clone()).or_insert(0.0) += tx.quantity as f64;
            if tx.resource == "food" {
                food_sold += tx.quantity;
            } else if tx.resource == "water" {
                water_sold += tx.quantity;
            }
        }

        let food_demand = demand.get("food").copied().unwrap_or(0.0);
        let food_supply = supply.get("food").copied().unwrap_or(0.0);

        let prev_food_demand = self
            .state
            .markets
            .previous_demand
            .get("food")
            .copied()
            .unwrap_or(0.0);

        let food_demand_pressure = if prev_food_demand > 0.0 {
            (food_demand / prev_food_demand - 1.0).max(-1.0).min(1.0)
        } else if food_demand > 0.0 {
            0.5
        } else {
            0.0
        };

        let new_food_price = adjust_price_with_scarcity(
            food_price,
            food_supply as u32,
            food_demand_pressure,
            SCARCITY_THRESHOLD_FOOD,
            FOOD_PRICE_MIN,
            FOOD_PRICE_MAX,
        );

        if (new_food_price - food_price).abs() > 0.01 {
            self.state
                .markets
                .prices
                .insert("food".to_string(), new_food_price);
            self.state.events.push(Event {
                id: format!("evt-price-food-{}", tick),
                tick,
                event_type: EventType::PriceChanged,
                actor: None,
                cause: Some("supply_demand".to_string()),
                entities: vec![],
                state_snapshot: format!(
                    "Food price: {:.2} -> {:.2} (demand: {:.1}, supply: {:.1})",
                    food_price, new_food_price, food_demand, food_supply
                ),
            });
        }

        self.state
            .markets
            .previous_demand
            .insert("food".to_string(), food_demand);
        self.state
            .markets
            .previous_supply
            .insert("food".to_string(), food_supply);

        let water_demand = demand.get("water").copied().unwrap_or(0.0);
        let water_supply = supply.get("water").copied().unwrap_or(0.0);

        let prev_water_demand = self
            .state
            .markets
            .previous_demand
            .get("water")
            .copied()
            .unwrap_or(0.0);

        let water_demand_pressure = if prev_water_demand > 0.0 {
            (water_demand / prev_water_demand - 1.0).max(-1.0).min(1.0)
        } else if water_demand > 0.0 {
            0.5
        } else {
            0.0
        };

        let new_water_price = adjust_price_with_scarcity(
            water_price,
            water_supply as u32,
            water_demand_pressure,
            SCARCITY_THRESHOLD_WATER,
            WATER_PRICE_MIN,
            WATER_PRICE_MAX,
        );

        if (new_water_price - water_price).abs() > 0.01 {
            self.state
                .markets
                .prices
                .insert("water".to_string(), new_water_price);
            self.state.events.push(Event {
                id: format!("evt-price-water-{}", tick),
                tick,
                event_type: EventType::PriceChanged,
                actor: None,
                cause: Some("supply_demand".to_string()),
                entities: vec![],
                state_snapshot: format!(
                    "Water price: {:.2} -> {:.2} (demand: {:.1}, supply: {:.1})",
                    water_price, new_water_price, water_demand, water_supply
                ),
            });
        }

        self.state
            .markets
            .previous_demand
            .insert("water".to_string(), water_demand);
        self.state
            .markets
            .previous_supply
            .insert("water".to_string(), water_supply);

        self.state.markets.depth = MarketDepth {
            food_bid_depth: food_listing_data.iter().map(|(_, q)| *q).sum(),
            food_ask_depth: food_sold,
            water_bid_depth: water_listing_data.iter().map(|(_, q)| *q).sum(),
            water_ask_depth: water_sold,
        };

        self.state
            .markets
            .transactions
            .retain(|t| t.tick + 24 >= tick);
    }

    fn process_wages(&mut self, tick: u64) {
        let is_work_hours = self.clock.is_work_hours();
        if !is_work_hours {
            return;
        }

        let minute_of_hour = self.clock.minute_of_hour();
        if minute_of_hour != 0 {
            return;
        }

        let mut company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
        company_ids.sort();

        for company_id in &company_ids {
            let company = match self.state.companies.get(company_id) {
                Some(c) if c.active => c,
                _ => continue,
            };
            let company_cash = company.cash;

            let employments: Vec<(String, f64)> = self
                .state
                .labor_market
                .employments_for_company(company_id)
                .iter()
                .map(|e| (e.agent_id.clone(), e.wage))
                .collect();

            if employments.is_empty() {
                continue;
            }

            let mut total_wages = 0.0;
            let mut paid_employees = Vec::new();

            for (emp_id, wage) in &employments {
                if company_cash - total_wages >= *wage {
                    total_wages += wage;
                    paid_employees.push(emp_id.clone());
                }
            }

            if total_wages > 0.0 {
                let company = self.state.companies.get_mut(company_id).unwrap();
                company.cash -= total_wages;
                company.expenses += total_wages;

                self.state.labor_market.total_wages_paid += total_wages;

                for (emp_id, wage) in &employments {
                    if paid_employees.contains(emp_id) {
                        let agent = self.state.agents.get_mut(emp_id).unwrap();
                        agent.money += wage;
                    }
                }

                self.state.events.push(Event {
                    id: format!("evt-wage-{}-{}", company_id, tick),
                    tick,
                    event_type: EventType::WagePaid,
                    actor: Some(company_id.clone()),
                    cause: Some("regular_wage".to_string()),
                    entities: paid_employees.clone(),
                    state_snapshot: format!(
                        "Paid {:.0} N to {} employees",
                        total_wages,
                        paid_employees.len()
                    ),
                });
            }
        }
    }

    fn process_calendar_transitions(&mut self, tick: u64) {
        if tick == 0 {
            return;
        }
        let prev_tick = tick - 1;

        let prev_hour = (prev_tick / TICKS_PER_HOUR) % HOURS_PER_DAY;
        let curr_hour = (tick / TICKS_PER_HOUR) % HOURS_PER_DAY;
        if curr_hour != prev_hour {
            self.state.events.push(Event {
                id: format!("evt-hour-{}", tick),
                tick,
                event_type: EventType::HourStarted,
                actor: None,
                cause: None,
                entities: vec![],
                state_snapshot: format!("Hour {} started", curr_hour),
            });
        }

        let prev_day = prev_tick / TICKS_PER_DAY;
        let curr_day = tick / TICKS_PER_DAY;
        if curr_day != prev_day {
            self.state.events.push(Event {
                id: format!("evt-day-{}", tick),
                tick,
                event_type: EventType::DayStarted,
                actor: None,
                cause: None,
                entities: vec![],
                state_snapshot: format!(
                    "Day {} started (month day {})",
                    curr_day + 1,
                    self.clock.day_of_month()
                ),
            });

            let prev_week = prev_day / DAYS_PER_WEEK;
            let curr_week = curr_day / DAYS_PER_WEEK;
            if curr_week != prev_week {
                self.state.events.push(Event {
                    id: format!("evt-week-{}", tick),
                    tick,
                    event_type: EventType::WeekStarted,
                    actor: None,
                    cause: None,
                    entities: vec![],
                    state_snapshot: format!("Week {} started", curr_week + 1),
                });
            }

            let prev_month = prev_day / DAYS_PER_MONTH;
            let curr_month = curr_day / DAYS_PER_MONTH;
            if curr_month != prev_month {
                self.state.events.push(Event {
                    id: format!("evt-month-{}", tick),
                    tick,
                    event_type: EventType::MonthStarted,
                    actor: None,
                    cause: None,
                    entities: vec![],
                    state_snapshot: format!("Month {} started", curr_month + 1),
                });
            }

            let prev_year = prev_day / (DAYS_PER_MONTH * MONTHS_PER_YEAR);
            let curr_year = curr_day / (DAYS_PER_MONTH * MONTHS_PER_YEAR);
            if curr_year != prev_year {
                self.state.events.push(Event {
                    id: format!("evt-year-{}", tick),
                    tick,
                    event_type: EventType::YearStarted,
                    actor: None,
                    cause: None,
                    entities: vec![],
                    state_snapshot: format!("Year {} started", curr_year + 1),
                });
            }
        }
    }

    fn process_water_purchasing(&mut self, tick: u64) {
        let water_price = self
            .state
            .markets
            .prices
            .get("water")
            .copied()
            .unwrap_or(WATER_BASE_PRICE);

        let mut agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        agent_ids.sort();
        for agent_id in &agent_ids {
            let (thirst, water_qty, money, agent_location) = {
                let agent = self.state.agents.get(agent_id).unwrap();
                (
                    agent.needs.thirst,
                    agent.inventory.resource_quantity("water"),
                    agent.money,
                    agent.location.clone(),
                )
            };

            if !agent_wants_water(thirst, water_qty) {
                continue;
            }

            let desired_qty = compute_desired_water_quantity(thirst, water_qty);
            if desired_qty == 0 {
                continue;
            }

            let max_affordable = (money / water_price) as u32;
            let buy_qty = desired_qty.min(max_affordable);
            if buy_qty == 0 {
                continue;
            }

            let mut best_listing_idx = None;
            let mut best_price = f64::MAX;
            for (i, listing) in self.state.markets.listings.iter().enumerate() {
                if listing.resource == "water"
                    && listing.quantity >= buy_qty
                    && listing.price <= money / buy_qty as f64
                    && listing.price < best_price
                {
                    let same_location = self
                        .state
                        .companies
                        .get(&listing.seller_id)
                        .map(|c| c.location == agent_location)
                        .unwrap_or(false);
                    if same_location || best_listing_idx.is_none() {
                        best_listing_idx = Some(i);
                        best_price = listing.price;
                    }
                }
            }

            if let Some(idx) = best_listing_idx {
                let listing = &self.state.markets.listings[idx];
                let seller_id = listing.seller_id.clone();
                let resource = listing.resource.clone();
                let price = listing.price;
                let total_cost = price * buy_qty as f64;

                let agent = self.state.agents.get_mut(agent_id).unwrap();
                if agent.money >= total_cost {
                    agent.money -= total_cost;
                    let (ok, actual) = agent.inventory.add_resource(&resource, buy_qty);
                    if ok && actual > 0 {
                        self.state.markets.listings[idx].quantity -= actual;

                        let company = self.state.companies.get_mut(&seller_id).unwrap();
                        company.cash += total_cost;
                        company.revenue += total_cost;

                        *self
                            .state
                            .markets
                            .total_volume
                            .entry("water".to_string())
                            .or_insert(0) += actual as u64;

                        self.state.markets.transactions.push(MarketTransaction {
                            buyer_id: agent_id.clone(),
                            seller_id: seller_id.clone(),
                            resource: resource.clone(),
                            quantity: actual,
                            price,
                            total_cost,
                            tick,
                        });

                        self.state.events.push(Event {
                            id: format!("evt-water-purchase-{}-{}", agent_id, tick),
                            tick,
                            event_type: EventType::WaterPurchased,
                            actor: Some(agent_id.clone()),
                            cause: Some("thirst_driven".to_string()),
                            entities: vec![agent_id.clone(), seller_id],
                            state_snapshot: format!(
                                "Bought {} {} at {:.2} N each, total {:.2} N",
                                actual, resource, price, total_cost
                            ),
                        });
                    }
                }
            }
        }
    }

    fn process_job_evaluation(&mut self, tick: u64) {
        let mut agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        agent_ids.sort();
        let mut companies_snapshot: Vec<(String, String, f64)> = self
            .state
            .companies
            .iter()
            .filter(|(_, c)| c.active)
            .map(|(id, c)| (id.clone(), c.location.clone(), c.wage))
            .collect();
        companies_snapshot.sort_by(|a, b| a.0.cmp(&b.0));

        for agent_id in &agent_ids {
            let (is_employed, agent_location, agent_skills) = {
                let agent = self.state.agents.get(agent_id).unwrap();
                (
                    self.state.labor_market.is_agent_employed(agent_id),
                    agent.location.clone(),
                    agent.skills.clone(),
                )
            };

            if is_employed {
                continue;
            }

            let openings: Vec<JobOffer> = self.state.labor_market.job_openings.clone();

            let mut best_score = -1.0_f64;
            let mut best_opening_idx: Option<usize> = None;

            for (i, opening) in openings.iter().enumerate() {
                let live_count = self.state.labor_market.job_openings[i].openings;
                if live_count == 0 {
                    continue;
                }
                let company_info = companies_snapshot
                    .iter()
                    .find(|(id, _, _)| id == &opening.company_id);
                let (_, ref company_loc, _) = match company_info {
                    Some(info) => info,
                    None => continue,
                };

                let skill_level = agent_skills
                    .get(&opening.required_skill)
                    .copied()
                    .unwrap_or(0.0);
                if skill_level < opening.min_skill_level {
                    continue;
                }

                let location_match = *company_loc == agent_location;
                let skill_match = (skill_level - opening.min_skill_level).max(0.0);
                let score =
                    LaborMarketState::compute_job_score(opening.wage, skill_match, location_match);

                if score > best_score {
                    best_score = score;
                    best_opening_idx = Some(i);
                }
            }

            if let Some(idx) = best_opening_idx {
                if self.state.labor_market.job_openings[idx].openings == 0 {
                    continue;
                }
                self.state.labor_market.job_openings[idx].openings -= 1;

                let company_id = openings[idx].company_id.clone();
                let role = openings[idx].role.clone();
                let wage = openings[idx].wage;
                let required_skill = openings[idx].required_skill.clone();
                let min_skill = openings[idx].min_skill_level;

                let hired = self.state.labor_market.create_employment(
                    agent_id.clone(),
                    company_id.clone(),
                    role.clone(),
                    wage,
                    tick,
                );

                if hired {
                    let agent = self.state.agents.get_mut(agent_id).unwrap();
                    agent.employer = Some(company_id.clone());

                    if let Some(company) = self.state.companies.get_mut(&company_id) {
                        company.employees.insert(agent_id.clone(), 1);
                        company.last_hire_tick = tick;
                    }

                    self.state.events.push(Event {
                        id: format!("evt-hire-{}-{}-{}", company_id, agent_id, tick),
                        tick,
                        event_type: EventType::AgentHired,
                        actor: Some(company_id.clone()),
                        cause: Some(format!("job_score={:.3}", best_score)),
                        entities: vec![agent_id.clone(), company_id],
                        state_snapshot: format!(
                            "Hired {} as {} at {:.1} N/tick (skill: {} {:.2})",
                            agent_id, role, wage, required_skill, min_skill
                        ),
                    });
                }
            }
        }
    }

    fn process_hiring(&mut self, tick: u64) {
        let mut company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
        company_ids.sort();

        for company_id in &company_ids {
            let (active, is_insolvent, desired, current_count, wage, _location, last_hire) = {
                let company = self.state.companies.get(company_id).unwrap();
                (
                    company.active,
                    company.status == CompanyStatus::Insolvent,
                    company.desired_workforce,
                    self.state
                        .labor_market
                        .employee_count_for_company(company_id),
                    company.wage,
                    company.location.clone(),
                    company.last_hire_tick,
                )
            };

            if !active || is_insolvent {
                continue;
            }

            if tick >= last_hire && tick - last_hire < HIRE_COOLDOWN {
                continue;
            }

            if current_count >= desired {
                continue;
            }

            let needed = desired - current_count;
            let openings: Vec<JobOffer> = self
                .state
                .labor_market
                .job_openings
                .iter()
                .filter(|o| o.company_id == *company_id)
                .cloned()
                .collect();
            let total_openings: u32 = openings.iter().map(|o| o.openings).sum();
            if total_openings >= needed as u32 {
                continue;
            }

            let company = self.state.companies.get(company_id).unwrap();
            let role = match company.company_type {
                CompanyType::FoodProducer => "Production Worker",
                CompanyType::WaterProducer => "Water Technician",
                CompanyType::Retailer => "Sales Associate",
                CompanyType::Service => "Service Worker",
            };

            let required_skill = match company.company_type {
                CompanyType::FoodProducer | CompanyType::WaterProducer => "productivity",
                _ => "social",
            };

            let job_offer = JobOffer {
                company_id: company_id.clone(),
                role: role.to_string(),
                wage,
                required_skill: required_skill.to_string(),
                min_skill_level: 0.2,
                openings: (needed as u32).max(1),
            };

            self.state.labor_market.job_openings.push(job_offer.clone());

            self.state.events.push(Event {
                id: format!("evt-jobpost-{}-{}", company_id, tick),
                tick,
                event_type: EventType::JobPosted,
                actor: Some(company_id.clone()),
                cause: Some(format!("workforce {}/{}", current_count, desired)),
                entities: vec![company_id.clone()],
                state_snapshot: format!(
                    "Posted {} openings for {} at {:.1} N/tick",
                    job_offer.openings, role, wage
                ),
            });
        }

        self.state
            .labor_market
            .job_openings
            .retain(|o| o.openings > 0);
    }

    fn process_firing(&mut self, tick: u64) {
        let mut company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
        company_ids.sort();

        for company_id in &company_ids {
            let (active, cash, loss_streak, current_count, min_workforce, _revenue) = {
                let company = self.state.companies.get(company_id).unwrap();
                (
                    company.active,
                    company.cash,
                    company.loss_streak,
                    self.state
                        .labor_market
                        .employee_count_for_company(company_id),
                    company.min_workforce,
                    company.revenue,
                )
            };

            if !active {
                continue;
            }

            let total_wages = self
                .state
                .labor_market
                .total_wage_cost_for_company(company_id);

            let should_fire = (loss_streak >= COMPANY_LOSS_STREAK_TO_FIRE
                && cash < COMPANY_INSOLVENCY_THRESHOLD
                && current_count > min_workforce)
                || (cash < total_wages && current_count > min_workforce);

            if should_fire {
                // Fire most recently hired (LIFO) - one at a time for gradual reduction
                let employees: Vec<String> = self
                    .state
                    .labor_market
                    .employments_for_company(company_id)
                    .iter()
                    .map(|e| e.agent_id.clone())
                    .collect();

                if let Some(emp_id) = employees.last() {
                    let fired = self
                        .state
                        .labor_market
                        .terminate_employment(emp_id, company_id);

                    if fired {
                        let agent = self.state.agents.get_mut(emp_id).unwrap();
                        agent.employer = None;

                        if let Some(company) = self.state.companies.get_mut(company_id) {
                            company.employees.remove(emp_id);
                        }

                        self.state.events.push(Event {
                            id: format!("evt-fire-{}-{}-{}", company_id, emp_id, tick),
                            tick,
                            event_type: EventType::AgentFired,
                            actor: Some(company_id.clone()),
                            cause: Some(format!("loss_streak={}, cash={:.0}", loss_streak, cash)),
                            entities: vec![emp_id.clone(), company_id.clone()],
                            state_snapshot: format!(
                                "Fired {} from {} (workforce now {})",
                                emp_id,
                                company_id,
                                current_count - 1
                            ),
                        });
                    }
                }
            }
        }
    }

    fn process_company_accounting(&mut self, tick: u64) {
        let mut company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
        company_ids.sort();

        for company_id in &company_ids {
            let (active, cash, revenue, expenses, cumulative) = {
                let company = self.state.companies.get(company_id).unwrap();
                (
                    company.active,
                    company.cash,
                    company.revenue,
                    company.expenses,
                    company.cumulative_profit,
                )
            };

            if !active {
                continue;
            }

            let profit = revenue - expenses;
            let new_cumulative = cumulative + profit;

            let company = self.state.companies.get_mut(company_id).unwrap();
            company.profit_loss = profit;
            company.cumulative_profit = new_cumulative;

            if profit < 0.0 {
                company.loss_streak += 1;

                if company.cash < COMPANY_INSOLVENCY_THRESHOLD
                    && company.loss_streak >= COMPANY_LOSS_STREAK_TO_FIRE
                {
                    let fired = self
                        .state
                        .labor_market
                        .terminate_all_for_company(company_id);

                    for emp_id in &fired {
                        if let Some(agent) = self.state.agents.get_mut(emp_id) {
                            agent.employer = None;
                        }
                    }

                    self.state
                        .markets
                        .listings
                        .retain(|l| l.seller_id != *company_id);

                    company.active = false;
                    company.status = CompanyStatus::Insolvent;
                    company.closed_at_tick = Some(tick);

                    self.state.events.push(Event {
                        id: format!("evt-insolvent-{}-{}", company_id, tick),
                        tick,
                        event_type: EventType::CompanyClosedByInsolvency,
                        actor: Some(company_id.clone()),
                        cause: Some(format!(
                            "cash={:.0}, loss_streak={}",
                            cash, company.loss_streak
                        )),
                        entities: fired,
                        state_snapshot: format!(
                            "Company {} became insolvent and closed",
                            company_id
                        ),
                    });
                }
            } else {
                company.loss_streak = 0;
            }

            let employee_count = self
                .state
                .labor_market
                .employee_count_for_company(company_id);

            let company = self.state.companies.get_mut(company_id).unwrap();
            if employee_count < company.min_workforce {
                company.status = CompanyStatus::Understaffed;
            } else if employee_count > company.max_workforce {
                company.status = CompanyStatus::Overstaffed;
            } else if company.active {
                company.status = CompanyStatus::Operating;
            }

            company.revenue = 0.0;
            company.expenses = 0.0;
        }
    }

    fn process_workforce_adjustment(&mut self, tick: u64) {
        let mut company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
        company_ids.sort();

        for company_id in &company_ids {
            let (active, profit, cash, desired, max_wf, last_growth) = {
                let company = self.state.companies.get(company_id).unwrap();
                (
                    company.active,
                    company.profit_loss,
                    company.cash,
                    company.desired_workforce,
                    company.max_workforce,
                    company.last_growth_tick,
                )
            };

            if !active {
                continue;
            }

            if profit > PROFIT_FOR_GROWTH
                && cash > CASH_FOR_GROWTH
                && desired < max_wf
                && tick >= last_growth
                && tick - last_growth >= COMPANY_GROWTH_COOLDOWN
            {
                let new_desired = (desired + 1).min(max_wf);
                let company = self.state.companies.get_mut(company_id).unwrap();
                company.desired_workforce = new_desired;
                company.last_growth_tick = tick;

                self.state.events.push(Event {
                    id: format!("evt-expand-{}-{}", company_id, tick),
                    tick,
                    event_type: EventType::CompanyExpanded,
                    actor: Some(company_id.clone()),
                    cause: Some(format!("profit={:.0}, cash={:.0}", profit, cash)),
                    entities: vec![company_id.clone()],
                    state_snapshot: format!(
                        "Company {} expanded workforce target to {}",
                        company_id, new_desired
                    ),
                });
            } else if profit < LOSS_FOR_CONTRACTION && desired > 1 {
                let new_desired = (desired - 1).max(1);
                let company = self.state.companies.get_mut(company_id).unwrap();
                company.desired_workforce = new_desired;

                self.state.events.push(Event {
                    id: format!("evt-contract-{}-{}", company_id, tick),
                    tick,
                    event_type: EventType::CompanyContracted,
                    actor: Some(company_id.clone()),
                    cause: Some(format!("profit={:.0}", profit)),
                    entities: vec![company_id.clone()],
                    state_snapshot: format!(
                        "Company {} contracted workforce target to {}",
                        company_id, new_desired
                    ),
                });
            }
        }
    }

    fn process_housing(&mut self, _tick: u64) {
        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        for agent_id in &agent_ids {
            if self.state.agents.get(agent_id).unwrap().housing_id.is_some() {
                continue;
            }
            let agent_location = self.state.agents.get(agent_id).unwrap().location.clone();
            let available = self.state.buildings.values()
                .filter(|b| {
                    b.operational
                        && matches!(b.building_type, BuildingType::House | BuildingType::Apartment)
                        && self.state.parcels.get(&b.parcel_id)
                            .map(|p| p.district_id == agent_location)
                            .unwrap_or(false)
                })
                .filter(|b| {
                    let current_occupancy = self
                        .state
                        .agents
                        .values()
                        .filter(|a| a.housing_id.as_ref() == Some(&b.id))
                        .count();
                    (current_occupancy as u32) < b.capacity
                })
                .min_by_key(|b| b.id.clone())
                .cloned();
            if let Some(building) = available {
                let agent = self.state.agents.get_mut(agent_id).unwrap();
                agent.housing_id = Some(building.id);
            }
        }
    }

    fn process_construction(&mut self, _tick: u64) {
        let project_ids: Vec<String> = self.state.construction_projects.keys().cloned().collect();
        for pid in project_ids {
            let project = self.state.construction_projects.get_mut(&pid).unwrap();
            if project.status == ConstructionState::UnderConstruction {
                project.progress += 1;
                if project.progress >= project.total_duration {
                    let parcel_id = project.parcel_id.clone();
                    let target_building = project.target_building.clone();
                    let owner = project.owner.clone();
                    project.status = ConstructionState::Completed;
                    let bid = format!("building-{:04}", self.state.buildings.len());
                    let recipe_key = ConstructionRecipe::recipe_key(&target_building);
                    let capacity = self.state.construction_recipes.get(&recipe_key)
                        .map(|r| r.capacity)
                        .unwrap_or(4);
                    let building = Building {
                        id: bid.clone(),
                        parcel_id: parcel_id.clone(),
                        building_type: target_building,
                        owner: PropertyOwnership::Company(owner.clone()),
                        operator: Some(owner.clone()),
                        construction_state: ConstructionState::Completed,
                        construction_progress: 1.0,
                        capacity,
                        operational: true,
                        construction_cost: 0.0,
                        maintenance_cost: 0.0,
                    };
                    self.state.buildings.insert(bid.clone(), building);
                    if let Some(parcel) = self.state.parcels.get_mut(&parcel_id) {
                        parcel.occupied = true;
                        parcel.building_id = Some(bid.clone());
                    }
                    if let Some(comp) = self.state.companies.get_mut(&owner) {
                        comp.building_ids.push(bid);
                        comp.total_capacity += capacity;
                    }
                }
            }
        }
    }

    fn process_property_updates(&mut self, tick: u64) {
        if tick % TICKS_PER_DAY != 0 {
            return;
        }
        for parcel in self.state.parcels.values_mut() {
            let growth = parcel.current_value * PARCEL_VALUE_GROWTH_RATE * 30.0;
            parcel.current_value += growth;
        }
        let district_ids: Vec<String> = self.state.city.districts.keys().cloned().collect();
        for district_id in district_ids {
            let parcel_ids = self
                .state
                .city
                .districts
                .get(&district_id)
                .unwrap()
                .parcel_ids
                .clone();
            let mut total_value = 0.0f64;
            for pid in &parcel_ids {
                if let Some(parcel) = self.state.parcels.get(pid) {
                    total_value += parcel.current_value;
                }
            }
            let buildings_in_district: Vec<String> = self
                .state
                .buildings
                .values()
                .filter(|b| {
                    self.state
                        .parcels
                        .get(&b.parcel_id)
                        .map(|p| p.district_id == district_id)
                        .unwrap_or(false)
                })
                .map(|b| b.id.clone())
                .collect();
            let mut res_cap = 0u32;
            let mut com_cap = 0u32;
            let mut ind_cap = 0u32;
            for bid in &buildings_in_district {
                if let Some(building) = self.state.buildings.get(bid) {
                    if building.operational {
                        match building.building_type {
                            BuildingType::House | BuildingType::Apartment => {
                                res_cap += building.capacity
                            }
                            BuildingType::Shop | BuildingType::Office => {
                                com_cap += building.capacity
                            }
                            _ => ind_cap += building.capacity,
                        }
                    }
                }
            }
            let residents = self
                .state
                .agents
                .values()
                .filter(|a| {
                    if let Some(ref hid) = a.housing_id {
                        if let Some(building) = self.state.buildings.get(hid) {
                            if let Some(parcel) = self.state.parcels.get(&building.parcel_id) {
                                return parcel.district_id == district_id;
                            }
                        }
                    }
                    false
                })
                .count() as u32;
            if let Some(district) = self.state.city.districts.get_mut(&district_id) {
                district.total_property_value = total_value;
                district.residential_capacity = res_cap;
                district.commercial_capacity = com_cap;
                district.industrial_capacity = ind_cap;
                district.residents = residents;
            }
        }
    }

    fn process_resource_regeneration(&mut self, tick: u64) {
        if tick % TICKS_PER_DAY != 0 {
            return;
        }

        for company in self.state.companies.values_mut() {
            if !company.active {
                continue;
            }
            if company.company_type == CompanyType::FoodProducer {
                let raw_food = company.inventory.get("raw_food").copied().unwrap_or(0);
                if raw_food < 500 {
                    let regen = crate::resources::REGENERATION_RATE_FOOD;
                    let new_val = (raw_food + regen).min(500);
                    company.inventory.insert("raw_food".to_string(), new_val);
                }
            } else if company.company_type == CompanyType::WaterProducer {
                let raw_water = company.inventory.get("raw_water").copied().unwrap_or(0);
                if raw_water < 400 {
                    let regen = crate::resources::REGENERATION_RATE_WATER;
                    let new_val = (raw_water + regen).min(400);
                    company.inventory.insert("raw_water".to_string(), new_val);
                }
            }
        }
    }
}

pub fn serialize_state(world: &World) -> String {
    serde_json::to_string_pretty(&world.state).unwrap_or_default()
}

pub fn deserialize_state(json: &str) -> Option<SimulationState> {
    serde_json::from_str(json).ok()
}

pub fn default_world() -> World {
    World::initialize(1234, 50, 10)
}

pub fn run_tick(world: &mut World) -> u64 {
    world.tick();
    world.tick_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── PHASE 2: NEEDS TESTS ──────────────────────────────────────────

    #[test]
    fn needs_start_satisfied() {
        let n = AgentNeeds::new();
        assert_eq!(n.hunger, 0.0);
        assert_eq!(n.thirst, 0.0);
        assert_eq!(n.fatigue, 0.0);
    }

    #[test]
    fn needs_clamp_bounded() {
        let mut n = AgentNeeds {
            hunger: 1.5,
            thirst: -0.5,
            fatigue: 0.8,
        };
        n.clamp();
        assert_eq!(n.hunger, 1.0);
        assert_eq!(n.thirst, 0.0);
        assert_eq!(n.fatigue, 0.8);
    }

    #[test]
    fn hunger_increases_over_time() {
        let mut w = World::initialize(42, 1, 0);
        let initial = w.state.agents.values().next().unwrap().needs.hunger;
        w.advance(10);
        let final_val = w.state.agents.values().next().unwrap().needs.hunger;
        assert!(
            final_val > initial,
            "hunger should increase: {} -> {}",
            initial,
            final_val
        );
    }

    #[test]
    fn thirst_increases_faster_than_hunger() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        w.advance(50);
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert!(
            agent.needs.thirst > agent.needs.hunger,
            "thirst ({}) should exceed hunger ({})",
            agent.needs.thirst,
            agent.needs.hunger
        );
    }

    #[test]
    fn fatigue_increases_while_awake() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        let initial = w.state.agents.get(&agent_id).unwrap().needs.fatigue;
        w.advance(100);
        let final_val = w.state.agents.get(&agent_id).unwrap().needs.fatigue;
        assert!(final_val > initial);
    }

    #[test]
    fn fatigue_never_exceeds_one() {
        let mut w = World::initialize(42, 1, 0);
        w.advance(10000);
        for agent in w.state.agents.values() {
            assert!(agent.needs.fatigue <= 1.0);
        }
    }

    #[test]
    fn needs_never_go_negative() {
        let mut w = World::initialize(42, 1, 0);
        w.advance(1000);
        for agent in w.state.agents.values() {
            assert!(agent.needs.hunger >= 0.0);
            assert!(agent.needs.thirst >= 0.0);
            assert!(agent.needs.fatigue >= 0.0);
        }
    }

    // ─── PHASE 2: INVENTORY TESTS ─────────────────────────────────────

    #[test]
    fn inventory_add_resource() {
        let mut inv = default_inventory();
        let (ok, amt) = inv.add_resource("food", 3);
        assert!(ok);
        assert_eq!(amt, 3);
        assert_eq!(inv.resource_quantity("food"), 3);
    }

    #[test]
    fn inventory_add_respects_capacity() {
        let mut inv = default_inventory();
        let (ok, amt) = inv.add_resource("water", 100);
        assert!(ok);
        assert_eq!(amt, 5);
        assert_eq!(inv.resource_quantity("water"), 5);
    }

    #[test]
    fn inventory_remove_resource() {
        let mut inv = default_inventory();
        inv.add_resource("food", 5);
        let (ok, amt) = inv.remove_resource("food", 2);
        assert!(ok);
        assert_eq!(amt, 2);
        assert_eq!(inv.resource_quantity("food"), 3);
    }

    #[test]
    fn inventory_remove_insufficient() {
        let mut inv = default_inventory();
        inv.add_resource("food", 2);
        let (ok, amt) = inv.remove_resource("food", 5);
        assert!(!ok);
        assert_eq!(amt, 2);
        assert_eq!(inv.resource_quantity("food"), 0);
    }

    #[test]
    fn inventory_has_resource() {
        let mut inv = default_inventory();
        assert!(!inv.has_resource("food"));
        inv.add_resource("food", 1);
        assert!(inv.has_resource("food"));
        inv.remove_resource("food", 1);
        assert!(!inv.has_resource("food"));
    }

    #[test]
    fn inventory_add_zero_is_noop() {
        let mut inv = default_inventory();
        let (ok, amt) = inv.add_resource("food", 0);
        assert!(ok);
        assert_eq!(amt, 0);
    }

    #[test]
    fn inventory_remove_zero_is_noop() {
        let mut inv = default_inventory();
        inv.add_resource("food", 5);
        let (ok, amt) = inv.remove_resource("food", 0);
        assert!(ok);
        assert_eq!(amt, 0);
        assert_eq!(inv.resource_quantity("food"), 5);
    }

    // ─── PHASE 2: ROUTINE TESTS ───────────────────────────────────────

    #[test]
    fn eating_reduces_hunger() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.inventory.add_resource("food", 5);
            agent.needs.hunger = 0.8;
        }
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.status = AgentStatus::Eating;
        }
        w.tick();
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert!(
            agent.needs.hunger < 0.8,
            "hunger should decrease after eating"
        );
    }

    #[test]
    fn drinking_reduces_thirst() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.inventory.add_resource("water", 5);
            agent.needs.thirst = 0.8;
            agent.status = AgentStatus::Drinking;
        }
        w.tick();
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert!(
            agent.needs.thirst < 0.8,
            "thirst should decrease after drinking"
        );
    }

    #[test]
    fn eating_without_food_doesnt_crash() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.needs.hunger = 0.9;
            agent.status = AgentStatus::Eating;
        }
        w.tick();
    }

    #[test]
    fn drinking_without_water_doesnt_crash() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.needs.thirst = 0.9;
            agent.status = AgentStatus::Drinking;
        }
        w.tick();
    }

    #[test]
    fn consumption_reduces_inventory() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.inventory.add_resource("food", 5);
            agent.needs.hunger = 0.8;
            agent.status = AgentStatus::Eating;
        }
        w.tick();
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert_eq!(agent.inventory.resource_quantity("food"), 4);
    }

    #[test]
    fn critical_thirst_takes_priority() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.inventory.add_resource("water", 5);
            agent.needs.thirst = 0.9;
            agent.needs.hunger = 0.9;
            agent.needs.fatigue = 0.9;
        }
        w.tick();
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert_eq!(
            agent.status,
            AgentStatus::Drinking,
            "Critical thirst should trigger drinking first"
        );
    }

    #[test]
    fn sleep_when_exhausted_at_night() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        w.clock.tick = 23 * TICKS_PER_HOUR;
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.needs.fatigue = 0.9;
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
        }
        w.tick();
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Sleeping);
    }

    #[test]
    fn work_during_work_hours_if_employed() {
        let mut w = World::initialize(42, 2, 2);
        w.clock.tick = 10 * TICKS_PER_HOUR;
        let agent_id = "agent-0000".to_string();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.employer = Some("company-000".to_string());
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.needs.fatigue = 0.1;
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        w.tick();
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Working);
    }

    #[test]
    fn unemployed_agent_becomes_idle() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        w.clock.tick = 10 * TICKS_PER_HOUR;
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.employer = None;
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.needs.fatigue = 0.1;
        }
        w.tick();
        let agent = w.state.agents.get(&agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn work_improves_skill() {
        let mut w = World::initialize(42, 2, 2);
        w.clock.tick = 10 * TICKS_PER_HOUR;
        let agent_id = "agent-0000".to_string();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.employer = Some("company-000".to_string());
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.needs.fatigue = 0.1;
            agent.status = AgentStatus::Working;
        }
        let initial_skill = w
            .state
            .agents
            .get(&agent_id)
            .unwrap()
            .skills
            .get("productivity")
            .copied()
            .unwrap_or(0.0);
        w.tick();
        let final_skill = w
            .state
            .agents
            .get(&agent_id)
            .unwrap()
            .skills
            .get("productivity")
            .copied()
            .unwrap_or(0.0);
        assert!(
            final_skill > initial_skill,
            "skill should increase after work: {} -> {}",
            initial_skill,
            final_skill
        );
    }

    #[test]
    fn skill_never_exceeds_one() {
        let mut w = World::initialize(42, 2, 2);
        w.clock.tick = 10 * TICKS_PER_HOUR;
        let agent_id = "agent-0000".to_string();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.employer = Some("company-000".to_string());
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.needs.fatigue = 0.1;
            agent.status = AgentStatus::Working;
            agent.skills.insert("productivity".to_string(), 0.9999);
        }
        for _ in 0..1000 {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.needs.fatigue = 0.1;
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.status = AgentStatus::Working;
            w.clock.advance_tick();
            let tick = w.clock.tick;
            w.state.tick = tick;
            w.process_needs(tick);
            w.process_purchasing(tick);
            w.process_routines(tick, w.clock.is_night(), w.clock.is_work_hours());
        }
        let skill = w
            .state
            .agents
            .get(&agent_id)
            .unwrap()
            .skills
            .get("productivity")
            .copied()
            .unwrap_or(0.0);
        assert!(skill <= 1.0, "skill should not exceed 1.0: {}", skill);
    }

    #[test]
    fn skill_never_negative() {
        let w = World::initialize(42, 1, 0);
        for agent in w.state.agents.values() {
            for skill_val in agent.skills.values() {
                assert!(*skill_val >= 0.0);
            }
        }
    }

    // ─── PHASE 1: DETERMINISM TESTS ──────────────────────────────────

    #[test]
    fn deterministic_replay() {
        let mut w1 = World::initialize(1234, 5, 2);
        let mut w2 = World::initialize(1234, 5, 2);
        for _ in 0..100 {
            w1.tick();
            w2.tick();
        }
        assert_eq!(w1.tick_count(), w2.tick_count());
        for id in w1.state.agents.keys() {
            let a1 = w1.state.agents.get(id).unwrap();
            let a2 = w2.state.agents.get(id).unwrap();
            assert_eq!(a1.status, a2.status);
            assert!((a1.needs.hunger - a2.needs.hunger).abs() < 0.0001);
            assert!((a1.needs.thirst - a2.needs.thirst).abs() < 0.0001);
            assert!((a1.needs.fatigue - a2.needs.fatigue).abs() < 0.0001);
            assert_eq!(a1.inventory.resources, a2.inventory.resources);
        }
        assert_eq!(w1.state.events.len(), w2.state.events.len());
    }

    // ─── PHASE 1: CLOCK TESTS ────────────────────────────────────────

    #[test]
    fn hour_of_day_progresses() {
        let mut w = World::initialize(42, 1, 0);
        assert_eq!(w.clock.hour_of_day(), 0);
        w.advance(TICKS_PER_HOUR * 12);
        assert_eq!(w.clock.hour_of_day(), 12);
    }

    #[test]
    fn day_progresses() {
        let mut w = World::initialize(42, 1, 0);
        assert_eq!(w.clock.day(), 1);
        w.advance(TICKS_PER_DAY);
        assert_eq!(w.clock.day(), 2);
    }

    #[test]
    fn night_detected() {
        let mut w = World::initialize(42, 1, 0);
        w.clock.tick = 23 * TICKS_PER_HOUR;
        assert!(w.clock.is_night());
        w.clock.tick = 3 * TICKS_PER_HOUR;
        assert!(w.clock.is_night());
        w.clock.tick = 12 * TICKS_PER_HOUR;
        assert!(!w.clock.is_night());
    }

    #[test]
    fn work_hours_detected() {
        let mut w = World::initialize(42, 1, 0);
        w.clock.tick = 10 * TICKS_PER_HOUR;
        assert!(w.clock.is_work_hours());
        w.clock.tick = 20 * TICKS_PER_HOUR;
        assert!(!w.clock.is_work_hours());
    }

    #[test]
    fn calendar_year_month_week() {
        let mut w = World::initialize(42, 1, 0);
        assert_eq!(w.clock.year(), 1);
        assert_eq!(w.clock.month(), 1);
        assert_eq!(w.clock.day_of_week(), 0);
        assert_eq!(w.clock.day_of_month(), 1);
        assert_eq!(w.clock.minute_of_hour(), 0);

        w.advance(TICKS_PER_DAY * 30);
        assert_eq!(w.clock.month(), 2);
        assert_eq!(w.clock.day_of_month(), 1);

        w.advance(TICKS_PER_YEAR);
        assert_eq!(w.clock.year(), 2);
    }

    // ─── PHASE 1: EVENT TESTS ────────────────────────────────────────

    #[test]
    fn events_generated_during_tick() {
        let mut w = World::initialize(42, 1, 1);
        let agent_id = "agent-0000".to_string();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.needs.hunger = 0.8;
            agent.needs.thirst = 0.8;
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        let events_before = w.state.events.len();
        w.tick();
        let events_after = w.state.events.len();
        assert!(events_after > events_before);
    }

    #[test]
    fn status_change_generates_event() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.inventory.add_resource("water", 5);
            agent.needs.thirst = 0.9;
            agent.status = AgentStatus::Active;
        }
        w.tick();
        let has_status_event = w.state.events.iter().any(|e| {
            e.event_type == EventType::AgentDrank && e.actor.as_deref() == Some(&agent_id)
        });
        assert!(has_status_event, "Should have an AgentDrank event");
    }

    // ─── PHASE 3: INVARIANT TESTS ────────────────────────────────────

    #[test]
    fn agent_money_never_negative() {
        let mut w = World::initialize(42, 10, 3);
        w.advance(500);
        for agent in w.state.agents.values() {
            assert!(
                agent.money >= 0.0,
                "Agent {} has negative money: {}",
                agent.id,
                agent.money
            );
        }
    }

    #[test]
    fn company_cash_never_negative() {
        let mut w = World::initialize(42, 10, 3);
        w.advance(500);
        for company in w.state.companies.values() {
            assert!(
                company.cash >= 0.0,
                "Company {} has negative cash: {}",
                company.id,
                company.cash
            );
        }
    }

    #[test]
    fn inventory_never_negative() {
        let mut w = World::initialize(42, 10, 3);
        w.advance(500);
        for agent in w.state.agents.values() {
            for (res, qty) in &agent.inventory.resources {
                assert!(
                    *qty > 0 || !agent.inventory.has_resource(res),
                    "Agent {} has negative inventory for {}",
                    agent.id,
                    res
                );
            }
        }
    }

    // ─── PHASE 3: SOAK TESTS ─────────────────────────────────────────

    #[test]
    fn soak_test_7_days() {
        let mut w = World::initialize(1234, 50, 10);
        let ticks_7_days = TICKS_PER_DAY * 7;
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 30);
            agent.inventory.add_resource("water", 15);
        }
        for _ in 0..ticks_7_days {
            w.tick();
        }
        assert_eq!(w.tick_count(), ticks_7_days);
        assert_eq!(w.state.agents.len(), 50);
        assert_eq!(w.state.companies.len(), 10);
        let total_food: u32 = w
            .state
            .agents
            .values()
            .map(|a| a.inventory.resource_quantity("food"))
            .sum();
        assert!(total_food < 50 * 30, "Some food should have been consumed");
        assert!(!w.state.events.is_empty());
        for agent in w.state.agents.values() {
            assert!(agent.money >= 0.0);
            assert!(agent.needs.hunger >= 0.0 && agent.needs.hunger <= 1.0);
            assert!(agent.needs.thirst >= 0.0 && agent.needs.thirst <= 1.0);
            assert!(agent.needs.fatigue >= 0.0 && agent.needs.fatigue <= 1.0);
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let mut w = World::initialize(42, 3, 1);
        w.advance(10);
        let json = serialize_state(&w);
        let restored = deserialize_state(&json).unwrap();
        assert_eq!(restored.agents.len(), 3);
        assert_eq!(restored.companies.len(), 1);
    }

    // ─── PHASE 3: RESOURCE TESTS ────────────────────────────────────────

    #[test]
    fn resource_type_names() {
        assert_eq!(ResourceType::Food.name(), "food");
        assert_eq!(ResourceType::Water.name(), "water");
        assert_eq!(ResourceType::RawFood.name(), "raw_food");
    }

    #[test]
    fn resource_type_from_name() {
        assert_eq!(ResourceType::from_name("food"), Some(ResourceType::Food));
        assert_eq!(ResourceType::from_name("water"), Some(ResourceType::Water));
        assert_eq!(
            ResourceType::from_name("raw_food"),
            Some(ResourceType::RawFood)
        );
        assert_eq!(ResourceType::from_name("invalid"), None);
    }

    #[test]
    fn resource_inventory_add_remove() {
        let mut inv = ResourceInventory::new();
        inv.add(ResourceType::Food, 10).unwrap();
        assert_eq!(inv.quantity(ResourceType::Food), 10);
        inv.remove(ResourceType::Food, 3).unwrap();
        assert_eq!(inv.quantity(ResourceType::Food), 7);
    }

    #[test]
    fn resource_inventory_overflow_fails() {
        let mut inv = ResourceInventory::new();
        inv.add(ResourceType::Food, u32::MAX - 10).unwrap();
        let result = inv.add(ResourceType::Food, 20);
        assert!(result.is_err());
    }

    #[test]
    fn resource_inventory_remove_insufficient_fails() {
        let mut inv = ResourceInventory::new();
        inv.add(ResourceType::Food, 5).unwrap();
        let result = inv.remove(ResourceType::Food, 10);
        assert!(result.is_err());
        assert_eq!(inv.quantity(ResourceType::Food), 5);
    }

    // ─── PHASE 3: PRODUCTION TESTS ──────────────────────────────────────

    #[test]
    fn food_recipe_has_correct_structure() {
        let recipe = Recipe::food_production();
        assert_eq!(recipe.name, "food_production");
        assert_eq!(recipe.inputs.len(), 1);
        assert_eq!(recipe.inputs[0].resource, "raw_food");
        assert_eq!(recipe.inputs[0].quantity, RAW_FOOD_INPUT_PER_CYCLE);
        assert_eq!(recipe.outputs.len(), 1);
        assert_eq!(recipe.outputs[0].resource, "food");
        assert_eq!(recipe.outputs[0].quantity, FOOD_OUTPUT_PER_CYCLE);
        assert_eq!(recipe.labor_required, MIN_WORKERS_FOR_PRODUCTION);
    }

    #[test]
    fn production_consumes_inputs() {
        let recipe = Recipe::food_production();
        let mut inventory = HashMap::new();
        inventory.insert("raw_food".to_string(), 100);
        let result = execute_production(&recipe, &mut inventory, 0.5);
        assert!(result.is_ok());
        assert_eq!(inventory.get("raw_food").copied().unwrap_or(0), 90);
        assert!(inventory.get("food").copied().unwrap_or(0) > 0);
    }

    #[test]
    fn production_fails_without_inputs() {
        let recipe = Recipe::food_production();
        let mut inventory = HashMap::new();
        inventory.insert("raw_food".to_string(), 5);
        let result = execute_production(&recipe, &mut inventory, 0.5);
        assert!(result.is_err());
        assert_eq!(inventory.get("raw_food").copied().unwrap_or(0), 5);
    }

    #[test]
    fn production_fails_without_workers() {
        let recipe = Recipe::food_production();
        let mut inventory = HashMap::new();
        inventory.insert("raw_food".to_string(), 100);
        assert!(!can_produce(&recipe, &inventory, 0, 0));
    }

    #[test]
    fn production_fails_with_cooldown() {
        let recipe = Recipe::food_production();
        let mut inventory = HashMap::new();
        inventory.insert("raw_food".to_string(), 100);
        assert!(!can_produce(&recipe, &inventory, 1, 1));
    }

    #[test]
    fn productivity_multiplier_increases_with_skill() {
        let low = compute_productivity_multiplier(0.0);
        let mid = compute_productivity_multiplier(0.5);
        let high = compute_productivity_multiplier(1.0);
        assert!(low < mid);
        assert!(mid < high);
        assert!(high <= 1.5);
    }

    #[test]
    fn company_has_production_recipe() {
        let w = World::initialize(42, 5, 5);
        for company in w.state.companies.values() {
            if company.company_type == CompanyType::FoodProducer {
                assert!(company.recipe_name.is_some());
                assert_eq!(company.recipe_name.as_deref(), Some("food_production"));
                assert!(company.inventory.get("raw_food").copied().unwrap_or(0) > 0);
            }
        }
    }

    // ─── PHASE 3: MARKET TESTS ──────────────────────────────────────────

    #[test]
    fn market_has_food_listing_after_tick() {
        let mut w = World::initialize(42, 5, 5);
        let ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        w.clock.tick = 10 * TICKS_PER_HOUR;
        w.tick();
        let food_listings: Vec<_> = w
            .state
            .markets
            .listings
            .iter()
            .filter(|l| l.resource == "food" && l.quantity > 0)
            .collect();
        assert!(
            !food_listings.is_empty(),
            "Should have food listings after tick"
        );
    }

    #[test]
    fn market_price_positive() {
        let w = World::initialize(42, 5, 5);
        for price in w.state.markets.prices.values() {
            assert!(*price > 0.0, "All prices should be positive");
        }
    }

    #[test]
    fn market_listing_price_bounded() {
        let mut w = World::initialize(42, 5, 5);
        let ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        w.clock.tick = 10 * TICKS_PER_HOUR;
        for _ in 0..50 {
            w.tick();
        }
        for listing in &w.state.markets.listings {
            let min = if listing.resource == "water" {
                WATER_PRICE_MIN
            } else {
                FOOD_PRICE_MIN
            };
            let max = if listing.resource == "water" {
                WATER_PRICE_MAX
            } else {
                FOOD_PRICE_MAX
            };
            assert!(
                listing.price >= min,
                "Listing price for {} should be >= min: {}",
                listing.resource,
                listing.price
            );
            assert!(
                listing.price <= max,
                "Listing price for {} should be <= max: {}",
                listing.resource,
                listing.price
            );
        }
    }

    // ─── PHASE 3: ECONOMY TESTS ─────────────────────────────────────────

    #[test]
    fn agent_food_purchasing_decision() {
        assert!(agent_wants_food(0.8, 1));
        assert!(!agent_wants_food(0.2, 5));
        assert!(!agent_wants_food(0.8, 10));
    }

    #[test]
    fn food_quantity_computation() {
        let qty = compute_desired_food_quantity(0.8, 0);
        assert!(qty > 0);
        assert!(qty <= 3);
        let qty_zero = compute_desired_food_quantity(0.2, 5);
        assert_eq!(qty_zero, 0);
    }

    #[test]
    fn price_adjustment_increases_with_demand() {
        let p1 = adjust_price(2.0, 0, 100, 0.0);
        let p2 = adjust_price(2.0, 10, 10, 0.8);
        assert!(p2 > p1);
    }

    #[test]
    fn price_adjustment_decreases_with_high_supply() {
        let p1 = adjust_price(2.0, 0, 10, 0.0);
        let p2 = adjust_price(2.0, 0, 200, 0.0);
        assert!(p2 < p1);
    }

    #[test]
    fn price_always_positive_and_bounded() {
        let mut p = FOOD_BASE_PRICE;
        for i in 0..1000 {
            let supply = (i as u32) % 200;
            let demand = ((1000 - i) as f64) / 500.0;
            p = adjust_price(p, 0, supply, demand);
            assert!(
                p >= FOOD_PRICE_MIN,
                "Price {} below minimum at tick {}",
                p,
                i
            );
            assert!(
                p <= FOOD_PRICE_MAX,
                "Price {} above maximum at tick {}",
                p,
                i
            );
        }
    }

    #[test]
    fn money_conservation_over_ticks() {
        let mut w = World::initialize(1234, 10, 5);
        let ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }
        let initial_total = {
            let agents_money: f64 = w.state.agents.values().map(|a| a.money).sum();
            let companies_cash: f64 = w.state.companies.values().map(|c| c.cash).sum();
            agents_money + companies_cash
        };
        w.clock.tick = 10 * TICKS_PER_HOUR;
        w.advance(100);
        let final_total = {
            let agents_money: f64 = w.state.agents.values().map(|a| a.money).sum();
            let companies_cash: f64 = w.state.companies.values().map(|c| c.cash).sum();
            agents_money + companies_cash
        };
        assert!(
            (initial_total - final_total).abs() < 0.01,
            "Money should be conserved: initial={}, final={}",
            initial_total,
            final_total
        );
    }

    #[test]
    fn company_revenue_from_transactions() {
        let w = World::initialize(42, 5, 5);
        let company_id = "company-000".to_string();
        let company = w.state.companies.get(&company_id).unwrap();
        assert_eq!(company.revenue, 0.0);
    }

    #[test]
    fn wage_payment_reduces_company_cash() {
        let mut w = World::initialize(42, 5, 5);
        let company_id = "company-000".to_string();
        let agent_id = "agent-0000".to_string();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.needs.fatigue = 0.1;
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        // Genesis already created employment for agent-0000 at company-000
        // Ensure employment record and agent.employer are aligned
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.employer = Some(company_id.clone());
        }
        let initial_cash = w.state.companies.get(&company_id).unwrap().cash;
        w.clock.tick = 10 * TICKS_PER_HOUR - 1;
        w.tick();
        let final_cash = w.state.companies.get(&company_id).unwrap().cash;
        assert!(
            final_cash < initial_cash,
            "Company cash should decrease from wages: initial={}, final={}",
            initial_cash,
            final_cash
        );
    }

    #[test]
    fn wage_payment_increases_agent_money() {
        let mut w = World::initialize(42, 5, 5);
        let agent_id = "agent-0000".to_string();
        let company_id = "company-000".to_string();
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.employer = Some(company_id);
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.needs.fatigue = 0.1;
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        let initial_money = w.state.agents.get(&agent_id).unwrap().money;
        w.clock.tick = 10 * TICKS_PER_HOUR - 1;
        w.tick();
        let final_money = w.state.agents.get(&agent_id).unwrap().money;
        assert!(
            final_money > initial_money,
            "Agent money should increase from wages"
        );
    }

    #[test]
    fn no_negative_inventory_after_ticks() {
        let mut w = World::initialize(42, 10, 5);
        let ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }
        w.clock.tick = 10 * TICKS_PER_HOUR;
        w.advance(200);
        for agent in w.state.agents.values() {
            for (res, qty) in &agent.inventory.resources {
                assert!(
                    *qty > 0 || !agent.inventory.has_resource(res),
                    "Agent {} has negative inventory for {}",
                    agent.id,
                    res
                );
            }
        }
        for company in w.state.companies.values() {
            for (_res, qty) in &company.inventory {
                let _ = qty;
            }
        }
    }

    #[test]
    fn no_negative_money_after_ticks() {
        let mut w = World::initialize(42, 10, 5);
        let ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }
        w.clock.tick = 10 * TICKS_PER_HOUR;
        w.advance(200);
        for agent in w.state.agents.values() {
            assert!(
                agent.money >= 0.0,
                "Agent {} has negative money: {}",
                agent.id,
                agent.money
            );
        }
        for company in w.state.companies.values() {
            assert!(
                company.cash >= 0.0,
                "Company {} has negative cash: {}",
                company.id,
                company.cash
            );
        }
    }

    #[test]
    fn genesis_food_economy_runs() {
        let mut w = World::initialize(1234, 50, 10);
        let ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        w.clock.tick = 9 * TICKS_PER_HOUR;
        w.advance(168);
        let food_produced: u32 = w
            .state
            .events
            .iter()
            .filter(|e| e.event_type == EventType::ProductionCompleted)
            .count() as u32;
        assert!(
            food_produced > 0,
            "Food should be produced in Genesis economy"
        );
        let food_purchased: u32 = w
            .state
            .events
            .iter()
            .filter(|e| e.event_type == EventType::FoodPurchased)
            .count() as u32;
        assert!(
            food_purchased > 0,
            "Food should be purchased in Genesis economy"
        );
        let total_agents_money: f64 = w.state.agents.values().map(|a| a.money).sum();
        let total_companies_cash: f64 = w.state.companies.values().map(|c| c.cash).sum();
        let total_money = total_agents_money + total_companies_cash;
        assert!(total_money > 0.0, "Total money should be positive");
        for agent in w.state.agents.values() {
            assert!(agent.money >= 0.0);
        }
        for company in w.state.companies.values() {
            assert!(company.cash >= 0.0);
        }
    }

    #[test]
    fn deterministic_replay_phase3() {
        let mut w1 = World::initialize(1234, 10, 5);
        let mut w2 = World::initialize(1234, 10, 5);
        let ids1: Vec<String> = w1.state.agents.keys().cloned().collect();
        for id in &ids1 {
            let agent = w1.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 10);
            agent.inventory.add_resource("water", 5);
        }
        let ids2: Vec<String> = w2.state.agents.keys().cloned().collect();
        for id in &ids2 {
            let agent = w2.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 10);
            agent.inventory.add_resource("water", 5);
        }
        w1.clock.tick = 9 * TICKS_PER_HOUR;
        w2.clock.tick = 9 * TICKS_PER_HOUR;
        for _ in 0..168 {
            w1.tick();
            w2.tick();
        }
        assert_eq!(w1.tick_count(), w2.tick_count());
        for id in w1.state.agents.keys() {
            let a1 = w1.state.agents.get(id).unwrap();
            let a2 = w2.state.agents.get(id).unwrap();
            assert!(
                (a1.money - a2.money).abs() < 0.0001,
                "Agent {} money differs: {} vs {}",
                id,
                a1.money,
                a2.money
            );
            assert!((a1.needs.hunger - a2.needs.hunger).abs() < 0.0001);
            assert!((a1.needs.thirst - a2.needs.thirst).abs() < 0.0001);
            assert_eq!(a1.inventory.resources, a2.inventory.resources);
        }
        assert_eq!(w1.state.events.len(), w2.state.events.len());
    }

    #[test]
    fn soak_test_30_days() {
        let mut w = World::initialize(5678, 50, 10);
        let ticks_30_days = TICKS_PER_DAY * 30;
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 30);
            agent.inventory.add_resource("water", 15);
        }
        let initial_total_money = {
            let agents_money: f64 = w.state.agents.values().map(|a| a.money).sum();
            let companies_cash: f64 = w.state.companies.values().map(|c| c.cash).sum();
            agents_money + companies_cash
        };
        for _ in 0..ticks_30_days {
            w.tick();
        }
        assert_eq!(w.tick_count(), ticks_30_days);
        assert_eq!(w.state.agents.len(), 50);
        assert_eq!(w.state.companies.len(), 10);
        let total_food: u32 = w
            .state
            .agents
            .values()
            .map(|a| a.inventory.resource_quantity("food"))
            .sum();
        assert!(total_food < 50 * 10, "Some food should have been consumed");
        assert!(!w.state.events.is_empty());
        for agent in w.state.agents.values() {
            assert!(agent.money >= 0.0);
            assert!(agent.needs.hunger >= 0.0 && agent.needs.hunger <= 1.0);
            assert!(agent.needs.thirst >= 0.0 && agent.needs.thirst <= 1.0);
            assert!(agent.needs.fatigue >= 0.0 && agent.needs.fatigue <= 1.0);
        }
        for company in w.state.companies.values() {
            assert!(company.cash >= 0.0);
        }
        let final_total_money = {
            let agents_money: f64 = w.state.agents.values().map(|a| a.money).sum();
            let companies_cash: f64 = w.state.companies.values().map(|c| c.cash).sum();
            agents_money + companies_cash
        };
        assert!(
            (initial_total_money - final_total_money).abs() < 0.01,
            "Money must be conserved: initial={:.2} final={:.2}",
            initial_total_money,
            final_total_money
        );
        let has_production = w
            .state
            .events
            .iter()
            .any(|e| e.event_type == EventType::ProductionCompleted);
        assert!(
            has_production,
            "Production should occur in 30-day soak"
        );
        let employed = w.state.labor_market.active_employments().len();
        assert!(employed > 0, "Some agents should be employed");
    }

    #[test]
    fn deterministic_replay_phase4() {
        let mut w1 = World::initialize(1234, 10, 5);
        let mut w2 = World::initialize(1234, 10, 5);
        let ids1: Vec<String> = w1.state.agents.keys().cloned().collect();
        for id in &ids1 {
            let agent = w1.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 10);
            agent.inventory.add_resource("water", 5);
        }
        let ids2: Vec<String> = w2.state.agents.keys().cloned().collect();
        for id in &ids2 {
            let agent = w2.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 10);
            agent.inventory.add_resource("water", 5);
        }
        w1.clock.tick = 9 * TICKS_PER_HOUR;
        w2.clock.tick = 9 * TICKS_PER_HOUR;
        for _ in 0..TICKS_PER_DAY * 7 {
            w1.tick();
            w2.tick();
        }
        assert_eq!(w1.tick_count(), w2.tick_count());
        for id in w1.state.agents.keys() {
            let a1 = w1.state.agents.get(id).unwrap();
            let a2 = w2.state.agents.get(id).unwrap();
            assert!(
                (a1.money - a2.money).abs() < 0.0001,
                "Agent {} money differs: {} vs {}",
                id,
                a1.money,
                a2.money
            );
            assert!((a1.needs.hunger - a2.needs.hunger).abs() < 0.0001);
            assert!((a1.needs.thirst - a2.needs.thirst).abs() < 0.0001);
            assert_eq!(a1.inventory.resources, a2.inventory.resources);
        }
        assert_eq!(w1.state.events.len(), w2.state.events.len());
    }

    // ─── PHASE 5: EMPLOYMENT TESTS ──────────────────────────────────────

    #[test]
    fn genesis_creates_employment_records() {
        let w = World::initialize(1234, 50, 10);
        let total_employments = w.state.labor_market.active_employments().len();
        assert!(
            total_employments > 0,
            "Genesis should create employment records"
        );
    }

    #[test]
    fn genesis_has_unemployed_agents() {
        let w = World::initialize(1234, 50, 10);
        let employed = w.state.labor_market.active_employments().len();
        let total = w.state.agents.len();
        assert!(
            employed < total,
            "Genesis should leave some agents unemployed: employed={}, total={}",
            employed,
            total
        );
    }

    #[test]
    fn agent_cannot_be_employed_twice() {
        let mut w = World::initialize(42, 10, 3);
        w.clock.tick = 10 * TICKS_PER_HOUR;
        let agent_id = "agent-0000".to_string();
        let company_id = "company-000".to_string();
        let company2_id = "company-001".to_string();

        // Terminate existing bootstrap employment first
        let _ = w
            .state
            .labor_market
            .terminate_all_for_company(&company_id);

        let hired1 = w.state.labor_market.create_employment(
            agent_id.clone(),
            company_id,
            "Worker".to_string(),
            10.0,
            0,
        );
        assert!(hired1);

        let hired2 = w.state.labor_market.create_employment(
            agent_id.clone(),
            company2_id,
            "Worker".to_string(),
            12.0,
            0,
        );
        assert!(!hired2, "Agent should not be hired at two companies");

        assert_eq!(
            w.state.labor_market.employments_for_agent(&agent_id).len(),
            1
        );
    }

    #[test]
    fn firing_releases_agent() {
        let mut w = World::initialize(42, 10, 3);
        let agent_id = "agent-0000".to_string();
        let company_id = "company-000".to_string();

        // Terminate existing bootstrap employment first
        let _ = w
            .state
            .labor_market
            .terminate_all_for_company(&company_id);

        let hired = w.state.labor_market.create_employment(
            agent_id.clone(),
            company_id.clone(),
            "Worker".to_string(),
            10.0,
            0,
        );
        assert!(hired);
        assert!(w.state.labor_market.is_agent_employed(&agent_id));

        let fired = w
            .state
            .labor_market
            .terminate_employment(&agent_id, &company_id);
        assert!(fired);
        assert!(!w.state.labor_market.is_agent_employed(&agent_id));
    }

    #[test]
    fn terminate_all_for_company() {
        let mut w = World::initialize(42, 10, 3);
        let company_id = "company-000".to_string();

        let initial_count = w
            .state
            .labor_market
            .employee_count_for_company(&company_id);

        // Terminate all existing employees
        let fired = w
            .state
            .labor_market
            .terminate_all_for_company(&company_id);
        assert_eq!(fired.len(), initial_count);
        assert_eq!(
            w.state.labor_market.employee_count_for_company(&company_id),
            0
        );
    }

    #[test]
    fn job_openings_created_for_understaffed_companies() {
        let mut w = World::initialize(42, 10, 5);
        w.clock.tick = 10 * TICKS_PER_HOUR - 1;
        for _ in 0..5 {
            w.tick();
        }
        let has_openings = !w.state.labor_market.job_openings.is_empty();
        assert!(
            has_openings,
            "Understaffed companies should create job openings"
        );
    }

    #[test]
    fn unemployed_agent_can_find_job() {
        let mut w = World::initialize(42, 30, 3);
        w.clock.tick = 10 * TICKS_PER_HOUR;

        let agent_count = w.state.agents.len();
        let employed_before = w.state.labor_market.active_employments().len();

        for _ in 0..10 {
            w.tick();
        }

        let employed_after = w.state.labor_market.active_employments().len();
        assert!(
            employed_after > employed_before || employed_after == agent_count,
            "Employment should grow or reach full employment: before={}, after={}, total={}",
            employed_before,
            employed_after,
            agent_count
        );
    }

    #[test]
    fn wages_paid_only_to_employed_agents() {
        let mut w = World::initialize(42, 10, 3);
        let agent_id = w
            .state
            .agents
            .keys()
            .find(|id| !w.state.labor_market.is_agent_employed(id))
            .cloned()
            .expect("Should have at least one unemployed agent");

        let initial_money = w.state.agents.get(&agent_id).unwrap().money;
        w.clock.tick = 10 * TICKS_PER_HOUR;
        w.tick();
        let final_money = w.state.agents.get(&agent_id).unwrap().money;
        let still_unemployed = !w.state.labor_market.is_agent_employed(&agent_id);
        if still_unemployed {
            assert!(
                (final_money - initial_money).abs() < 0.01,
                "Unemployed agent should not receive wages: initial={}, final={}",
                initial_money,
                final_money
            );
        }
    }

    #[test]
    fn company_profit_loss_tracking() {
        let mut w = World::initialize(1234, 30, 5);
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }
        w.clock.tick = 9 * TICKS_PER_HOUR;
        for _ in 0..TICKS_PER_DAY * 7 {
            w.tick();
        }

        let mut any_profit = false;
        let mut any_loss = false;
        for company in w.state.companies.values() {
            if company.cumulative_profit > 0.0 {
                any_profit = true;
            }
            if company.cumulative_profit < 0.0 {
                any_loss = true;
            }
        }
        assert!(
            any_profit || any_loss,
            "Some company should have cumulative profit or loss"
        );
    }

    #[test]
    fn company_workforce_grows_on_profit() {
        let mut w = World::initialize(1234, 30, 5);
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }
        w.clock.tick = 9 * TICKS_PER_HOUR;

        let initial_desired: Vec<(String, usize)> = w
            .state
            .companies
            .iter()
            .filter(|(_, c)| c.active)
            .map(|(id, c)| (id.clone(), c.desired_workforce))
            .collect();

        for _ in 0..TICKS_PER_DAY * 30 {
            w.tick();
        }

        let mut any_growth = false;
        for (id, initial) in &initial_desired {
            if let Some(company) = w.state.companies.get(id) {
                if company.desired_workforce > *initial {
                    any_growth = true;
                    break;
                }
            }
        }
        assert!(
            any_growth,
            "At least one company should grow workforce over 30 days"
        );
    }

    #[test]
    fn labor_affects_production() {
        let mut w = World::initialize(42, 30, 5);
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }

        // Count employments in food-producing companies
        let food_companies: Vec<String> = w
            .state
            .companies
            .iter()
            .filter(|(_, c)| c.company_type == CompanyType::FoodProducer && c.active)
            .map(|(id, _)| id.clone())
            .collect();

        let total_food_workers: usize = food_companies
            .iter()
            .map(|cid| w.state.labor_market.employee_count_for_company(cid))
            .sum();

        assert!(
            total_food_workers > 0,
            "Should have food company workers from genesis"
        );
    }

    #[test]
    fn company_insolvency_closes_company() {
        let mut w = World::initialize(42, 5, 3);
        let company_id = "company-000".to_string();

        {
            let company = w.state.companies.get_mut(&company_id).unwrap();
            company.cash = 10.0;
            company.loss_streak = COMPANY_LOSS_STREAK_TO_FIRE + 1;
            company.active = true;
        }

        for i in 0..3 {
            let agent_id = format!("agent-{:04}", i);
            let _ = w.state.labor_market.create_employment(
                agent_id.clone(),
                company_id.clone(),
                "Worker".to_string(),
                10.0,
                0,
            );
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.employer = Some(company_id.clone());
        }

        w.clock.tick = 10 * TICKS_PER_HOUR - 1;
        w.tick();

        let company = w.state.companies.get(&company_id).unwrap();
        assert!(
            !company.active,
            "Company should be inactive after insolvency"
        );
        assert_eq!(
            w.state.labor_market.employee_count_for_company(&company_id),
            0,
            "All employees should be released"
        );
    }

    #[test]
    fn wages_cannot_create_money() {
        let mut w = World::initialize(42, 10, 3);
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }

        let initial_total = {
            let agents_money: f64 = w.state.agents.values().map(|a| a.money).sum();
            let companies_cash: f64 = w.state.companies.values().map(|c| c.cash).sum();
            agents_money + companies_cash
        };

        w.clock.tick = 10 * TICKS_PER_HOUR;
        for _ in 0..100 {
            w.tick();
        }

        let final_total = {
            let agents_money: f64 = w.state.agents.values().map(|a| a.money).sum();
            let companies_cash: f64 = w.state.companies.values().map(|c| c.cash).sum();
            agents_money + companies_cash
        };

        assert!(
            (initial_total - final_total).abs() < 0.01,
            "Money must be conserved: initial={}, final={}",
            initial_total,
            final_total
        );
    }

    #[test]
    fn labor_market_metrics() {
        let w = World::initialize(1234, 50, 10);
        let total = w.state.agents.len();
        let employed = w.state.labor_market.active_employments().len();
        let unemployed = w.state.labor_market.unemployed_count(total);
        let _open = w.state.labor_market.open_positions();
        let avg_wage = w.state.labor_market.average_wage();

        assert!(employed > 0, "Should have employed agents");
        assert!(unemployed > 0, "Should have unemployed agents");
        assert_eq!(employed + unemployed, total);
        assert!(avg_wage >= 0.0);
    }

    #[test]
    fn hiring_event_emitted() {
        let mut w = World::initialize(42, 20, 5);
        w.clock.tick = 10 * TICKS_PER_HOUR;
        let events_before = w.state.events.len();
        for _ in 0..5 {
            w.tick();
        }
        let hire_events: Vec<_> = w.state.events[events_before..]
            .iter()
            .filter(|e| e.event_type == EventType::AgentHired)
            .collect();
        assert!(
            !hire_events.is_empty(),
            "Should emit AgentHired events during hiring"
        );
    }

    #[test]
    fn job_posted_event_emitted() {
        let mut w = World::initialize(42, 20, 5);
        w.clock.tick = 10 * TICKS_PER_HOUR;
        let events_before = w.state.events.len();
        for _ in 0..5 {
            w.tick();
        }
        let job_events: Vec<_> = w.state.events[events_before..]
            .iter()
            .filter(|e| e.event_type == EventType::JobPosted)
            .collect();
        assert!(
            !job_events.is_empty(),
            "Should emit JobPosted events for understaffed companies"
        );
    }

    #[test]
    fn company_cash_never_negative_after_long_run() {
        let mut w = World::initialize(1234, 50, 10);
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 30);
            agent.inventory.add_resource("water", 15);
        }
        for _ in 0..TICKS_PER_DAY * 7 {
            w.tick();
        }
        for company in w.state.companies.values() {
            assert!(
                company.cash >= 0.0,
                "Company {} has negative cash: {}",
                company.id,
                company.cash
            );
        }
    }

    #[test]
    fn production_event_emitted() {
        let mut w = World::initialize(1234, 30, 5);
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }
        w.clock.tick = 9 * TICKS_PER_HOUR;
        for _ in 0..TICKS_PER_DAY {
            w.tick();
        }
        let prod_events: Vec<_> = w
            .state
            .events
            .iter()
            .filter(|e| e.event_type == EventType::ProductionCompleted)
            .collect();
        assert!(!prod_events.is_empty(), "Should have production events");
    }

    #[test]
    fn wage_payment_event_emitted() {
        let mut w = World::initialize(1234, 30, 5);
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 20);
            agent.inventory.add_resource("water", 10);
        }
        w.clock.tick = 10 * TICKS_PER_HOUR - 1;
        for _ in 0..10 {
            w.tick();
        }
        let wage_events: Vec<_> = w
            .state
            .events
            .iter()
            .filter(|e| e.event_type == EventType::WagePaid)
            .collect();
        assert!(!wage_events.is_empty(), "Should have wage payment events");
    }

    #[test]
    fn labor_market_highest_paying_company() {
        let w = World::initialize(42, 20, 5);
        let best = w.state.labor_market.highest_paying_company();
        assert!(best.is_some(), "Should have a highest paying company");
        let (_, wage) = best.unwrap();
        assert!(wage > 0.0, "Highest wage should be positive");
    }

    #[test]
    fn labor_market_largest_employer() {
        let w = World::initialize(42, 20, 5);
        let best = w.state.labor_market.largest_employer();
        assert!(best.is_some(), "Should have a largest employer");
        let (_, count) = best.unwrap();
        assert!(count > 0, "Largest employer should have employees");
    }

    #[test]
    fn genesis_employment_consistent() {
        let w = World::initialize(1234, 50, 10);

        // Every employed agent should have employer set
        for emp in w.state.labor_market.active_employments() {
            let agent = w.state.agents.get(&emp.agent_id).unwrap();
            assert_eq!(
                agent.employer.as_deref(),
                Some(emp.company_id.as_str()),
                "Agent {} employer should match employment record",
                emp.agent_id
            );
        }

        // Companies with employees should have them in the employees HashMap
        for company in w.state.companies.values() {
            for emp_id in company.employees.keys() {
                assert!(
                    w.state
                        .labor_market
                        .is_agent_employed_at(emp_id, &company.id),
                    "Company {} employees HashMap should match employment records",
                    company.id
                );
            }
        }
    }

    // ─── PHASE 6: PROPERTY, BUILDINGS & CONSTRUCTION TESTS ─────────

    #[test]
    fn genesis_creates_parcels() {
        let w = World::initialize(42, 10, 3);
        assert!(!w.state.parcels.is_empty(), "Genesis should create parcels");
        let total = w.state.parcels.len();
        assert_eq!(total, 5 * PARCELS_PER_DISTRICT);
        for parcel in w.state.parcels.values() {
            assert!(!parcel.id.is_empty());
            assert!(!parcel.district_id.is_empty());
            assert!(w.state.city.districts.contains_key(&parcel.district_id));
        }
    }

    #[test]
    fn parcels_belong_to_districts() {
        let w = World::initialize(42, 10, 3);
        for parcel in w.state.parcels.values() {
            let district = w.state.city.districts.get(&parcel.district_id);
            assert!(district.is_some(), "Parcel district should exist");
            assert!(
                district.unwrap().parcel_ids.contains(&parcel.id),
                "District should reference parcel"
            );
        }
    }

    #[test]
    fn zoning_exists() {
        let w = World::initialize(42, 10, 3);
        let mut has_r = false;
        let mut has_c = false;
        let mut has_i = false;
        for parcel in w.state.parcels.values() {
            match parcel.zone {
                Zone::Residential => has_r = true,
                Zone::Commercial => has_c = true,
                Zone::Industrial => has_i = true,
                _ => {}
            }
        }
        assert!(has_r, "Should have residential parcels");
        assert!(has_c, "Should have commercial parcels");
        assert!(has_i, "Should have industrial parcels");
    }

    #[test]
    fn genesis_creates_buildings() {
        let w = World::initialize(42, 10, 3);
        assert!(!w.state.buildings.is_empty(), "Genesis should create buildings");
        for building in w.state.buildings.values() {
            assert!(building.operational, "Genesis buildings should be operational");
            assert_eq!(building.construction_state, ConstructionState::Completed);
            assert!(w.state.parcels.contains_key(&building.parcel_id));
        }
    }

    #[test]
    fn buildings_occupy_real_parcels() {
        let w = World::initialize(42, 10, 3);
        for building in w.state.buildings.values() {
            let parcel = w.state.parcels.get(&building.parcel_id).unwrap();
            assert!(parcel.occupied, "Parcel should be occupied");
            assert_eq!(parcel.building_id.as_ref(), Some(&building.id));
        }
    }

    #[test]
    fn buildings_have_useful_capacity() {
        let w = World::initialize(42, 10, 3);
        for building in w.state.buildings.values() {
            assert!(building.capacity > 0, "Building should have capacity");
        }
    }

    #[test]
    fn company_has_building_and_capacity() {
        let w = World::initialize(42, 10, 3);
        for company in w.state.companies.values() {
            if company.active {
                assert!(
                    !company.building_ids.is_empty(),
                    "Active company {} should have buildings",
                    company.id
                );
                assert!(
                    company.total_capacity > 0,
                    "Active company {} should have capacity",
                    company.id
                );
            }
        }
    }

    #[test]
    fn genesis_creates_properties() {
        let w = World::initialize(42, 10, 3);
        assert!(!w.state.properties.is_empty(), "Genesis should create properties");
        for property in w.state.properties.values() {
            assert!(property.value > 0.0, "Property should have value");
        }
    }

    #[test]
    fn property_owner_matches_building_owner() {
        let w = World::initialize(42, 10, 3);
        for property in w.state.properties.values() {
            if let Some(building_id) = &property.building_id {
                let building = w.state.buildings.get(building_id).unwrap();
                assert_eq!(
                    format!("{:?}", property.owner),
                    format!("{:?}", building.owner),
                    "Property and building owners should match"
                );
            }
        }
    }

    #[test]
    fn construction_recipes_exist() {
        let w = World::initialize(42, 10, 3);
        assert!(!w.state.construction_recipes.is_empty(), "Should have recipes");
        for rt in &["house", "apartment", "factory", "farm", "shop", "office", "warehouse", "water_plant"] {
            assert!(w.state.construction_recipes.contains_key(*rt), "Missing recipe: {}", rt);
        }
    }

    #[test]
    fn construction_recipe_costs_positive() {
        let w = World::initialize(42, 10, 3);
        for recipe in w.state.construction_recipes.values() {
            assert!(recipe.money_cost > 0.0, "Recipe should have money cost");
            assert!(recipe.duration_ticks > 0, "Recipe should have duration");
            assert!(recipe.capacity > 0, "Recipe should have capacity");
        }
    }

    #[test]
    fn zone_building_compatibility() {
        assert!(zone_compatible_building(&Zone::Residential, &BuildingType::House));
        assert!(zone_compatible_building(&Zone::Residential, &BuildingType::Apartment));
        assert!(!zone_compatible_building(&Zone::Residential, &BuildingType::Factory));
        assert!(zone_compatible_building(&Zone::Commercial, &BuildingType::Shop));
        assert!(!zone_compatible_building(&Zone::Commercial, &BuildingType::House));
        assert!(zone_compatible_building(&Zone::Industrial, &BuildingType::Factory));
        assert!(!zone_compatible_building(&Zone::Industrial, &BuildingType::Shop));
    }

    #[test]
    fn manual_construction_flow() {
        let mut w = World::initialize(42, 10, 3);
        let cid = "company-000".to_string();
        let district_id = w.state.companies.get(&cid).unwrap().location.clone();
        let suitable = w.state.parcels.values()
            .filter(|p| {
                p.district_id == district_id
                    && p.owner == PropertyOwnership::Unowned
                    && zone_compatible_building(&p.zone, &BuildingType::Shop)
            })
            .min_by(|a, b| a.id.cmp(&b.id))
            .cloned();
        let parcel = suitable.expect("Should find suitable parcel");
        let recipe = w.state.construction_recipes.get("shop").unwrap().clone();
        {
            let company = w.state.companies.get_mut(&cid).unwrap();
            company.cash -= recipe.money_cost;
            company.last_expansion_tick = 0;
        }
        let project_id = "project-test-0".to_string();
        w.state.construction_projects.insert(project_id.clone(), ConstructionProject {
            id: project_id.clone(),
            owner: cid.clone(),
            parcel_id: parcel.id.clone(),
            target_building: BuildingType::Shop,
            required_resources: recipe.resource_cost.clone(),
            reserved_resources: recipe.resource_cost.clone(),
            labor_required: recipe.labor_hours,
            total_duration: recipe.duration_ticks,
            progress: 0,
            total_cost: recipe.money_cost,
            paid_cost: recipe.money_cost,
            status: ConstructionState::UnderConstruction,
        });
        for _ in 0..recipe.duration_ticks + 100 {
            w.tick();
        }
        let proj = w.state.construction_projects.get(&project_id).unwrap();
        assert_eq!(proj.status, ConstructionState::Completed);
        let new_buildings: Vec<&Building> = w.state.buildings.values()
            .filter(|b| b.building_type == BuildingType::Shop && b.operator.as_ref() == Some(&cid))
            .collect();
        assert!(!new_buildings.is_empty(), "Company should have new building");
    }

    #[test]
    fn money_conservation_with_construction() {
        let mut w = World::initialize(42, 10, 3);
        let initial_total = {
            let a: f64 = w.state.agents.values().map(|a| a.money).sum();
            let c: f64 = w.state.companies.values().map(|c| c.cash).sum();
            a + c
        };
        for _ in 0..1008 {
            w.tick();
        }
        let final_total = {
            let a: f64 = w.state.agents.values().map(|a| a.money).sum();
            let c: f64 = w.state.companies.values().map(|c| c.cash).sum();
            a + c
        };
        assert!(
            (initial_total - final_total).abs() < 0.01,
            "Money must be conserved: initial={:.2} final={:.2}",
            initial_total,
            final_total
        );
    }

    #[test]
    fn housing_assignment() {
        let mut w = World::initialize(42, 20, 3);
        for _ in 0..120 {
            w.tick();
        }
        let housed = w.state.agents.values().filter(|a| a.housing_id.is_some()).count();
        assert!(housed > 0, "Some agents should be housed");
    }

    #[test]
    fn housing_occupancy_bounded() {
        let mut w = World::initialize(42, 20, 3);
        for _ in 0..120 {
            w.tick();
        }
        for building in w.state.buildings.values() {
            if matches!(building.building_type, BuildingType::House | BuildingType::Apartment) {
                let occupants = w.state.agents.values()
                    .filter(|a| a.housing_id.as_ref() == Some(&building.id))
                    .count();
                assert!(
                    (occupants as u32) <= building.capacity,
                    "Building {} occupancy {} exceeds capacity {}",
                    building.id,
                    occupants,
                    building.capacity
                );
            }
        }
    }

    #[test]
    fn district_metrics_exist() {
        let w = World::initialize(42, 10, 3);
        for district in w.state.city.districts.values() {
            assert!(!district.parcel_ids.is_empty(), "District should have parcels");
            assert!(district.total_property_value >= 0.0);
        }
    }

    #[test]
    fn property_values_positive() {
        let w = World::initialize(42, 10, 3);
        for parcel in w.state.parcels.values() {
            assert!(
                parcel.current_value > 0.0,
                "Parcel {} should have positive value",
                parcel.id
            );
        }
    }

    #[test]
    fn deterministic_replay_phase6() {
        let mut w1 = World::initialize(42, 10, 3);
        let mut w2 = World::initialize(42, 10, 3);
        for _ in 0..500 {
            w1.tick();
            w2.tick();
        }
        assert_eq!(w1.state.parcels.len(), w2.state.parcels.len());
        assert_eq!(w1.state.buildings.len(), w2.state.buildings.len());
        assert_eq!(w1.state.properties.len(), w2.state.properties.len());
        for pid in w1.state.parcels.keys() {
            let p1 = w1.state.parcels.get(pid).unwrap();
            let p2 = w2.state.parcels.get(pid).unwrap();
            assert_eq!(p1.owner, p2.owner, "Parcel {} owner mismatch", pid);
            assert!(
                (p1.current_value - p2.current_value).abs() < 0.01,
                "Parcel {} value mismatch",
                pid
            );
        }
        for bid in w1.state.buildings.keys() {
            let b1 = w1.state.buildings.get(bid).unwrap();
            let b2 = w2.state.buildings.get(bid).unwrap();
            assert_eq!(b1.construction_state, b2.construction_state, "Building {} state mismatch", bid);
            assert_eq!(b1.capacity, b2.capacity, "Building {} capacity mismatch", bid);
        }
    }
}
