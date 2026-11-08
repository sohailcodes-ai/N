use crate::{SimulationState, World};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationSnapshot {
    pub seed: u64,
    pub tick: u64,
    pub state: SimulationState,
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

/// Capture the authoritative simulation state plus the seed/tick needed to
/// identify exactly where the snapshot belongs in a deterministic run.
pub fn snapshot(world: &World) -> SimulationSnapshot {
    SimulationSnapshot {
        seed: world.state.metadata.seed,
        tick: world.clock.tick,
        state: world.state.clone(),
    }
}

pub fn snapshot_to_json(world: &World) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&snapshot(world))
}

pub fn snapshot_from_json(json: &str) -> Result<SimulationSnapshot, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn save_snapshot<P: AsRef<Path>>(world: &World, path: P) -> Result<(), String> {
    let json = snapshot_to_json(world).map_err(|e| format!("serialize snapshot: {e}"))?;
    fs::write(path, json).map_err(|e| format!("write snapshot: {e}"))
}

pub fn load_snapshot<P: AsRef<Path>>(path: P) -> Result<SimulationSnapshot, String> {
    let json = fs::read_to_string(path).map_err(|e| format!("read snapshot: {e}"))?;
    snapshot_from_json(&json).map_err(|e| format!("deserialize snapshot: {e}"))
}

/// Replay the simulation from genesis and compare the complete event stream
/// with an expected stream. This intentionally compares events rather than
/// only final state: two simulations can converge to the same state while
/// taking different causal paths.
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

    let matched = mismatch_index.is_none() && actual.len() == expected_events.len();

    ReplayReport {
        matched,
        expected_event_count: expected_events.len(),
        actual_event_count: actual.len(),
        mismatch_index: mismatch_index.or_else(|| {
            if actual.len() != expected_events.len() {
                Some(common_len)
            } else {
                None
            }
        }),
    }
}

/// Build a canonical event stream for a deterministic run.
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
    use crate::World;

    #[test]
    fn snapshot_round_trip_preserves_state() {
        let mut world = World::initialize(1234, 10, 3);
        for _ in 0..20 {
            world.tick();
        }

        let json = snapshot_to_json(&world).expect("serialize");
        let restored = snapshot_from_json(&json).expect("deserialize");

        assert_eq!(restored.seed, 1234);
        assert_eq!(restored.tick, world.clock.tick);
        assert_eq!(restored.state, world.state);
    }

    #[test]
    fn deterministic_runs_have_identical_event_streams() {
        let first = record_run(1234, 10, 3, 100);
        let second = record_run(1234, 10, 3, 100);

        assert_eq!(first, second);
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
