use failsafe_core::validate_telemetry;
use failsafe_sim::SimRunner;
use std::fs;

#[test]
fn first_tick_not_invalid() {
    let path = format!("{}/scenarios/nominal.toml", env!("CARGO_MANIFEST_DIR"));
    let toml = fs::read_to_string(path).unwrap();
    let sim = SimRunner::from_toml(&toml).unwrap();
    let t = sim.snapshot_telemetry();
    let v = validate_telemetry(&t, sim.supervisor.config(), sim.supervisor.fence());
    assert!(!v.invalid_input, "invalid telemetry at t=0");
}
