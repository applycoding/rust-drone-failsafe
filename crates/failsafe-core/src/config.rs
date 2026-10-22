use crate::geometry::{ConvexPolygon, FenceError, Geofence};
use crate::types::Point2;

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub cell_count: u8,
    pub usable_capacity_wh: f32,
    pub rc_required: bool,
    pub pos_fresh_ms: u32,
    pub gps_fresh_ms: u32,
    pub status_fresh_ms: u32,
    pub battery_fresh_ms: u32,
    pub recovery_hold_ms: u32,
    pub gcs_warn_ms: u32,
    pub gcs_hold_ms: u32,
    pub gcs_rtl_ms: u32,
    pub gcs_clear_ms: u32,
    pub gcs_clear_hold_ms: u32,
    pub rc_lost_set_ms: u32,
    pub rc_lost_clear_ms: u32,
    pub gps_degraded_set_ms: u32,
    pub gps_degraded_clear_ms: u32,
    pub gps_lost_set_ms: u32,
    pub gps_lost_clear_ms: u32,
    pub ekf_unhealthy_set_ms: u32,
    pub ekf_unhealthy_clear_ms: u32,
    pub fence_approach_m: f32,
    pub fence_approach_clear_m: f32,
    pub fence_breach_set_ms: u32,
    pub fence_breach_clear_m: f32,
    pub fence_breach_clear_ms: u32,
    pub alt_max_clear_m: f32,
    pub alt_max_set_ms: u32,
    pub alt_max_clear_hold_ms: u32,
    pub alt_min_home_radius_m: f32,
    pub alt_min_set_ms: u32,
    pub alt_min_clear_m: f32,
    pub alt_min_clear_ms: u32,
    pub stale_battery_clear_ms: u32,
    pub battery_warn_pct: f32,
    pub battery_warn_clear_pct: f32,
    pub battery_warn_set_ms: u32,
    pub battery_warn_clear_ms: u32,
    pub battery_low_pct: f32,
    pub battery_low_clear_pct: f32,
    pub battery_low_set_ms: u32,
    pub battery_low_clear_ms: u32,
    pub battery_critical_pct: f32,
    pub battery_critical_v_per_cell: f32,
    pub battery_critical_set_ms: u32,
    pub energy_marginal_wh: f32,
    pub energy_marginal_clear_wh: f32,
    pub energy_marginal_set_ms: u32,
    pub energy_marginal_clear_ms: u32,
    pub energy_infeasible_set_ms: u32,
    pub invalid_input_clear_ms: u32,
    pub disarm_hold_ms: u32,
    pub v_cruise: f32,
    pub p_cruise: f32,
    pub k_wind: f32,
    pub v_climb: f32,
    pub p_climb: f32,
    pub v_desc: f32,
    pub p_desc: f32,
    pub h_rtl: f32,
    pub reserve_wh: f32,
    pub v_max: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cell_count: 4,
            usable_capacity_wh: 40.0,
            rc_required: true,
            pos_fresh_ms: 500,
            gps_fresh_ms: 1000,
            status_fresh_ms: 1000,
            battery_fresh_ms: 2000,
            recovery_hold_ms: 5000,
            gcs_warn_ms: 3000,
            gcs_hold_ms: 5000,
            gcs_rtl_ms: 15000,
            gcs_clear_ms: 1500,
            gcs_clear_hold_ms: 3000,
            rc_lost_set_ms: 1000,
            rc_lost_clear_ms: 3000,
            gps_degraded_set_ms: 2000,
            gps_degraded_clear_ms: 5000,
            gps_lost_set_ms: 1500,
            gps_lost_clear_ms: 5000,
            ekf_unhealthy_set_ms: 1000,
            ekf_unhealthy_clear_ms: 5000,
            fence_approach_m: 10.0,
            fence_approach_clear_m: 15.0,
            fence_breach_set_ms: 500,
            fence_breach_clear_m: 5.0,
            fence_breach_clear_ms: 5000,
            alt_max_clear_m: 5.0,
            alt_max_set_ms: 500,
            alt_max_clear_hold_ms: 3000,
            alt_min_home_radius_m: 20.0,
            alt_min_set_ms: 1000,
            alt_min_clear_m: 2.0,
            alt_min_clear_ms: 1000,
            stale_battery_clear_ms: 3000,
            battery_warn_pct: 30.0,
            battery_warn_clear_pct: 33.0,
            battery_warn_set_ms: 2000,
            battery_warn_clear_ms: 5000,
            battery_low_pct: 20.0,
            battery_low_clear_pct: 23.0,
            battery_low_set_ms: 2000,
            battery_low_clear_ms: 5000,
            battery_critical_pct: 10.0,
            battery_critical_v_per_cell: 3.3,
            battery_critical_set_ms: 3000,
            energy_marginal_wh: 5.0,
            energy_marginal_clear_wh: 7.0,
            energy_marginal_set_ms: 2000,
            energy_marginal_clear_ms: 5000,
            energy_infeasible_set_ms: 2000,
            invalid_input_clear_ms: 3000,
            disarm_hold_ms: 2000,
            v_cruise: 10.0,
            p_cruise: 350.0,
            k_wind: 1.25,
            v_climb: 2.0,
            p_climb: 400.0,
            v_desc: 1.5,
            p_desc: 300.0,
            h_rtl: 30.0,
            reserve_wh: 8.0,
            v_max: 25.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    Fence(FenceError),
    HomeOutsideFence,
    RtlAltOutsideFence,
    NonFiniteParam,
    NonPositiveSpeed,
    NegativePower,
    BadCellCount,
    BadCapacity,
    BadThresholdOrder,
    DurationOutOfRange,
}

pub fn validate_config(cfg: &Config, fence: &Geofence) -> Result<(), ConfigError> {
    if !(1..=14).contains(&cfg.cell_count) {
        return Err(ConfigError::BadCellCount);
    }
    if !cfg.usable_capacity_wh.is_finite() || cfg.usable_capacity_wh <= 0.0 {
        return Err(ConfigError::BadCapacity);
    }
    macro_rules! fin {
        ($x:expr) => {
            if !$x.is_finite() {
                return Err(ConfigError::NonFiniteParam);
            }
        };
    }
    fin!(cfg.v_cruise);
    fin!(cfg.v_climb);
    fin!(cfg.v_desc);
    fin!(cfg.p_cruise);
    fin!(cfg.p_climb);
    fin!(cfg.p_desc);
    fin!(cfg.reserve_wh);
    fin!(cfg.h_rtl);
    if cfg.v_cruise <= 0.0 || cfg.v_climb <= 0.0 || cfg.v_desc <= 0.0 {
        return Err(ConfigError::NonPositiveSpeed);
    }
    if cfg.p_cruise < 0.0 || cfg.p_climb < 0.0 || cfg.p_desc < 0.0 || cfg.reserve_wh < 0.0 {
        return Err(ConfigError::NegativePower);
    }
    if cfg.battery_warn_pct <= cfg.battery_low_pct
        || cfg.battery_low_pct <= cfg.battery_critical_pct
        || cfg.battery_warn_clear_pct <= cfg.battery_warn_pct
        || cfg.battery_low_clear_pct <= cfg.battery_low_pct
    {
        return Err(ConfigError::BadThresholdOrder);
    }
    let durations = [
        cfg.recovery_hold_ms,
        cfg.gcs_warn_ms,
        cfg.disarm_hold_ms,
        cfg.energy_infeasible_set_ms,
    ];
    for d in durations {
        if !(1..=600_000).contains(&d) {
            return Err(ConfigError::DurationOutOfRange);
        }
    }
    let home = Point2 { n: 0.0, e: 0.0 };
    if !fence.poly.contains(home) {
        return Err(ConfigError::HomeOutsideFence);
    }
    if cfg.h_rtl < fence.min_alt_m || cfg.h_rtl > fence.max_alt_m {
        return Err(ConfigError::RtlAltOutsideFence);
    }
    Ok(())
}

pub fn default_fence() -> Result<Geofence, FenceError> {
    let pts = [
        Point2 {
            n: -500.0,
            e: -500.0,
        },
        Point2 {
            n: -500.0,
            e: 500.0,
        },
        Point2 { n: 500.0, e: 500.0 },
        Point2 {
            n: 500.0,
            e: -500.0,
        },
    ];
    let poly = ConvexPolygon::new(&pts)?;
    Geofence::new(poly, 5.0, 120.0)
}
