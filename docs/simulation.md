# N Genesis — Simulation Design

## Clock

The simulation uses a deterministic clock with `StdRng` seeded by a `u64`.

Each tick = 1 simulated minute (1440 ticks/day).

Time concepts:
- `tick` — monotonic counter
- `hour_of_day` — `(tick / 60) % 24` (0-23)
- `minute_of_hour` — `tick % 60` (0-59)
- `day` — `(tick / 1440) + 1` (1-based)
- `is_night` — hours 22-5
- `is_work_hours` — hours 9-17 (ticks 540-959)

## Tick Lifecycle

Each `world.tick()`:

1. **Clock advancement** — increment tick counter
2. **Needs update** — hunger/thirst/fatigue increase per tick
3. **Agent purchasing** — buy food from market if hungry
4. **Agent routines** — determine status, execute action (eat/drink/sleep/work)
5. **Labor contribution** — working agents contribute to company production
6. **Production** — companies with recipes produce goods
7. **Market update** — list food/water, adjust prices, clean up old transactions
8. **Wage payment** — companies pay working employees (hourly, work hours only)
9. **Job evaluation** — unemployed agents apply for job openings
10. **Hiring** — understaffed companies create job offers
11. **Firing** — unprofitable companies fire workers
12. **Profit tracking** — companies compute revenue vs expenses
13. **Insolvency** — close companies below cash threshold
14. **Cleanup** — prune stale job openings and dead employments

## Needs Processing

Every agent's needs increase each tick:
- Hunger += 0.0069 (1/1440)
- Thirst += 0.0139 (2/1440)
- Fatigue += 0.0056 (0.8/1440 while awake) or -= 0.0556 (8/1440 sleeping) or -= 0.0278 (4/1440 resting)

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
- Sleeping: reduce fatigue by 0.0556/tick

## Employment Processing

### Job Evaluation (per tick, sorted by agent id)
1. Each unemployed agent scans job openings
2. Selects best opening (highest wage, meets skill requirement)
3. Skill cost deducted (0.02)
4. Employment record created
5. Agent.employer set

### Hiring (per tick, sorted by company id)
1. Check hire cooldown (60 ticks)
2. Compute needed workers
3. Create job openings for gaps

### Wage Payment (per tick, work hours only, hourly)
1. Check `is_work_hours` and `minute_of_hour == 0`
2. For each company, pay employees their recorded wage
3. Company cash decreases, agent money increases

### Firing (per tick)
1. Track company profit/loss
2. If loss streak >= 3, terminate all employees
3. Mark company inactive

## Production Processing

Each tick, companies with recipes:
1. Count working employees (single scan)
2. Check inputs available
3. Check cooldown is 0
4. Consume inputs
5. Produce outputs (with skill multiplier)
6. Set cooldown

## Market Processing

Each tick:
1. Companies list produced food/water on market
2. Supply/demand computed from recent transactions (single-pass)
3. Price adjusts based on demand pressure vs supply pressure
4. Old transactions cleaned up (> 24 hours old)
5. Prune zero-quantity listings

## Agent Purchasing

Each tick, before routines:
1. Check if agent hunger > 0.4 and food inventory < 5
2. Find cheapest food listing (prefer same district)
3. Buy affordable quantity
4. Money moves: agent → company
5. Food moves: company listing → agent inventory

## Water Purchasing

Each tick, before routines:
1. Check if agent thirst > 0.4 and water inventory < 3
2. Find cheapest water listing
3. Buy affordable quantity
4. Money moves: agent → company
5. Water moves: company listing → agent inventory

## Determinism

All randomness uses the seeded `StdRng`. Same seed + same tick count = identical world state.

No wall-clock time, no concurrent state changes, no unseeded randomness.

All iterating over HashMaps must be sorted by key to ensure deterministic ordering.

## Events

State transitions emit events with:
- Unique event ID
- Tick number
- Event type enum
- Actor (agent_id/company_id)
- Cause (reason)
- Affected entities
- State snapshot (human-readable)

Events are append-only. Capped at 50,000 entries (drains to 25,000).

## Performance

Key optimizations:
- `job_openings` pruned when `openings == 0` (prevents O(T²) growth)
- `employments` pruned when `active == false` (prevents O(T²) scan)
- `events` capped at 50,000 (drains to 25,000 to prevent memory growth)
- Single-pass computation for market transactions (no clone)
- Merged labor scans in production (single `employments_for_company` per company)
- Sorted iterators for determinism

## Persistence (Future Phase 7)

The simulation state can be serialized to JSON via `serialize_state()` and restored via `deserialize_state()`. This allows:
- Save/load
- Deterministic replay
- State inspection
