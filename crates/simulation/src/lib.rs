pub mod agent;
pub mod city;
pub mod clock;
pub mod company;
pub mod economy;
pub mod events;
pub mod market;
pub mod production;
pub mod resources;

pub use agent::*;
pub use city::*;
pub use clock::*;
pub use company::*;
pub use economy::*;
pub use events::*;
pub use market::*;
pub use production::*;
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

            let mut employees = HashMap::new();
            let count = (i % 5) as u32 + 1;
            employees.insert(format!("agent-{:04}", i), count);

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

            let company = CompanyState {
                id: company_id.clone(),
                name: format!("Company-{}", i),
                founder: founder_id,
                owners,
                employees,
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
                },
            );
        }

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
        self.process_purchasing(tick);
        self.process_water_purchasing(tick);
        self.process_routines(tick, is_night, is_work_hours);
        self.process_labor(tick);
        self.process_production(tick);
        self.process_market(tick);
        self.process_wages(tick);
        self.process_resource_regeneration(tick);
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
        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
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

        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
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
        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
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
        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        let company_ids: Vec<String> = self.state.companies.keys().cloned().collect();

        for company_id in &company_ids {
            let working_agents: Vec<String> = agent_ids
                .iter()
                .filter(|aid| {
                    self.state
                        .agents
                        .get(*aid)
                        .map(|a| {
                            a.employer.as_deref() == Some(company_id.as_str())
                                && a.status == AgentStatus::Working
                        })
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            if working_agents.is_empty() {
                continue;
            }

            let working_count = working_agents.len();
            let company = self.state.companies.get(company_id).unwrap();
            let recipe_name = company.recipe_name.clone();

            if recipe_name.is_none() {
                continue;
            }

            let _recipe_name = recipe_name.unwrap();

            let (has_enough_workers, average_skill) = {
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
                let avg = if skill_count > 0 {
                    total_skill / skill_count as f64
                } else {
                    0.5
                };
                (working_count, avg)
            };

            self.state.events.push(Event {
                id: format!("evt-labor-{}-{}", company_id, tick),
                tick,
                event_type: EventType::AgentStartedWork,
                actor: Some(company_id.clone()),
                cause: Some(format!("{} workers contributing labor", has_enough_workers)),
                entities: working_agents,
                state_snapshot: format!(
                    "Company {} received {} units of labor, avg skill {:.2}",
                    company_id, has_enough_workers, average_skill
                ),
            });
        }
    }

    fn process_production(&mut self, tick: u64) {
        let company_ids: Vec<String> = self.state.companies.keys().cloned().collect();
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
            let cooldown = company.production_cooldown;
            let inventory = company.inventory.clone();

            let working_count = self
                .state
                .agents
                .values()
                .filter(|a| {
                    a.employer.as_deref() == Some(company_id.as_str())
                        && a.status == AgentStatus::Working
                })
                .count();

            let average_skill = {
                let working_agents: Vec<_> = self
                    .state
                    .agents
                    .values()
                    .filter(|a| {
                        a.employer.as_deref() == Some(company_id.as_str())
                            && a.status == AgentStatus::Working
                    })
                    .collect();
                if working_agents.is_empty() {
                    0.5
                } else {
                    let total: f64 = working_agents
                        .iter()
                        .map(|a| a.skills.get("productivity").copied().unwrap_or(0.5))
                        .sum();
                    total / working_agents.len() as f64
                }
            };

            if can_produce(&recipe, &inventory, working_count, cooldown) {
                self.state.events.push(Event {
                    id: format!("evt-prodstart-{}-{}", company_id, tick),
                    tick,
                    event_type: EventType::ProductionStarted,
                    actor: Some(company_id.clone()),
                    cause: Some(format!("recipe: {}", recipe_name)),
                    entities: vec![company_id.clone()],
                    state_snapshot: format!(
                        "Starting production of {} with {} workers, avg skill {:.2}",
                        recipe_name, working_count, average_skill
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
                                    "Produced {} {} from {}",
                                    output.quantity, output.resource, recipe_name
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
        let food_listing_data: Vec<(String, u32)> = self
            .state
            .companies
            .values()
            .filter_map(|c| {
                c.inventory
                    .get("food")
                    .copied()
                    .filter(|q| *q > 0)
                    .map(|q| (c.id.clone(), q))
            })
            .collect();

        let water_listing_data: Vec<(String, u32)> = self
            .state
            .companies
            .values()
            .filter_map(|c| {
                c.inventory
                    .get("water")
                    .copied()
                    .filter(|q| *q > 0)
                    .map(|q| (c.id.clone(), q))
            })
            .collect();

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

        let recent_transactions: Vec<_> = self
            .state
            .markets
            .transactions
            .iter()
            .filter(|t| t.tick + 24 >= tick)
            .cloned()
            .collect();

        for tx in &recent_transactions {
            *demand.entry(tx.resource.clone()).or_insert(0.0) += tx.quantity as f64;
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

        let _food_sold: u32 = recent_transactions
            .iter()
            .filter(|t| t.resource == "food")
            .map(|t| t.quantity)
            .sum();

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

        let _water_sold: u32 = recent_transactions
            .iter()
            .filter(|t| t.resource == "water")
            .map(|t| t.quantity)
            .sum();

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
            food_ask_depth: recent_transactions
                .iter()
                .filter(|t| t.resource == "food")
                .map(|t| t.quantity)
                .sum(),
            water_bid_depth: water_listing_data.iter().map(|(_, q)| *q).sum(),
            water_ask_depth: recent_transactions
                .iter()
                .filter(|t| t.resource == "water")
                .map(|t| t.quantity)
                .sum(),
        };

        self.state
            .markets
            .transactions
            .retain(|t| t.tick + 24 >= tick);
    }

    fn process_wages(&mut self, tick: u64) {
        let company_ids: Vec<String> = self.state.companies.keys().cloned().collect();

        for company_id in &company_ids {
            let working_employees: Vec<String> = self
                .state
                .agents
                .values()
                .filter(|a| {
                    a.employer.as_deref() == Some(company_id.as_str())
                        && a.status == AgentStatus::Working
                })
                .map(|a| a.id.clone())
                .collect();

            if working_employees.is_empty() {
                continue;
            }

            let company = self.state.companies.get(company_id).unwrap();
            let company_cash = company.cash;

            let mut total_wages = 0.0;
            let mut paid_employees = Vec::new();

            for emp_id in &working_employees {
                if company_cash - total_wages >= WAGE_PER_TICK {
                    total_wages += WAGE_PER_TICK;
                    paid_employees.push(emp_id.clone());
                }
            }

            if total_wages > 0.0 {
                let company = self.state.companies.get_mut(company_id).unwrap();
                company.cash -= total_wages;
                company.expenses += total_wages;

                for emp_id in &paid_employees {
                    let agent = self.state.agents.get_mut(emp_id).unwrap();
                    agent.money += WAGE_PER_TICK;
                }

                self.state.events.push(Event {
                    id: format!("evt-wage-{}-{}", company_id, tick),
                    tick,
                    event_type: EventType::WagePaid,
                    actor: Some(company_id.clone()),
                    cause: Some("regular_wage".to_string()),
                    entities: paid_employees.clone(),
                    state_snapshot: format!(
                        "Paid {} N to {} employees",
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

        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
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

    fn process_resource_regeneration(&mut self, tick: u64) {
        if tick % TICKS_PER_DAY != 0 {
            return;
        }

        for company in self.state.companies.values_mut() {
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
            assert!(
                listing.price >= FOOD_PRICE_MIN,
                "Listing price should be >= min: {}",
                listing.price
            );
            assert!(
                listing.price <= FOOD_PRICE_MAX,
                "Listing price should be <= max: {}",
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
            agent.employer = Some(company_id.clone());
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.needs.fatigue = 0.1;
            agent.inventory.add_resource("food", 5);
            agent.inventory.add_resource("water", 5);
        }
        let initial_cash = w.state.companies.get(&company_id).unwrap().cash;
        w.clock.tick = 10 * TICKS_PER_HOUR;
        w.tick();
        let final_cash = w.state.companies.get(&company_id).unwrap().cash;
        assert!(
            final_cash < initial_cash,
            "Company cash should decrease from wages"
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
        w.clock.tick = 10 * TICKS_PER_HOUR;
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
        let company_ids: Vec<String> = w.state.companies.keys().cloned().collect();
        for cid in &company_ids {
            let employees: Vec<String> = w
                .state
                .companies
                .get(cid)
                .unwrap()
                .employees
                .keys()
                .cloned()
                .collect();
            for emp_id in &employees {
                if let Some(agent) = w.state.agents.get_mut(emp_id) {
                    agent.employer = Some(cid.clone());
                }
            }
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
        assert!(total_food < 50 * 30, "Some food should have been consumed");
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
        let has_water_purchased = w
            .state
            .events
            .iter()
            .any(|e| e.event_type == EventType::WaterPurchased);
        assert!(
            has_water_purchased,
            "Water should be purchased in 30-day soak"
        );
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
}
