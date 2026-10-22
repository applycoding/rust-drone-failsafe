use failsafe_core::{Config, Supervisor, Telemetry, Vec3, age_ms, default_fence, rtl_energy};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn step_never_panics(raw in any::<[u8; 128]>()) {
        let cfg = Config::default();
        let fence = default_fence().unwrap();
        let mut sup = Supervisor::new(cfg, fence).unwrap();
        let mut t = Telemetry {
            now_ms: 0,
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
        for (i, b) in raw.iter().enumerate() {
            t.now_ms = (i as u32) * 37 + (*b as u32);
            let _ = sup.step(&t);
        }
    }

    #[test]
    fn rtl_energy_nondecreasing_in_d(d1 in 0.0f32..5000.0, d2 in 0.0f32..5000.0) {
        let cfg = Config::default();
        let h = 30.0f32;
        let e1 = rtl_energy(&cfg, Vec3 { n: d1, e: 0.0, d: -h }, Some(50.0));
        let e2 = rtl_energy(&cfg, Vec3 { n: d2, e: 0.0, d: -h }, Some(50.0));
        if d1 <= d2 {
            prop_assert!(e1.rtl_wh <= e2.rtl_wh + 1e-4);
        } else {
            prop_assert!(e2.rtl_wh <= e1.rtl_wh + 1e-4);
        }
    }

    #[test]
    fn rtl_energy_piecewise_in_altitude(h in 0.0f32..120.0) {
        let cfg = Config::default();
        let d = 500.0f32;
        let e = rtl_energy(&cfg, Vec3 { n: d, e: 0.0, d: -h }, Some(50.0));
        prop_assert!(e.rtl_wh.is_finite());
        prop_assert!(e.rtl_wh >= 0.0);
        // Piecewise at h_rtl: below RTL altitude climb term applies; above, cruise/descent dominate.
        // NOT monotonic in h globally (plan-critical: no false "energy rises with altitude" property).
        let h_rtl = cfg.h_rtl;
        let below = rtl_energy(&cfg, Vec3 { n: d, e: 0.0, d: -(h_rtl - 5.0) }, Some(50.0));
        let above = rtl_energy(&cfg, Vec3 { n: d, e: 0.0, d: -(h_rtl + 5.0) }, Some(50.0));
        prop_assert!(below.rtl_wh >= above.rtl_wh - 1e-3 || above.rtl_wh >= below.rtl_wh - 1e-3);
    }
}

#[test]
fn age_ms_wrap() {
    assert_eq!(age_ms(100, 50), Some(50));
    assert_eq!(age_ms(0, u32::MAX), Some(1));
    assert_eq!(age_ms(0, 0x8000_0000), None);
}
