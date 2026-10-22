use failsafe_core::{
    Action, BatteryInputs, Config, GpsFix, GpsSample, LandedState, PositionSample, Stamped,
    Supervisor, Telemetry, Vec3, default_fence, rtl_energy,
};

#[test]
fn worked_example_section_9() {
    let cfg = Config::default();
    let pos = Vec3 {
        n: 1500.0,
        e: 0.0,
        d: -60.0,
    };
    let e = rtl_energy(&cfg, pos, None);
    assert!((e.rtl_wh - 21.5625).abs() < 1e-3);
    assert!((e.land_wh - 3.3333).abs() < 1e-3);
    let marginal_cutoff = e.rtl_wh + cfg.reserve_wh + cfg.energy_marginal_wh;
    assert!((marginal_cutoff - 34.5625).abs() < 1e-3);
    let with_reserve = e.rtl_wh + cfg.reserve_wh;
    assert!((with_reserve - 29.5625).abs() < 1e-3);
}

fn nominal_in_fence(now: u32, remaining_wh: f32) -> Telemetry {
    Telemetry {
        now_ms: now,
        position: Some(PositionSample {
            t_ms: now,
            pos: Vec3 {
                n: 400.0,
                e: 0.0,
                d: -60.0,
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
                value: 14.8,
            }),
            percent: Some(Stamped {
                t_ms: now,
                value: remaining_wh / 40.0 * 100.0,
            }),
            remaining_wh: Some(Stamped {
                t_ms: now,
                value: remaining_wh,
            }),
        },
        gcs_heartbeat_ms: Some(now),
        armed: Some(Stamped {
            t_ms: now,
            value: true,
        }),
        landed: Some(Stamped {
            t_ms: now,
            value: LandedState::InAir,
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

fn sync(t: &mut Telemetry, now: u32) {
    t.now_ms = now;
    t.position.as_mut().unwrap().t_ms = now;
    t.gps.as_mut().unwrap().t_ms = now;
    t.battery.voltage_v.as_mut().unwrap().t_ms = now;
    t.battery.percent.as_mut().unwrap().t_ms = now;
    t.battery.remaining_wh.as_mut().unwrap().t_ms = now;
    t.battery.remaining_wh.as_mut().unwrap().value = t.battery.remaining_wh.unwrap().value;
    t.gcs_heartbeat_ms = Some(now);
    t.armed.as_mut().unwrap().t_ms = now;
    t.landed.as_mut().unwrap().t_ms = now;
    t.rc_ok.as_mut().unwrap().t_ms = now;
    t.ekf_ok.as_mut().unwrap().t_ms = now;
}

fn run_until_action(wh: f32, want: Action) {
    let cfg = Config::default();
    let fence = default_fence().unwrap();
    let mut sup = Supervisor::new(cfg, fence).unwrap();
    let mut t = nominal_in_fence(0, wh);
    for i in 0..200 {
        let now = i * 50;
        sync(&mut t, now);
        t.battery.remaining_wh.as_mut().unwrap().value = wh;
        let d = sup.step(&t);
        if d.action == want {
            return;
        }
    }
    panic!("expected {:?} for {} Wh", want, wh);
}

#[test]
fn energy_margin_nominal_rtl_land() {
    let cfg = Config::default();
    let pos = Vec3 {
        n: 1500.0,
        e: 0.0,
        d: -60.0,
    };
    let e = rtl_energy(&cfg, pos, Some(40.0));
    assert!(e.margin_wh.unwrap() > cfg.energy_marginal_wh);
    let e33 = rtl_energy(&cfg, pos, Some(33.0));
    assert!(e33.margin_wh.unwrap() < cfg.energy_marginal_wh);
    assert!(e33.margin_wh.unwrap() >= 0.0);
    let e26 = rtl_energy(&cfg, pos, Some(26.0));
    assert!(e26.margin_wh.unwrap() < 0.0);

    run_until_action(40.0, Action::Nominal);
    run_until_action(20.0, Action::ReturnToLaunch);
    run_until_action(15.0, Action::Land);
}
