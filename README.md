# N Genesis

**Persistent autonomous civilization simulation.**

N Genesis is a deterministic, persistent civilization simulator where agents live, work, consume resources, participate in an economy, form companies, interact with property and markets, and make decisions through a bounded cognition system.

The simulation is designed around one rule:

> **Agents propose. The simulation decides.**

There is no hidden AI authority, no external API dependency, and no scripted macroeconomic outcome. The world is governed by explicit simulation rules, resource constraints, economic causality, and deterministic state transitions.

---

## Current State

**Phase 9 — Emergent Behavior**

N Genesis currently includes:

### Simulation

- Deterministic simulation clock
- Seeded `ChaCha8Rng`
- Persistent world state
- 50+ agents
- 10+ companies
- 5 districts
- Road network
- Deterministic daily routines
- Physical needs: hunger, thirst, fatigue
- Agent inventory with capacity limits
- Skill progression with diminishing returns
- Employment and labor systems
- Property and building systems
- Construction and housing

### Resources & Production

- Typed resource system
- Food
- Water
- Raw food
- Typed inventories
- Production recipes
- Input/output requirements
- Labor requirements
- Production cooldowns
- Physical resource consumption
- Production output tracking

### Economy

- Market listings
- Supply/demand pricing
- Agent purchasing
- Transactions
- Wage payments
- Company revenue
- Company profitability
- Loss streaks
- Insolvency
- Hiring and firing
- Competitive labor market
- Skill-based hiring
- Wage-based job selection
- Money conservation

### Property & Construction

- Parcels
- Zoning
- Ownership
- Property values
- Typed buildings
- Building capacity
- Operational status
- Construction recipes
- Labor investment
- Construction progress
- Housing assignment
- District capacity metrics

### Events & Persistence

- Append-only event history
- Economic events
- Versioned world snapshots
- Serializable simulation state
- Exact RNG state persistence
- Deterministic snapshot restoration
- Deterministic continuation
- Replay verification
- Economic invariants
- Long-running soak tests

### Agent Cognition

Phase 8 introduced an explicit cognition pipeline:

```text
Observation
    ↓
Decision
    ↓
Typed Intent
    ↓
Validation
    ↓
Execution
    ↓
Event
```

The cognition layer includes:

- `CognitionProvider` abstraction
- `DeterministicRuleProvider`
- `ManualCognitionProvider`
- Bounded agent observations
- Typed `AgentIntent`
- Intent validation
- First-class rejection reasons
- Decision scheduling
- Cognition state
- Agent goals
- Bounded agent memory
- Cognition statistics
- Deterministic decision ordering

Cognition is **off by default**.

There is no required LLM, API key, internet connection, or paid service.

The simulation remains authoritative at every stage.

---

## Emergent Behavior

Phase 9 added the infrastructure required to study what happens when the existing systems interact over time.

The simulation can now measure and experiment with:

- Population statistics
- Wealth distributions
- Median/mean/min/max wealth
- Gini coefficient
- Wealth histograms
- Needs
- Employment
- Production
- Consumption
- Transactions
- Market prices
- Company activity
- Cognition decisions
- Intent distributions
- Rejection reasons
- District activity
- Agent trajectories
- Time-series metrics
- Multi-seed experiments
- Variance and standard deviation
- Deterministic experiment reports

### Experimental inputs

Supported controlled perturbations include:

- Food supply shocks
- Wage shocks
- Production shocks
- Agent money shocks

These are experimental inputs applied by the experiment runner. They are not hidden simulation rules.

The goal is to observe whether macro-level behavior emerges from local decisions and existing simulation mechanics.

---

## Architecture

```text
                    ┌──────────────────────┐
                    │      WORLD STATE     │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │     OBSERVATION      │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │      COGNITION       │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │    TYPED INTENT      │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │     VALIDATION       │
                    └──────────┬───────────┘
                               │
                         ┌─────┴─────┐
                         ▼           ▼
                    REJECTED       VALID
                         │           │
                         ▼           ▼
                      EVENT      EXECUTION
                                     │
                                     ▼
                              WORLD STATE
                                     │
                                     ▼
                                   EVENTS
                                     │
                                     ▼
                                  METRICS
                                     │
                                     ▼
                                EXPERIMENTS
```

The simulation core remains the source of truth.

Cognition can propose an action.

Validation determines whether that action is legal.

The simulation executes valid actions.

Metrics observe the result.

Experiments analyze the result.

Nothing in the cognition or metrics layer is allowed to bypass simulation authority.

---

## Economic Causality

N Genesis is built around explicit causal chains rather than isolated stat changes.

For example:

```text
Agent becomes hungry
        ↓
Checks inventory
        ↓
Food is unavailable
        ↓
Purchases food
        ↓
Money moves to seller
        ↓
Food moves to agent
        ↓
Agent consumes food
        ↓
Hunger decreases
```

Production creates another chain:

```text
Raw resources
      ↓
Labor
      ↓
Production recipe
      ↓
Inputs consumed
      ↓
Output produced
      ↓
Company inventory
      ↓
Market listing
      ↓
Agent purchase
      ↓
Company revenue
      ↓
Wages
      ↓
Agent income
      ↓
Future consumption
```

Companies can then react to their economic position:

```text
Revenue
  ↓
Profit / Loss
  ↓
Hiring pressure
  ↓
Production capacity
  ↓
Inventory
  ↓
Market supply
  ↓
Prices
```

Expansion extends the same chain:

```text
Profit exceeds expansion threshold
        ↓
Land acquisition
        ↓
Construction project
        ↓
Money + labor invested
        ↓
Construction progresses
        ↓
Building completed
        ↓
Capacity increases
        ↓
Production potential increases
        ↓
Property value changes
        ↓
District metrics change
```

The important part is that these outcomes are consequences of interacting systems rather than a single hardcoded "economy" rule.

---

## Emergence Experiments

Phase 9 provides the infrastructure to ask questions such as:

```text
What happens when food production drops 30%?

Does the market recover?

Does unemployment increase?

Do prices rise?

Do agents change their purchasing behavior?

Does wealth inequality change?

Do districts begin to differentiate economically?

Do agents with different goals and experiences diverge?

Does cognition materially change macro-level outcomes?
```

The simulator can run the same experiment across multiple deterministic seeds and compare the resulting distributions.

That makes it possible to distinguish reproducible behavior from a single interesting trajectory.

---

## Determinism

Determinism is a core architectural constraint.

Given:

```text
same seed
+ same initial state
+ same configuration
+ same inputs
```

the simulation should produce:

```text
same state
+ same events
+ same metrics
+ same future
```

This applies to:

- Simulation execution
- Agent cognition
- Market behavior
- Event generation
- Snapshot restoration
- Replay
- Experiments
- Multi-seed aggregation

Snapshot persistence includes the simulation clock and exact RNG state, allowing a restored simulation to continue deterministically.

---

## Persistence

World snapshots are versioned and contain the state required to resume the simulation.

Conceptually:

```text
World State
    +
Simulation Clock
    +
RNG State
    +
Version
```

This allows:

```text
Run
  ↓
Snapshot
  ↓
Shutdown
  ↓
Restore
  ↓
Continue
```

while preserving deterministic behavior.

---

## Testing

The simulation is heavily test-driven.

Current baseline:

**211 tests passing, 0 failing.**

Coverage includes:

- Simulation mechanics
- Agent needs
- Inventory
- Resources
- Production
- Markets
- Purchasing
- Employment
- Labor markets
- Wages
- Company profitability
- Insolvency
- Property
- Buildings
- Construction
- Housing
- Events
- Persistence
- Replay
- Cognition
- Intent validation
- Decision scheduling
- Cognition determinism
- Cognition persistence
- Metrics
- Distribution analysis
- Gini coefficient
- Time-series recording
- Experiments
- Multi-seed reproducibility
- Perturbations
- Agent trajectories
- Emergence analysis
- Money conservation
- Long-running simulation invariants
- Performance smoke tests

A 10,000-tick soak test also verifies long-run invariants including:

- money conservation
- bounded needs
- employment consistency
- bounded trajectory history

---

## Performance

Current measured baselines:

| Workload | Result |
|---|---:|
| `cargo test -p n-simulation` | ~102s debug |
| `cargo test -p n-simulation --release` | ~38s |
| 30-day soak | ~25s release |
| 30-day soak | ~102s debug |
| 500 agents × 100 ticks | ~852ms release |
| 500 agents × 100 ticks + cognition | ~871ms release |
| 500 agents × 100 ticks + Phase 9 metrics | comparable to Phase 8 baseline |

Phase 8 cognition adds approximately **2% overhead** to the 500-agent benchmark.

Phase 9 metrics are designed as an observational layer and do not mutate authoritative simulation state.

---

## Project Structure

The core simulation is organized around explicit subsystems rather than one monolithic engine.

```text
crates/
├── simulation/
│   └── src/
│       ├── agents/
│       ├── cognition/
│       ├── company/
│       ├── construction/
│       ├── economy/
│       ├── events/
│       ├── metrics/
│       ├── property/
│       ├── resources/
│       ├── ...
│       └── lib.rs
│
└── api/
    └── ...
```

The exact module structure evolves with the simulation.

For architectural details, see:

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/agents.md`](docs/agents.md)
- [`docs/simulation.md`](docs/simulation.md)
- [`docs/economy.md`](docs/economy.md)
- [`docs/employment.md`](docs/employment.md)
- [`docs/property.md`](docs/property.md)
- [`docs/cognition.md`](docs/cognition.md)
- [`docs/emergence.md`](docs/emergence.md)

---

## Building

Requires Rust and the Visual Studio 2022 Build Tools with the C++ workload on Windows.

```bash
cargo check --workspace
cargo test -p n-simulation
cargo run -p n-simulation --release
```

For release verification:

```bash
cargo test -p n-simulation --release
cargo clippy --workspace --tests
cargo fmt --all --check
```

---

## Roadmap

```text
Phase 1  — Engine foundation                         ✅
Phase 2  — Agent needs, routines, inventory, skills  ✅
Phase 3  — Resources, production, market, economy   ✅
Phase 4  — Advanced market dynamics                  ✅
Phase 5  — Employment, labor market, ownership      ✅
Phase 6  — Property, buildings, construction        ✅
Phase 7  — Events, persistence, replay              ✅
Phase 8  — Agent cognition                           ✅
Phase 9  — Emergent behavior & experimentation      ✅
Phase 10 — Dashboard, API, visualization             🔄
```

### Phase 10

The next stage is exposing the simulation as an actual simulation laboratory:

- Simulation session management
- Read-only API
- Simulation commands
- Event streaming
- Agent inspection
- Company inspection
- District inspection
- Market inspection
- Metrics API
- Cognition monitoring
- Experiment API
- Snapshot/replay controls
- Live simulation controls
- Historical metric visualization
- Agent trajectory visualization
- Economic dashboards
- Experiment comparison

The API and dashboard will remain interfaces around the Rust simulation core.

They will not become a second simulation engine.

---

## Design Principles

### 1. Simulation is authoritative

Agents can propose.

The world decides.

### 2. Determinism matters

```text
same inputs → same world
```

### 3. Resources are physical

Production consumes actual inputs and creates actual outputs.

### 4. Money has a causal path

Money enters and moves through the economy through explicit economic activity.

### 5. Cognition is bounded

Agents only receive the information exposed through their observations.

### 6. Rejection is real

An agent can make a decision that the simulation refuses to execute.

### 7. Metrics observe

Measurement must not secretly alter the world.

### 8. Experiments are reproducible

Interesting behavior is not enough. It must be measurable and repeatable.

### 9. No fake intelligence

N Genesis does not require an LLM to produce interesting agent behavior.

### 10. Complexity should emerge from interaction

The goal is not to hardcode a civilization.

The goal is to build the conditions under which one can emerge.

---

## Status

N Genesis is currently a **working deterministic civilization simulation with persistent state, a causal economy, agent cognition, experimental infrastructure, and measurable emergent behavior.**

The next step is not adding more isolated mechanics.

It is making the existing civilization observable and interactive through **Phase 10 — Dashboard, API & Visualization**.
