# N Genesis — Agent Architecture

## Overview

Agents are the fundamental population of the N civilization simulation. Each agent is an autonomous entity with physical needs, skills, inventory, employment, and behavior.

## Agent State

```rust
pub struct AgentState {
    pub id: String,              // unique identifier
    pub name: String,            // display name
    pub age: u8,                 // age in years
    pub skills: HashMap<String, f64>,  // skill_name → 0.0-1.0
    pub money: f64,              // currency in N
    pub inventory: AgentInventory,     // food, water, etc.
    pub employer: Option<String>,      // company_id if employed
    pub location: String,        // current district
    pub status: AgentStatus,     // current activity
    pub needs: AgentNeeds,       // hunger, thirst, fatigue
    pub experience: HashMap<String, f64>,  // work XP per skill
    pub memories: Vec<String>,   // recent decisions/events
    pub goals: Vec<String>,      // long-term goals
}
```

## Needs Model

Bounded `[0.0, 1.0]`:
- `0.0` = fully satisfied
- `1.0` = maximum need (danger)

Needs increase over time, decrease through consumption/rest.

See `docs/architecture.md` Phase 2 section for exact rates and thresholds.

## Skills

Skills represent agent capabilities in the range `[0.0, 1.0]`:
- `productivity` — work output quality/speed
- `social` — interaction effectiveness

Skills improve through work experience with diminishing returns.

## Inventory

Resources stored as `HashMap<String, u32>`:
- `food` — consumed to reduce hunger
- `water` — consumed to reduce thirst
- Future: energy, materials, etc.

Capacity-limited per resource type.

## Status

Deterministic status driven by needs and employment:
- `Working` → skill progression, fatigue increase
- `Sleeping` → fast fatigue recovery
- `Resting` → slow fatigue recovery
- `Eating` → hunger reduction, food consumption
- `Drinking` → thirst reduction, water consumption
- `Idle` → no current activity
- `Active` → awake, not in any specific activity

## Future: Agent Cognition (Phase 8)

Phase 2 implements deterministic survival routines. Phase 8 adds LLM-driven cognition on top:

```
Phase 2: Needs + Routines (deterministic)
    ↓
Phase 8: AgentBrain (LLM proposes intent)
    ↓
Phase 8: Validation (simulation rules accept/reject)
    ↓
State transition
```

The deterministic routine always runs. LLM cognition adds high-level decision-making on top.

## Economic Role

Agents participate in the economy by:
- Working for companies (earning salary)
- Consuming food/water (demand)
- Owning property (future)
- Trading (future)
- Starting companies (future)
