use failsafe_core::{Action, Config, Supervisor, Telemetry, default_fence};

#[test]
fn never_hold_or_rtl_without_valid_position() {
    let cfg = Config::default();
    let fence = default_fence().unwrap();
    let mut sup = Supervisor::new(cfg, fence).unwrap();
    let t = Telemetry {
        now_ms: 5000,
        position: None,
        gps: None,
        battery: failsafe_core::BatteryInputs {
            voltage_v: None,
            percent: None,
            remaining_wh: None,
        },
        gcs_heartbeat_ms: None,
        armed: None,
        landed: None,
        rc_ok: None,
        ekf_ok: None,
    };
    let d = sup.step(&t);
    assert!(
        d.action != Action::Hold && d.action != Action::ReturnToLaunch,
        "got {:?}",
        d.action
    );
}
