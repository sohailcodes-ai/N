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
1. Advance clock by 1 tick
2. Process agent needs (increase hunger/thirst/fatigue)
3. Process agent routines (determine status, execute action)

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
- Working: gain skill XP
- Sleeping: reduce fatigue by 0.08

## Determinism

All randomness uses the seeded `StdRng`. Same seed + same tick count = identical world state.

No wall-clock time, no concurrent state changes, no unseeded randomness.

## Events

State transitions emit events with:
- Unique event ID
- Tick number
- Event type enum
- Actor (agent_id)
- Cause (reason)
- Affected entities
- State snapshot (human-readable)

Events are append-only. Never deleted or modified.

## Persistence (Future Phase 7)

The simulation state can be serialized to JSON via `serialize_state()` and restored via `deserialize_state()`. This allows:
- Save/load
- Deterministic replay
- State inspection
