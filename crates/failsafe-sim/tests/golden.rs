#![forbid(unsafe_code)]

use failsafe_sim::{SimRunner, parse_action};
use std::fs;
use std::path::PathBuf;

fn scenario_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scenarios")
        .join(name)
}

fn run_expectations(name: &str) {
    let path = scenario_path(name);
    let toml = fs::read_to_string(&path).unwrap();
    let mut sim = SimRunner::from_toml(&toml).unwrap();
    let expects = sim.scenario.expect.clone();
    let end = sim.scenario.meta.duration_ms;
    let mut rows = Vec::new();
    while sim.t_ms <= end {
        let (row, _) = sim.tick();
        rows.push(row);
    }
    for exp in expects {
        let row = rows
            .iter()
            .find(|r| r.t_ms == exp.at_ms)
            .unwrap_or_else(|| panic!("no row at {} in {}", exp.at_ms, name));
        let want = parse_action(&exp.action).unwrap();
        let got = parse_action(&row.action).unwrap();
        assert_eq!(got, want, "scenario {} at {}ms", name, exp.at_ms);
    }
}

#[test]
fn nominal() {
    run_expectations("nominal.toml");
}

#[test]
fn gps_loss() {
    run_expectations("gps_loss.toml");
}

#[test]
fn low_battery_far() {
    run_expectations("low_battery_far.toml");
}

#[test]
fn fence_breach() {
    run_expectations("fence_breach.toml");
}

#[test]
fn lost_landing_evidence() {
    run_expectations("lost_landing_evidence.toml");
}

#[test]
fn stale_status() {
    run_expectations("stale_status.toml");
}

#[test]
fn simultaneous_triggers() {
    run_expectations("simultaneous_triggers.toml");
}

#[test]
fn clock_wrap_reboot() {
    run_expectations("clock_wrap_reboot.toml");
}

#[test]
fn marginal_hysteresis() {
    run_expectations("marginal_hysteresis.toml");
}

#[test]
fn recovery_invalid_position() {
    run_expectations("recovery_invalid_position.toml");
}
