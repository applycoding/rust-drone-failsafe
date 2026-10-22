use failsafe_core::Action;
use failsafe_sim::SimRunner;
use std::fs;

#[test]
fn gps_kill_control_eventually_lands() {
    let path = format!("{}/scenarios/gps_loss.toml", env!("CARGO_MANIFEST_DIR"));
    let toml = fs::read_to_string(path).unwrap();
    let mut sim = SimRunner::from_toml(&toml).unwrap();
    sim.set_controls(false, false, true, false, 0.01);
    for _ in 0..200 {
        let (_, d) = sim.tick();
        if d.action == Action::Land {
            return;
        }
    }
    panic!("expected land after gps kill");
}
