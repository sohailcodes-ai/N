# N Genesis — Persistence & Replay

Phase 7 makes the simulation restartable without losing deterministic continuity.

## Snapshot Model

A `SimulationSnapshot` contains:

- `version` — snapshot schema version
- `seed` — simulation seed
- `tick` — authoritative simulation tick
- `state` — complete `SimulationState`
- `clock` — clock speed, tick and RNG state

The RNG state is serialized directly. Saving only the seed is not sufficient for exact continuation after random numbers have already been consumed.

## APIs

```rust
let json = snapshot_to_json(&world)?;
let world = restore_from_json(&json)?;

save_snapshot(&world, "world.json")?;
let world = load_world("world.json")?;
```

`restore_from_json` validates snapshot schema version, tick consistency, and seed consistency.

## Exact Resume

```text
World at tick 50
       |
       v
   serialize
       |
       v
   world.json
       |
       v
  deserialize
       |
       v
World at tick 50
       |
       v
     tick()
       |
       v
same state as uninterrupted run at tick 51
```

The serialized RNG state makes this stronger than seed-based reconstruction.

## Replay Verification

`record_run` generates a canonical event stream for a seed and tick count.

`verify_replay` runs the same simulation and compares events in order.

This detects event-order divergence, missing events, extra events, different event payloads, and different causal paths.

Final-state equality is insufficient: two simulations can converge to the same balances while taking different causal paths. N therefore treats the event stream as part of deterministic verification.

## Versioning

Snapshots are versioned from day one.

Current version: `1`

A future schema migration can explicitly accept older versions instead of attempting to deserialize incompatible state.

## Next

Phase 8 introduces the first cognition layer:

```text
Observation
    ↓
Agent cognition
    ↓
Intent
    ↓
Validation
    ↓
Simulation rules
    ↓
State transition
    ↓
Event
```

The simulation remains authoritative. AI does not receive direct mutation access.