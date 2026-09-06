# N Genesis

Persistent autonomous civilization simulation.

## Current State

**Phase 2 — Agent Needs, Routines, Inventory & Skill Progression**

- Simulation engine with deterministic clock (ChaCha8Rng seeded)
- 50 agents with physical needs (hunger, thirst, fatigue)
- 10 companies with ownership and products
- 5 districts with road network
- Deterministic daily routines (drink → eat → sleep → work → idle)
- Inventory system (food, water with capacity limits)
- Consumption mechanics (eating reduces hunger, drinking reduces thirst)
- Skill progression through work (diminishing returns)
- Event system (append-only history)
- Economic invariants enforced
- 30+ unit tests covering all systems
- 7-day soak test

## How to Compile

Requires Visual Studio 2022 Build Tools with C++ workload:

```bash
# Install VS Build Tools (if not installed)
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# Check and test
cargo check --workspace
cargo test --workspace

# Run the simulation
cargo run -p n-simulation
```

## Architecture

See `docs/architecture.md` for full architecture documentation.
See `docs/agents.md` for agent architecture.
See `docs/simulation.md` for simulation design.

## Key Principles

1. **Simulation is authoritative** — AI proposes, reality decides
2. **Determinism** — same seed + same inputs = same results
3. **Causal economy** — money emerges from economic activity
4. **No LLMs in Phase 2** — deterministic survival layer first

## Roadmap

- Phase 1: Engine foundation ✅
- Phase 2: Agent needs, routines, inventory, skills ✅
- Phase 3: Resources, production, consumption
- Phase 4: Markets, supply/demand, price discovery
- Phase 5: Companies, employment, salaries
- Phase 6: Property, buildings, construction
- Phase 7: Events, persistence, replay
- Phase 8: AI cognition (LLM intents)
- Phase 9: Emergence (autonomous behavior)
- Phase 10: Dashboard, API, visualization
