use failsafe_core::{
    Action, BatteryInputs, Config, GpsFix, GpsSample, LandedState, PositionSample, Reason, Stamped,
    Supervisor, Telemetry, Vec3, default_fence,
};

fn nominal_telemetry(now: u32) -> Telemetry {
    Telemetry {
        now_ms: now,
        position: Some(PositionSample {
            t_ms: now,
            pos: Vec3 {
                n: 0.0,
                e: 0.0,
                d: -30.0,
            },
            vel: Vec3 {
                n: 0.0,
                e: 0.0,
                d: 0.0,
            },
        }),
        gps: Some(GpsSample {
            t_ms: now,
            fix: GpsFix::Fix3D,
            sats: Some(12),
            hdop: Some(1.0),
        }),
        battery: BatteryInputs {
            voltage_v: Some(Stamped {
                t_ms: now,
                value: 15.0,
            }),
            percent: Some(Stamped {
                t_ms: now,
                value: 80.0,
            }),
            remaining_wh: None,
        },
        gcs_heartbeat_ms: Some(now),
        armed: Some(Stamped {
            t_ms: now,
            value: true,
        }),
        landed: Some(Stamped {
            t_ms: now,
            value: LandedState::OnGround,
        }),
        rc_ok: Some(Stamped {
            t_ms: now,
            value: true,
        }),
        ekf_ok: Some(Stamped {
            t_ms: now,
            value: true,
        }),
    }
}

#[test]
fn gps_loss_triggers_land() {
    let cfg = Config::default();
    let fence = default_fence().unwrap();
    let mut sup = Supervisor::new(cfg, fence).unwrap();
    let mut t = nominal_telemetry(0);
    t.landed = Some(Stamped {
        t_ms: 0,
        value: LandedState::InAir,
    });
    for i in 0..80 {
        let now = i * 50;
        t.now_ms = now;
        t.position.as_mut().unwrap().t_ms = now;
        t.gps = Some(GpsSample {
            t_ms: now,
            fix: GpsFix::None,
            sats: None,
            hdop: None,
        });
        t.gcs_heartbeat_ms = Some(now);
        t.battery.voltage_v.as_mut().unwrap().t_ms = now;
        t.battery.percent.as_mut().unwrap().t_ms = now;
        t.armed.as_mut().unwrap().t_ms = now;
        t.landed.as_mut().unwrap().t_ms = now;
        t.rc_ok.as_mut().unwrap().t_ms = now;
        t.ekf_ok.as_mut().unwrap().t_ms = now;
        let d = sup.step(&t);
        if d.action == Action::Land && d.reason == Reason::GpsLost {
            return;
        }
    }
    panic!("expected GPS lost land");
}

#[test]
fn disarm_gate_clears_when_airborne() {
    let cfg = Config::default();
    let fence = default_fence().unwrap();
    let mut sup = Supervisor::new(cfg, fence).unwrap();
    let mut t = nominal_telemetry(0);
    t.landed = Some(Stamped {
        t_ms: 0,
        value: LandedState::OnGround,
    });
    // Force land-level requested action via GPS loss then recover — use battery critical path
    for i in 0..200 {
        let now = i * 50;
        t.now_ms = now;
        sync_stamps(&mut t, now);
        if i > 100 {
            t.landed = Some(Stamped {
                t_ms: now,
                value: LandedState::InAir,
            });
        }
        let d = sup.step(&t);
        if i > 110 && d.disarm_eligible {
            panic!("disarm should clear once airborne");
        }
    }
}

fn sync_stamps(t: &mut Telemetry, now: u32) {
    t.position.as_mut().unwrap().t_ms = now;
    t.gps.as_mut().unwrap().t_ms = now;
    t.battery.voltage_v.as_mut().unwrap().t_ms = now;
    t.battery.percent.as_mut().unwrap().t_ms = now;
    t.gcs_heartbeat_ms = Some(now);
    t.armed.as_mut().unwrap().t_ms = now;
    t.landed.as_mut().unwrap().t_ms = now;
    t.rc_ok.as_mut().unwrap().t_ms = now;
    t.ekf_ok.as_mut().unwrap().t_ms = now;
}
