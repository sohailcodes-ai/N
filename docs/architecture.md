# N Genesis — Architecture Documentation (Updated)

## Build Requirements

**Required for compilation:**
- Rust 1.75+ (1.98.1 installed)
- Visual Studio 2019 or later with "Desktop development with C++" workload
- Windows SDK (provides link.exe)
- Or: Mingw-w64 toolchain with `pkg-config` and `make`

**Alternative (no MSVC linker needed):**
- `rustup target add x86_64-unknown-linux-gnu` — compiles for Linux
- Use `musl` target for static linking
- Cross-compile from a Linux host

**This project is configured for:**
- Windows x86_64 with MSVC toolchain
- Linker: `link.exe` (Visual Studio C++ build tools)
- Runtime: CRT (C Runtime)

**If link.exe is unavailable:**
1. Install "Desktop development with C++" workload in Visual Studio 2022
2. Or: `rustup target add x86_64-unknown-linux-gnu` and compile for Linux
3. Or: Use a Linux host with `sudo apt install build-essential`

## Project Structure

```
n/
├── Cargo.toml                  # Project root (package n-project v0.1.0)
├── Cargo.lock                  # Auto-generated lock file
├── .gitignore                  # See below
├── docs/
│   └── architecture.md         # Full architecture documentation
├── crates/
│   ├── simulation/             # Core simulation engine
│   │   ├── src/
│   │   │   ├── lib.rs          # SimulationCore, World, Clock, Agent, Company, Event
│   │   │   └── main.rs         # Minimal binary for linking
│   │   ├── Cargo.toml          # n-simulation v0.1.0
│   │   └── tests/
│   │       └── phase1_tests.rs # Phase 1 test suite
│   ├── economy/                # Economy and market system
│   │   ├── src/
│   │   │   └── lib.rs          # Module declarations
│   │   └── Cargo.toml          # n-economy v0.1.0
│   ├── agents/                 # Agent entity and behavior
│   │   ├── src/
│   │   │   └── lib.rs          # Module declarations
│   │   └── Cargo.toml          # n-agents v0.1.0
│   ├── organizations/          # Company and ownership
│   │   ├── src/
│   │   │   └── lib.rs          # Module declarations
│   │   └── Cargo.toml          # n-organizations v0.1.0
│   ├── world/                  # City and district layout
│   │   ├── src/
│   │   │   └── lib.rs          # Module declarations
│   │   └── Cargo.toml          # n-world v0.1.0
│   ├── events/                 # Event system and history
│   │   ├── src/
│   │   │   └── lib.rs          # Module declarations
│   │   └── Cargo.toml          # n-events v0.1.0
│   └── api/                    # Axum HTTP server
│       ├── src/
│       │   └── lib.rs          # Module declarations
│       └── Cargo.toml          # n-api v0.1.0
├── docs/
│   └── architecture.md         # Full architecture documentation
├── migrations/                 # Database/migrations (future)
├── dashboard/                  # React + TypeScript dashboard (future)
│   └── tests/
│       └── test_suites.md
└── tests/                      # Integration tests (future)
```

## Core Crates

### n-simulation (PRIMARY)
- `SimulationClock`: Deterministic clock with ChaCha8Rng
- `SimulationState`: Authoritative world state
- `World`: Coordinates tick advances and state transitions
- `AgentState`: Agent properties, skills, money, inventory
- `CompanyState`: Company properties, finances, products
- `BuildingState`: Building properties, construction state
- `CityLayout`: Districts and road network
- `MarketState`: Resource prices and orders
- `Event`: Append-only event history
- `EventType`: Enum of all event types
- `run_tick()`: Advance simulation by one tick
- `default_world()`: Initialize world with 50 agents, 10 companies

### n-economy
- Resource and product definitions
- Market structures and price discovery
- Market orders (buy/sell)
- Pricing mechanisms

### n-agents
- Agent cognition interface
- Needs and skills
- Behavior models

### n-organizations
- Company founding and governance
- Employment and salaries
- Ownership structure

### n-world
- City layout and district management
- World state coordination

### n-events
- Event types and enumeration
- History tracking
- Persistence concepts

### n-api
- Axum HTTP server routes
- API endpoints for world state inspection
- Dashboard integration

## Key Domain Invariants (Tested)

1. An agent cannot spend money it does not possess.
2. Inventory cannot become negative.
3. A transaction requires valid counterparties.
4. A building cannot have two owners simultaneously.
5. A company cannot hire a nonexistent agent.
6. A company cannot magically receive revenue.
7. Production requires valid inputs.
8. Consumption reduces inventory.
9. Salary payments reduce company funds and increase employee funds.
10. Invalid intents cannot mutate state.

## Economic Causality Flow

```
Agent creates product → Company sells product → Customer pays → Company receives revenue
→ Company pays salaries → Company retains profit → Company expands → Hires more workers
→ Demands larger office → Property demand increases → Construction occurs → New building
→ More workers → Increased food demand → Shops appear → City evolves
```

## AI Cognition Workflow

```
LLM Observation → Cognition → Intent → Validation → Simulation Rules → State Transition → Event → Persistent World State
```

LLM never directly mutates simulation state. The simulation validates and executes.

## Economic Cycle

1. Agent has skill + money
2. Agent proposes intent (e.g., "buy food")
3. Simulation validates: agent has sufficient money + product available
4. State transition: money decreases, inventory increases
5. Event: ResourceConsumed recorded
6. Price updates via supply/demand
7. Next agent can decide based on updated market

## Event System

Every important state transition generates an event with:
- `id`: Unique event identifier
- `tick`: Simulation tick number
- `event_type`: Enum (Tick, AgentCreated, CompanyFounded, etc.)
- `actor`: Optional entity causing the event
- `cause`: Optional reason/cause
- `entities`: Affected entity IDs
- `state_snapshot`: JSON of relevant state changes
- `timestamp`: ISO 8601 timestamp

Events enable: debugging emergent behavior, replay, persistence, observability.

## Implementation Phases (10-Phase Order)

### Phase 1 — Engine (DONE)
- Repository architecture
- Simulation state and clock
- Seeded randomness (ChaCha8Rng)
- Tick loop
- World initialization (50 agents, 10 companies, seed 1234)
- Basic Agent and Company entities
- Deterministic simulation tick
- Tests and documentation

### Phase 2 — Agents
- Needs (hunger, thirst)
- Advanced skills
- Inventory management
- Basic behavior

### Phase 3 — Resources
- Food, water, energy definitions
- Production processes
- Consumption

### Phase 4 — Economy
- Markets and orders
- Supply/demand price discovery
- Transactions and settlement

### Phase 5 — Organizations
- Company formation
- Employment system
- Salary payment
- Ownership and shares

### Phase 6 — City
- Land and property
- Buildings and construction
- Maintenance

### Phase 7 — Events
- Full event system
- Persistence and replay
- Observability

### Phase 8 — AI
- AgentBrain model-agnostic interface
- Structured intents
- Validation
- Decision queue
- Fallback behavior (deterministic when LLM unavailable)

### Phase 9 — Emergence
- Autonomous companies
- Autonomous agents
- Investment and expansion
- Competition and specialization
- Economic cycles

### Phase 10 — Dashboard
- API development
- Developer dashboard
- World inspector
- 3D visualization (Three.js + React Three Fiber, post-core)

## Dependencies (per crate)

### n-simulation
- `uuid` v1 — entity IDs
- `serde` v1 with derive — serialization
- `serde_json` v1 — JSON handling
- `chrono` v0.4 — time stamps
- `rand` v0.8 — RNG
- `rand_chacha` v0.2 — ChaCha8Rng

### n-economy
- `serde` v1 with derive
- `serde_json` v1

### n-agents
- `serde` v1 with derive
- `serde_json` v1
- `uuid` v1

### n-organizations
- `serde` v1 with derive
- `serde_json` v1
- `uuid` v1

### n-world
- `serde` v1 with derive
- `serde_json` v1
- `uuid` v1

### n-events
- `serde` v1 with derive
- `serde_json` v1

### n-api
- `axum` v0.7 — HTTP server
- `tower-http` v0.5 — middleware
- `serde` v1 with derive
- `serde_json` v1
- `tokio` v1 with ["full"] — async runtime

## Phase 2 — Agent Needs, Routines, Inventory & Skill Progression

### Why LLMs Are NOT Used in Phase 2

Phase 2 implements the deterministic survival layer that sits underneath future AgentBrain/LLM cognition. The simulation must continue running correctly even when all LLM providers are unavailable. LLMs will later propose high-level intents; Phase 2 provides the deterministic routine that runs every tick without external dependencies.

### Agent Needs Model

All needs are bounded `f64` in range `[0.0, 1.0]`.
- `0.0` = completely satisfied
- `1.0` = maximum need (starving/dehydrated/exhausted)

Needs increase over time at documented rates:
- Hunger: +0.01/hour (~4 days to starvation from 0)
- Thirst: +0.02/hour (~2 days to dehydration from 0, ~2x hunger rate)
- Fatigue: +0.008/hour while awake (~5 days to exhaustion from 0)

Needs decrease through consumption/rest:
- Eating 1 food unit: -0.3 hunger
- Drinking 1 water unit: -0.35 thirst
- Sleeping: -0.08 fatigue/hour (full recovery in ~12.5 hours)
- Resting: -0.04 fatigue/hour (partial recovery in ~25 hours)

All needs are clamped to `[0.0, 1.0]` after every modification.

### Agent Status

`AgentStatus` enum:
- `Active` — awake, not working
- `Working` — employed and currently working
- `Resting` — partially recovering fatigue
- `Sleeping` — fully recovering fatigue
- `Eating` — consuming food
- `Drinking` — consuming water
- `Idle` — no employment, satiated
- `Unemployed` — no employer (legacy, use Idle + no employer)

Status transitions are deterministic and driven by the routine priority system.

### Inventory System

`AgentInventory` stores `HashMap<String, u32>` with per-resource capacity limits.

Operations:
- `add_resource(resource, amount)` → `(success, actual_added)`
- `remove_resource(resource, amount)` → `(success, actual_removed)`
- `has_resource(resource)` → `bool`
- `resource_quantity(resource)` → `u32`

Invariants:
- No negative inventory (quantities are `u32`)
- Cannot consume resources that don't exist
- Cannot exceed capacity
- Failed operations do not partially mutate state beyond what they removed

Default capacities: food=10, water=5.

### Deterministic Daily Routines

Priority system (checked every tick):
1. Critical thirst (thirst > 0.6) + has water → Drink
2. Critical hunger (hunger > 0.6) + has food → Eat
3. Critical fatigue (fatigue > 0.7) + is night → Sleep
4. Fatigue (fatigue > 0.5) → Rest
5. Employed + work hours → Work
6. Critical fatigue (any time) → Sleep
7. Otherwise → Idle

Time awareness:
- Night: hours 22-5 → prefer sleep
- Work hours: hours 9-17 → prefer work if employed
- Each tick = 1 simulated hour

### Skill Progression

Work generates experience with diminishing returns:
```
xp_gain = SKILL_XP_BASE_RATE * (1.0 - current_skill)
```
Base rate: 0.005 per work tick.

At skill 0.0 → 0.005 XP/tick
At skill 0.5 → 0.0025 XP/tick
At skill 0.9 → 0.0005 XP/tick
At skill 1.0 → 0 XP/tick (maxed)

Skills never exceed 1.0 and never go negative.
Skills accumulate as `experience` (f64) separate from the skill value.

### Employment Hook

Phase 2 provides the foundation for Phase 5 labor market:
- `AgentState.employer: Option<String>` — company ID if employed
- `AgentState.status: AgentStatus::Working` — currently working
- `AgentState.skills: HashMap<String, f64>` — relevant skills
- `AgentState.experience: HashMap<String, f64>` — work XP

Working is deterministic: if employed + work hours + needs satisfied → work.

### Events

New event types in Phase 2:
- `AgentAte` — agent consumed food
- `AgentDrank` — agent consumed water
- `AgentStartedWork` — began working
- `AgentStoppedWork` — stopped working
- `AgentStartedRest` — began resting
- `AgentStartedSleep` — began sleeping
- `SkillImproved` — skill increased from work
- `AgentNeedChanged` — need changed significantly (threshold-gated to avoid spam)

Events preserve causal information: who, what, when, why, what changed.

### Constants and Thresholds

All thresholds are documented in `lib.rs` as named constants:
- `HUNGER_RATE_PER_HOUR`: 0.01
- `THIRST_RATE_PER_HOUR`: 0.02
- `FATIGUE_RATE_PER_HOUR_AWAKE`: 0.008
- `FATIGUE_RECOVERY_RATE_PER_HOUR_SLEEPING`: 0.08
- `FATIGUE_RECOVERY_RATE_PER_HOUR_RESTING`: 0.04
- `HUNGER_REDUCTION_PER_FOOD`: 0.3
- `THIRST_REDUCTION_PER_WATER`: 0.35
- `CRITICAL_THIRST_THRESHOLD`: 0.6
- `CRITICAL_HUNGER_THRESHOLD`: 0.6
- `SLEEP_THRESHOLD`: 0.7
- `REST_THRESHOLD`: 0.5
- `SKILL_XP_BASE_RATE`: 0.005
- `HOURS_PER_DAY`: 24
- `DEFAULT_FOOD_CAPACITY`: 10
- `DEFAULT_WATER_CAPACITY`: 5
- `NEEDS_CHANGE_EVENT_THRESHOLD`: 0.05

### Architecture Constraint

Phase 2 implements ONLY the deterministic layer:

```
Observation → Deterministic constraints / needs → [Agent cognition — FUTURE] → Intent → Validation → Simulation → State transition → Event → Persistence
```

No LLM logic leaks into the simulation core. The routine system is the foundation that LLM cognition will layer on top of in Phase 8.

## Gitignore

```
target/
Cargo.lock
*.md.backup
*.rs.bak
.env
*.bak
```

## Licensing

TBD — open source suitable for commercial and research use.

## Contact

Lead Systems Architect — N Genesis Simulation