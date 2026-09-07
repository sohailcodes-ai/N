# N Genesis

Persistent autonomous civilization simulation.

## Current State

**Phase 5 — Employment & Labor Markets**

- Simulation engine with deterministic clock (ChaCha8Rng seeded)
- 50 agents with physical needs (hunger, thirst, fatigue)
- 3 companies (2 food producers, 1 service) with dynamic employment
- 5 districts with road network
- Deterministic daily routines (drink → eat → sleep → work → idle)
- Inventory system (food, water with capacity limits)
- Consumption mechanics (eating reduces hunger, drinking reduces thirst)
- Skill progression through work (diminishing returns)
- **Resource system** (Food, Water, RawFood with typed inventory)
- **Production system** (recipes, input/output, labor requirement, cooldown)
- **Market system** (listings, transactions, supply/demand pricing)
- **Agent purchasing** (hunger-driven food buying from market)
- **Wage system** (companies pay working employees hourly during work hours)
- **Money flow** (agent wages → agent purchases → company revenue)
- **Employment contracts** (hiring, firing, skill requirements)
- **Labor market** (job openings, competitive hiring, wage-based selection)
- **Company profitability** (revenue tracking, loss streaks, insolvency)
- **Insolvency** (companies below cash threshold close, employees released)
- Event system (append-only history with economic events)
- Economic invariants enforced (no negative inventory/money, money conserved)
- 91 unit tests covering all systems
- 30-day soak test with full economic cycle

## Performance

| Metric | Time |
|--------|------|
| `cargo test -p n-simulation` (debug) | ~96s |
| `cargo test -p n-simulation --release` | ~15s |
| 30-day soak (release) | ~10s |
| 30-day soak (debug) | ~96s |

## How to Compile

Requires Visual Studio 2022 Build Tools with C++ workload:

```bash
cargo check --workspace
cargo test -p n-simulation
cargo run -p n-simulation --release
```

## Architecture

See `docs/architecture.md` for full architecture documentation.
See `docs/agents.md` for agent architecture.
See `docs/simulation.md` for simulation design.
See `docs/economy.md` for economy and market design.
See `docs/employment.md` for employment and labor market design.

## Key Principles

1. **Simulation is authoritative** — AI proposes, reality decides
2. **Determinism** — same seed + same inputs = same results
3. **Causal economy** — money emerges from economic activity
4. **No LLMs in Phase 5** — deterministic survival + economic layer
5. **Physical resources** — production consumes real inputs, creates real outputs

## Economic Causality

```
Agent becomes hungry
→ checks food inventory
→ needs food
→ purchases food from market
→ money moves to seller (company)
→ food moves to agent
→ agent consumes food
→ hunger decreases

Company receives raw food
→ employees provide labor
→ production runs
→ raw food consumed
→ food produced
→ food enters company inventory
→ company lists food on market
→ agent purchases food
→ company receives revenue
→ company pays wages
→ company tracks profit/loss
→ profitable: hire more workers
→ unprofitable: loss streak grows
→ insolvency: company closes, workers released
```

## Roadmap

- Phase 1: Engine foundation ✅
- Phase 2: Agent needs, routines, inventory, skills ✅
- Phase 3: Resources, production, consumption, market, economy ✅
- Phase 4: Advanced market dynamics, multiple resource types ✅
- Phase 5: Employment contracts, hiring/firing, ownership ✅
- Phase 6: Property, buildings, construction
- Phase 7: Events, persistence, replay
- Phase 8: AI cognition (LLM intents)
- Phase 9: Emergence (autonomous behavior)
- Phase 10: Dashboard, API, visualization
