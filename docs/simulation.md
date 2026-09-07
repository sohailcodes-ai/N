# N Genesis — Simulation Design

## Clock

The simulation uses a deterministic clock with `StdRng` seeded by a `u64`.

Each tick = 1 simulated hour.

Time concepts:
- `tick` — monotonic counter
- `hour_of_day` — `tick % 24` (0-23)
- `day` — `(tick / 24) + 1` (1-based)
- `is_night` — hours 22-5
- `is_work_hours` — hours 9-17

## Tick Lifecycle

Each `world.tick()`:

1. **Clock advancement** — increment tick counter
2. **Needs update** — hunger/thirst/fatigue increase per tick
3. **Agent purchasing** — buy food from market if hungry
4. **Agent routines** — determine status, execute action (eat/drink/sleep/work)
5. **Labor contribution** — working agents contribute to company production
6. **Production** — companies with recipes produce goods
7. **Market update** — list food, adjust prices, clean up old transactions
8. **Wage payment** — companies pay working employees

## Needs Processing

Every agent's needs increase each tick:
- Hunger += 0.01
- Thirst += 0.02
- Fatigue += 0.008 (if awake) or -= 0.08 (sleeping) or -= 0.04 (resting)

All values clamped to `[0.0, 1.0]`.

## Routine Processing

Priority system applied to each agent:
1. Thirst > 0.6 + has water → Drinking
2. Hunger > 0.6 + has food → Eating
3. Fatigue > 0.7 + night → Sleeping
4. Fatigue > 0.5 → Resting
5. Employed + work hours → Working
6. Fatigue > 0.7 → Sleeping
7. Otherwise → Idle

Status change triggers appropriate action:
- Drinking: remove 1 water, reduce thirst by 0.35
- Eating: remove 1 food, reduce hunger by 0.3
- Working: gain skill XP, contribute labor to employer
- Sleeping: reduce fatigue by 0.08

## Production Processing

Each tick, companies with recipes:
1. Count working employees
2. Check inputs available
3. Check cooldown is 0
4. Consume inputs
5. Produce outputs (with skill multiplier)
6. Set cooldown

## Market Processing

Each tick:
1. Companies list produced food on market
2. Supply/demand computed from recent transactions
3. Price adjusts based on demand pressure vs supply pressure
4. Old transactions cleaned up (> 24 ticks old)

## Agent Purchasing

Each tick, before routines:
1. Check if agent hunger > 0.4 and food inventory < 5
2. Find cheapest food listing (prefer same district)
3. Buy affordable quantity
4. Money moves: agent → company
5. Food moves: company listing → agent inventory

## Wage Payment

Each tick:
1. For each company, find agents with status == Working
2. Pay 10 N per working employee (if company has cash)
3. Money moves: company → agent

## Determinism

All randomness uses the seeded `StdRng`. Same seed + same tick count = identical world state.

No wall-clock time, no concurrent state changes, no unseeded randomness.

## Events

State transitions emit events with:
- Unique event ID
- Tick number
- Event type enum
- Actor (agent_id/company_id)
- Cause (reason)
- Affected entities
- State snapshot (human-readable)

Events are append-only. Never deleted or modified.

## Persistence (Future Phase 7)

The simulation state can be serialized to JSON via `serialize_state()` and restored via `deserialize_state()`. This allows:
- Save/load
- Deterministic replay
- State inspection
