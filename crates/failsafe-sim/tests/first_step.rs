use failsafe_core::Reason;
use failsafe_sim::SimRunner;
use std::fs;

#[test]
fn first_supervisor_step_nominal() {
    let path = format!("{}/scenarios/nominal.toml", env!("CARGO_MANIFEST_DIR"));
    let toml = fs::read_to_string(path).unwrap();
    let mut sim = SimRunner::from_toml(&toml).unwrap();
    let t = sim.snapshot_telemetry();
    let d = sim.supervisor.step(&t);
    assert_eq!(d.reason, Reason::None, "action={:?}", d.action);
}
