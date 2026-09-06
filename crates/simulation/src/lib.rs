use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── CONSTANTS ────────────────────────────────────────────────────────────────
// Documented thresholds. All phases of the simulation reference these.

/// How much hunger increases per simulation hour.
/// Reasoning: at 0.01/hour, an agent goes from 0→1 (starving) in 100 hours (~4 days).
pub const HUNGER_RATE_PER_HOUR: f64 = 0.01;

/// How much thirst increases per simulation hour.
/// Thirst increases ~2x faster than hunger. Dehydration is more immediate.
pub const THIRST_RATE_PER_HOUR: f64 = 0.02;

/// How much fatigue increases per simulation hour while awake/working.
/// Reasoning: ~0.008/hr means ~125 hours (5 days) to max fatigue if never resting.
pub const FATIGUE_RATE_PER_HOUR_AWAKE: f64 = 0.008;

/// How much fatigue decreases per simulation hour while sleeping.
/// Full recovery in ~12.5 hours (0.8 recovery/hr).
pub const FATIGUE_RECOVERY_RATE_PER_HOUR_SLEEPING: f64 = 0.08;

/// How much fatigue decreases per simulation hour while resting (not sleeping).
/// Partial recovery in ~25 hours (0.04 recovery/hr).
pub const FATIGUE_RECOVERY_RATE_PER_HOUR_RESTING: f64 = 0.04;

/// How much hunger decreases per food unit consumed.
/// 1 food unit drops hunger by 0.3 (3 units to go from 1.0→0.1).
pub const HUNGER_REDUCTION_PER_FOOD: f64 = 0.3;

/// How much thirst decreases per water unit consumed.
/// 1 water unit drops thirst by 0.35 (3 units to go from 1.0→0.05).
pub const THIRST_REDUCTION_PER_WATER: f64 = 0.35;

/// Thirst above this triggers drinking (highest priority).
pub const CRITICAL_THIRST_THRESHOLD: f64 = 0.6;

/// Hunger above this triggers eating (second priority).
pub const CRITICAL_HUNGER_THRESHOLD: f64 = 0.6;

/// Fatigue above this triggers sleeping (third priority).
pub const SLEEP_THRESHOLD: f64 = 0.7;

/// Fatigue above this triggers resting (lower priority than sleep).
pub const REST_THRESHOLD: f64 = 0.5;

/// Hours in a simulated day.
pub const HOURS_PER_DAY: u64 = 24;

/// Ticks per simulated hour. With 1 tick = 1 hour in our simple model.
pub const TICKS_PER_HOUR: u64 = 1;

/// Skill experience gain per work tick (before diminishing returns).
/// At skill 0.0, one work tick gives 0.005 XP.
/// Diminishing returns curve: xp_gain = base_rate * (1.0 - skill).
pub const SKILL_XP_BASE_RATE: f64 = 0.005;

/// Maximum inventory capacity defaults.
pub const DEFAULT_FOOD_CAPACITY: u32 = 10;
pub const DEFAULT_WATER_CAPACITY: u32 = 5;

/// Thresholds for triggering needs-driven events (avoids spam).
pub const NEEDS_CHANGE_EVENT_THRESHOLD: f64 = 0.05;

// ─── SIMULATION CLOCK ─────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SimulationClock {
    pub tick: u64,
    pub speed: f64,
    pub rng: StdRng,
}

impl SimulationClock {
    pub fn new(seed: u64, speed: f64) -> Self {
        Self {
            tick: 0,
            speed,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    pub fn advance(&mut self, ticks: u64) {
        self.tick += ticks;
    }

    pub fn random(&mut self) -> f64 {
        self.rng.gen()
    }

    /// Convert current tick to simulated hour of day (0-23).
    pub fn hour_of_day(&self) -> u64 {
        self.tick % HOURS_PER_DAY
    }

    /// Convert current tick to simulated day number (1-based).
    pub fn day(&self) -> u64 {
        (self.tick / HOURS_PER_DAY) + 1
    }

    /// Is it nighttime? Hours 22-5 inclusive.
    pub fn is_night(&self) -> bool {
        let h = self.hour_of_day();
        h >= 22 || h < 6
    }

    /// Is it work hours? Hours 9-17 inclusive.
    pub fn is_work_hours(&self) -> bool {
        let h = self.hour_of_day();
        h >= 9 && h < 18
    }
}

// ─── AGENT NEEDS ──────────────────────────────────────────────────────────────

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

// ─── AGENT STATUS ─────────────────────────────────────────────────────────────

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

// ─── AGENT INVENTORY ──────────────────────────────────────────────────────────

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

    /// Add resources. Returns (success, actual_amount_added).
    /// Partially fills if capacity exceeded.
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

    /// Remove resources. Returns (success, actual_removed).
    /// Fails (returns false) if not enough resources, but still removes what's available.
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

// ─── AGENT STATE ──────────────────────────────────────────────────────────────

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
}

// ─── COMPANY STATE ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompanyStrategy {
    Growth,
    Stability,
    ProfitMaximization,
    Innovation,
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
}

// ─── CITY / DISTRICT ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictLayout {
    pub name: String,
    pub area: f64,
    pub residents: u32,
    pub businesses: u32,
    pub average_income: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityLayout {
    pub districts: HashMap<String, DistrictLayout>,
    pub road_network: Vec<(String, String)>,
}

// ─── MARKET ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    pub prices: HashMap<String, f64>,
    pub buy_orders: HashMap<String, f64>,
    pub sell_orders: HashMap<String, f64>,
}

// ─── EVENT SYSTEM ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    Tick,
    AgentCreated,
    AgentStatusChanged,
    CompanyFounded,
    CompanyClosed,
    AgentHired,
    AgentFired,
    SalaryPaid,
    ProductProduced,
    ProductSold,
    PropertyPurchased,
    PropertySold,
    BuildingConstructed,
    BuildingCompleted,
    MarketOrderPlaced,
    MarketOrderFilled,
    ResourceConsumed,
    AgentAte,
    AgentDrank,
    AgentStartedWork,
    AgentStoppedWork,
    AgentStartedRest,
    AgentStartedSleep,
    SkillImproved,
    AgentNeedChanged,
    IntentProposed,
    IntentValidated,
    DecisionMade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub tick: u64,
    pub event_type: EventType,
    pub actor: Option<String>,
    pub cause: Option<String>,
    pub entities: Vec<String>,
    pub state_snapshot: String,
}

// ─── SIMULATION STATE ─────────────────────────────────────────────────────────

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
}

// ─── WORLD ────────────────────────────────────────────────────────────────────

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
        prices.insert("food".to_string(), 1.0);
        prices.insert("water".to_string(), 0.5);
        prices.insert("energy".to_string(), 2.0);

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
            },
            tick: 0,
        };

        World { state, clock }
    }

    /// Advance simulation by one tick.
    pub fn tick(&mut self) {
        self.clock.advance_tick();
        self.state.tick = self.clock.tick;
        self.process_needs();
        self.process_routines();
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

    // ─── NEEDS PROCESSING ─────────────────────────────────────────────────

    fn process_needs(&mut self) {
        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        for agent_id in &agent_ids {
            let agent = self.state.agents.get_mut(agent_id).unwrap();

            let prev_hunger = agent.needs.hunger;
            let prev_thirst = agent.needs.thirst;
            let prev_fatigue = agent.needs.fatigue;

            // Increase needs based on current status
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

            // Record need-change events only when change exceeds threshold
            let hunger_delta = (agent.needs.hunger - prev_hunger).abs();
            let thirst_delta = (agent.needs.thirst - prev_thirst).abs();
            let fatigue_delta = (agent.needs.fatigue - prev_fatigue).abs();

            if hunger_delta > NEEDS_CHANGE_EVENT_THRESHOLD
                || thirst_delta > NEEDS_CHANGE_EVENT_THRESHOLD
                || fatigue_delta > NEEDS_CHANGE_EVENT_THRESHOLD
            {
                self.state.events.push(Event {
                    id: format!("evt-need-{}-{}", agent_id, self.clock.tick),
                    tick: self.clock.tick,
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

    // ─── DETERMINISTIC ROUTINES ───────────────────────────────────────────

    fn process_routines(&mut self) {
        let agent_ids: Vec<String> = self.state.agents.keys().cloned().collect();
        for agent_id in &agent_ids {
            let agent = self.state.agents.get(agent_id).unwrap();
            let current_status = agent.status.clone();
            let has_food = agent.inventory.has_resource("food");
            let has_water = agent.inventory.has_resource("water");

            // Determine new status based on priority system
            let new_status = if agent.needs.needs_drinking() && has_water {
                AgentStatus::Drinking
            } else if agent.needs.needs_eating() && has_food {
                AgentStatus::Eating
            } else if agent.needs.needs_sleeping() && self.clock.is_night() {
                AgentStatus::Sleeping
            } else if agent.needs.needs_resting() {
                AgentStatus::Resting
            } else if agent.employer.is_some() && self.clock.is_work_hours() {
                AgentStatus::Working
            } else if agent.needs.needs_sleeping() {
                AgentStatus::Sleeping
            } else {
                AgentStatus::Idle
            };

            if new_status != current_status {
                // Record status change event
                let event_type = match &new_status {
                    AgentStatus::Working => Some(EventType::AgentStartedWork),
                    AgentStatus::Sleeping => Some(EventType::AgentStartedSleep),
                    AgentStatus::Resting => Some(EventType::AgentStartedRest),
                    _ => None,
                };

                if let Some(evt) = event_type {
                    self.state.events.push(Event {
                        id: format!("evt-status-{}-{}", agent_id, self.clock.tick),
                        tick: self.clock.tick,
                        event_type: evt,
                        actor: Some(agent_id.clone()),
                        cause: Some(format!("status_change_{:?}", current_status)),
                        entities: vec![agent_id.clone()],
                        state_snapshot: format!("{:?} → {:?}", current_status, new_status),
                    });
                }

                // Also record AgentStoppedWork if leaving work
                if current_status == AgentStatus::Working {
                    self.state.events.push(Event {
                        id: format!("evt-stopwork-{}-{}", agent_id, self.clock.tick),
                        tick: self.clock.tick,
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

            // Execute the action for the current status
            let agent = self.state.agents.get_mut(agent_id).unwrap();
            match agent.status {
                AgentStatus::Drinking => {
                    let removed = agent.inventory.remove_resource("water", 1);
                    if removed.0 {
                        agent.needs.thirst -= THIRST_REDUCTION_PER_WATER;
                        agent.needs.clamp();
                        self.state.events.push(Event {
                            id: format!("evt-drink-{}-{}", agent_id, self.clock.tick),
                            tick: self.clock.tick,
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
                            id: format!("evt-eat-{}-{}", agent_id, self.clock.tick),
                            tick: self.clock.tick,
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
                    // Skill advancement through work
                    let agent = self.state.agents.get_mut(agent_id).unwrap();
                    let skill_key = "productivity".to_string();
                    let current_skill = agent.skills.get("productivity").copied().unwrap_or(0.0);
                    let current_xp = agent.experience.get("productivity").copied().unwrap_or(0.0);

                    // Diminishing returns: XP gain decreases as skill increases
                    let xp_gain = SKILL_XP_BASE_RATE * (1.0 - current_skill);
                    let new_xp = current_xp + xp_gain;
                    let new_skill = (new_xp).min(1.0);

                    // Only emit event if skill actually increased meaningfully
                    if (new_skill - current_skill) > 0.0001 {
                        self.state.events.push(Event {
                            id: format!("evt-skill-{}-{}", agent_id, self.clock.tick),
                            tick: self.clock.tick,
                            event_type: EventType::SkillImproved,
                            actor: Some(agent_id.clone()),
                            cause: Some("work_experience".to_string()),
                            entities: vec![agent_id.clone()],
                            state_snapshot: format!(
                                "productivity: {:.4} → {:.4}",
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
}

// ─── PUBLIC HELPERS ───────────────────────────────────────────────────────────

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

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── NEEDS TESTS ──────────────────────────────────────────────────────

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
            "hunger should increase: {} → {}",
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
        // Start fully satisfied, run for a while
        w.advance(1000);
        for agent in w.state.agents.values() {
            assert!(agent.needs.hunger >= 0.0);
            assert!(agent.needs.thirst >= 0.0);
            assert!(agent.needs.fatigue >= 0.0);
        }
    }

    // ─── INVENTORY TESTS ──────────────────────────────────────────────────

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
        assert_eq!(amt, 5); // capacity is 5
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
        assert_eq!(amt, 2); // removed what was available
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

    // ─── CONSUMPTION TESTS ────────────────────────────────────────────────

    #[test]
    fn eating_reduces_hunger() {
        let mut w = World::initialize(42, 1, 0);
        let agent_id = w.state.agents.keys().next().unwrap().clone();

        // Give agent food and boost hunger
        {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.inventory.add_resource("food", 5);
            agent.needs.hunger = 0.8;
        }

        // Force eating status
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

        // Should not panic
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

    // ─── ROUTINE TESTS ────────────────────────────────────────────────────

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

        // Set tick to nighttime hour and make agent exhausted
        w.clock.tick = 23; // 23:00 = night

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
        // Set tick to work hours
        w.clock.tick = 10; // 10:00

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
        w.clock.tick = 10; // work hours

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

    // ─── SKILL TESTS ──────────────────────────────────────────────────────

    #[test]
    fn work_improves_skill() {
        let mut w = World::initialize(42, 2, 2);
        w.clock.tick = 10;

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
            "skill should increase after work: {} → {}",
            initial_skill,
            final_skill
        );
    }

    #[test]
    fn skill_never_exceeds_one() {
        let mut w = World::initialize(42, 2, 2);
        w.clock.tick = 10;

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

        // Work many ticks
        for _ in 0..1000 {
            let agent = w.state.agents.get_mut(&agent_id).unwrap();
            agent.needs.fatigue = 0.1; // prevent sleeping
            agent.needs.hunger = 0.1;
            agent.needs.thirst = 0.1;
            agent.status = AgentStatus::Working;
            w.clock.advance_tick();
            w.process_needs();
            w.process_routines();
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
        let mut w = World::initialize(42, 1, 0);
        for agent in w.state.agents.values() {
            for skill_val in agent.skills.values() {
                assert!(*skill_val >= 0.0);
            }
        }
    }

    // ─── DETERMINISM TESTS ────────────────────────────────────────────────

    #[test]
    fn deterministic_replay() {
        let mut w1 = World::initialize(1234, 5, 2);
        let mut w2 = World::initialize(1234, 5, 2);

        for _ in 0..100 {
            w1.tick();
            w2.tick();
        }

        // Same tick count
        assert_eq!(w1.tick_count(), w2.tick_count());

        // Same agent states
        for id in w1.state.agents.keys() {
            let a1 = w1.state.agents.get(id).unwrap();
            let a2 = w2.state.agents.get(id).unwrap();
            assert_eq!(a1.status, a2.status);
            assert!((a1.needs.hunger - a2.needs.hunger).abs() < 0.0001);
            assert!((a1.needs.thirst - a2.needs.thirst).abs() < 0.0001);
            assert!((a1.needs.fatigue - a2.needs.fatigue).abs() < 0.0001);
            assert_eq!(a1.inventory.resources, a2.inventory.resources);
        }

        // Same event count
        assert_eq!(w1.state.events.len(), w2.state.events.len());
    }

    // ─── TIME TESTS ───────────────────────────────────────────────────────

    #[test]
    fn hour_of_day_progresses() {
        let mut w = World::initialize(42, 1, 0);
        assert_eq!(w.clock.hour_of_day(), 0);
        w.advance(12);
        assert_eq!(w.clock.hour_of_day(), 12);
    }

    #[test]
    fn day_progresses() {
        let mut w = World::initialize(42, 1, 0);
        assert_eq!(w.clock.day(), 1);
        w.advance(24);
        assert_eq!(w.clock.day(), 2);
    }

    #[test]
    fn night_detected() {
        let mut w = World::initialize(42, 1, 0);
        w.clock.tick = 23;
        assert!(w.clock.is_night());
        w.clock.tick = 3;
        assert!(w.clock.is_night());
        w.clock.tick = 12;
        assert!(!w.clock.is_night());
    }

    #[test]
    fn work_hours_detected() {
        let mut w = World::initialize(42, 1, 0);
        w.clock.tick = 10;
        assert!(w.clock.is_work_hours());
        w.clock.tick = 20;
        assert!(!w.clock.is_work_hours());
    }

    // ─── EVENT TESTS ──────────────────────────────────────────────────────

    #[test]
    fn events_generated_during_tick() {
        let mut w = World::initialize(42, 1, 0);
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

    // ─── ECONOMIC INVARIANT TESTS ─────────────────────────────────────────

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

    // ─── SOAK TEST ────────────────────────────────────────────────────────

    #[test]
    fn soak_test_7_days() {
        let mut w = World::initialize(1234, 50, 10);
        let ticks_7_days = HOURS_PER_DAY * 7;

        // Give every agent starting resources
        let agent_ids: Vec<String> = w.state.agents.keys().cloned().collect();
        for id in &agent_ids {
            let agent = w.state.agents.get_mut(id).unwrap();
            agent.inventory.add_resource("food", 30);
            agent.inventory.add_resource("water", 15);
        }

        for tick in 0..ticks_7_days {
            w.tick();
        }

        // Verify simulation survived
        assert_eq!(w.tick_count(), ticks_7_days);
        assert_eq!(w.state.agents.len(), 50);
        assert_eq!(w.state.companies.len(), 10);

        // Agents should have consumed some resources
        let total_food: u32 = w
            .state
            .agents
            .values()
            .map(|a| a.inventory.resource_quantity("food"))
            .sum();
        assert!(total_food < 50 * 30, "Some food should have been consumed");

        // Events should have been generated
        assert!(!w.state.events.is_empty());

        // Economic invariants
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
}
