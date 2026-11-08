use crate::{SimulationClock, SimulationState, World};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationSnapshot {
    pub version: u32,
    pub seed: u64,
    pub tick: u64,
    pub state: SimulationState,
    pub clock: SimulationClock,
}

impl SimulationSnapshot {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn from_world(world: &World) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            seed: world.state.metadata.seed,
            tick: world.clock.tick,
            state: world.state.clone(),
            clock: world.clock.clone(),
        }
    }

    pub fn into_world(self) -> Result<World, String> {
        if self.version != Self::CURRENT_VERSION {
            return Err(format!(
                "unsupported snapshot version: {} (expected {})",
                self.version,
                Self::CURRENT_VERSION
            ));
        }

        if self.tick != self.clock.tick || self.tick != self.state.tick {
            return Err(format!(
                "snapshot tick mismatch: snapshot={}, clock={}, state={}",
                self.tick, self.clock.tick, self.state.tick
            ));
        }

        if self.seed != self.state.metadata.seed {
            return Err(format!(
                "snapshot seed mismatch: snapshot={}, state={}",
                self.seed, self.state.metadata.seed
            ));
        }

        Ok(World {
            state: self.state,
            clock: self.clock,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    pub matched: bool,
    pub expected_event_count: usize,
    pub actual_event_count: usize,
    pub mismatch_index: Option<usize>,
}

impl ReplayReport {
    pub fn success(&self) -> bool {
        self.matched
    }
}

pub fn snapshot(world: &World) -> SimulationSnapshot {
    SimulationSnapshot::from_world(world)
}

pub fn snapshot_to_json(world: &World) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&snapshot(world))
}

pub fn snapshot_from_json(json: &str) -> Result<SimulationSnapshot, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn restore_from_json(json: &str) -> Result<World, String> {
    snapshot_from_json(json)?.into_world()
}

pub fn save_snapshot<P: AsRef<Path>>(world: &World, path: P) -> Result<(), String> {
    let json = snapshot_to_json(world).map_err(|e| format!("serialize snapshot: {e}"))?;
    fs::write(path, json).map_err(|e| format!("write snapshot: {e}"))
}

pub fn load_snapshot<P: AsRef<Path>>(path: P) -> Result<SimulationSnapshot, String> {
    let json = fs::read_to_string(path).map_err(|e| format!("read snapshot: {e}"))?;
    snapshot_from_json(&json).map_err(|e| format!("deserialize snapshot: {e}"))
}

pub fn load_world<P: AsRef<Path>>(path: P) -> Result<World, String> {
    load_snapshot(path)?.into_world()
}

/// Replay from genesis and compare the complete event stream.
///
/// This is intentionally stronger than comparing final state: two runs can
/// converge to an equal state while taking different causal paths.
pub fn verify_replay(
    seed: u64,
    num_agents: u32,
    num_companies: u32,
    ticks: u64,
    expected_events: &[crate::Event],
) -> ReplayReport {
    let mut world = World::initialize(seed, num_agents, num_companies);

    for _ in 0..ticks {
        world.tick();
    }

    let actual = &world.state.events;
    let common_len = actual.len().min(expected_events.len());
    let mismatch_index = (0..common_len).find(|&i| actual[i] != expected_events[i]);

    ReplayReport {
        matched: mismatch_index.is_none() && actual.len() == expected_events.len(),
        expected_event_count: expected_events.len(),
        actual_event_count: actual.len(),
        mismatch_index: mismatch_index.or_else(|| {
            (actual.len() != expected_events.len()).then_some(common_len)
        }),
    }
}

pub fn record_run(
    seed: u64,
    num_agents: u32,
    num_companies: u32,
    ticks: u64,
) -> Vec<crate::Event> {
    let mut world = World::initialize(seed, num_agents, num_companies);

    for _ in 0..ticks {
        world.tick();
    }

    world.state.events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trip_preserves_world() {
        let mut world = World::initialize(1234, 10, 3);
        for _ in 0..25 {
            world.tick();
        }

        let json = snapshot_to_json(&world).expect("serialize");
        let restored = restore_from_json(&json).expect("restore");

        assert_eq!(restored.state, world.state);
        assert_eq!(restored.clock.tick, world.clock.tick);
        assert_eq!(restored.clock.speed, world.clock.speed);
    }

    #[test]
    fn snapshot_preserves_rng_sequence() {
        let mut world = World::initialize(42, 2, 1);
        for _ in 0..10 {
            let _ = world.clock.random();
        }

        let json = snapshot_to_json(&world).expect("serialize");
        let mut restored = restore_from_json(&json).expect("restore");

        assert_eq!(world.clock.random(), restored.clock.random());
        assert_eq!(world.clock.random(), restored.clock.random());
        assert_eq!(world.clock.random(), restored.clock.random());
    }

    #[test]
    fn resume_after_snapshot_is_deterministic() {
        let mut uninterrupted = World::initialize(999, 10, 3);
        for _ in 0..100 {
            uninterrupted.tick();
        }

        let mut paused = World::initialize(999, 10, 3);
        for _ in 0..50 {
            paused.tick();
        }

        let json = snapshot_to_json(&paused).expect("serialize");
        let mut resumed = restore_from_json(&json).expect("restore");

        for _ in 0..50 {
            resumed.tick();
        }

        assert_eq!(resumed.state, uninterrupted.state);
        assert_eq!(resumed.clock.tick, uninterrupted.clock.tick);
    }

    #[test]
    fn rejects_unknown_snapshot_version() {
        let mut value = serde_json::to_value(snapshot(&World::initialize(1, 1, 1))).unwrap();
        value["version"] = serde_json::json!(999);

        let error = restore_from_json(&value.to_string()).unwrap_err();
        assert!(error.contains("unsupported snapshot version"));
    }

    #[test]
    fn replay_verifier_accepts_identical_run() {
        let expected = record_run(1234, 10, 3, 100);
        let report = verify_replay(1234, 10, 3, 100, &expected);

        assert!(report.success());
        assert_eq!(report.mismatch_index, None);
    }

    #[test]
    fn replay_verifier_finds_divergence() {
        let mut expected = record_run(1234, 10, 3, 100);
        expected.pop();

        let report = verify_replay(1234, 10, 3, 100, &expected);

        assert!(!report.success());
        assert_eq!(report.mismatch_index, Some(expected.len()));
    }
}
