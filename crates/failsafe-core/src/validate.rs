use crate::config::Config;
use crate::geometry::Geofence;
use crate::time::{age_ms, is_fresh};
use crate::types::{BatteryInputs, GpsSample, LandedState, PositionSample, Stamped, Telemetry};
use libm::fabsf;

#[derive(Clone, Copy, Debug, Default)]
pub struct GlitchState {
    pub last_accepted: Option<PositionSample>,
    pub reject_streak: u8,
}

pub struct ValidatedTelemetry {
    pub telemetry: Telemetry,
    pub invalid_input: bool,
}

pub fn validate_telemetry(raw: &Telemetry, cfg: &Config, fence: &Geofence) -> ValidatedTelemetry {
    let mut t = *raw;
    let mut invalid = false;

    if let Some(pos) = t.position {
        if !validate_position_sample(&pos, fence) {
            t.position = None;
            invalid = true;
        } else {
            t.position = Some(pos);
        }
    }

    if let Some(gps) = t.gps
        && !validate_gps(&gps)
    {
        t.gps = None;
        invalid = true;
    }

    if let Some(v) = t.battery.voltage_v
        && !validate_voltage(v.value)
    {
        t.battery.voltage_v = None;
        invalid = true;
    }
    if let Some(p) = t.battery.percent
        && !validate_percent(p.value)
    {
        t.battery.percent = None;
        invalid = true;
    }
    if let Some(r) = t.battery.remaining_wh
        && !validate_remaining(r.value, cfg.usable_capacity_wh)
    {
        t.battery.remaining_wh = None;
        invalid = true;
    }

    if let Some(a) = t.armed
        && !a.value
        && !a.value
    {
        // bool always ok
    }

    ValidatedTelemetry {
        telemetry: t,
        invalid_input: invalid,
    }
}

fn validate_position_sample(pos: &PositionSample, fence: &Geofence) -> bool {
    let p = pos.pos;
    if !p.n.is_finite() || !p.e.is_finite() || !p.d.is_finite() {
        return false;
    }
    if fabsf(p.n) > 10_000.0 || fabsf(p.e) > 10_000.0 {
        return false;
    }
    let alt = p.alt_m();
    if alt < -100.0 || alt > 2000.0 {
        return false;
    }
    let v = pos.vel;
    if !v.n.is_finite() || !v.e.is_finite() || !v.d.is_finite() {
        return false;
    }
    let speed = libm::sqrtf(v.n * v.n + v.e * v.e + v.d * v.d);
    if speed > 100.0 {
        return false;
    }
    let pt = crate::types::Point2 { n: p.n, e: p.e };
    if !fence.poly.contains(pt) && fence.poly.signed_distance(pt) < -1000.0 {
        // allow outside fence for breach detection — only reject non-finite envelope
    }
    true
}

fn validate_gps(gps: &GpsSample) -> bool {
    if let Some(h) = gps.hdop
        && (!h.is_finite() || h < 0.0 || h > 100.0)
    {
        return false;
    }
    true
}

fn validate_voltage(v: f32) -> bool {
    v.is_finite() && (0.0..=100.0).contains(&v)
}

fn validate_percent(p: f32) -> bool {
    p.is_finite() && (0.0..=100.0).contains(&p)
}

fn validate_remaining(r: f32, cap: f32) -> bool {
    r.is_finite() && r >= 0.0 && r <= cap * 1.1
}

pub fn apply_glitch_filter(
    glitch: &mut GlitchState,
    now_ms: u32,
    pos: Option<PositionSample>,
    cfg: &Config,
) -> Option<PositionSample> {
    let sample = pos?;

    let accept_gap = match glitch.last_accepted {
        None => true,
        Some(last) => {
            let gap = age_ms(now_ms, last.t_ms);
            if gap.map(|g| g > 2000).unwrap_or(true) || glitch.reject_streak >= 3 {
                true
            } else {
                let dt_s = age_ms(now_ms, last.t_ms)
                    .map(|g| g as f32 / 1000.0)
                    .unwrap_or(0.0);
                let dn = sample.pos.n - last.pos.n;
                let de = sample.pos.e - last.pos.e;
                let dd = sample.pos.d - last.pos.d;
                let dist = libm::sqrtf(dn * dn + de * de + dd * dd);
                let max_jump = cfg.v_max * dt_s * 2.0 + 5.0;
                dist <= max_jump
            }
        }
    };

    if accept_gap {
        glitch.last_accepted = Some(sample);
        glitch.reject_streak = 0;
        Some(sample)
    } else {
        glitch.reject_streak = glitch.reject_streak.saturating_add(1);
        None
    }
}

pub fn position_valid(
    now_ms: u32,
    pos: Option<PositionSample>,
    ekf_ok: Option<Stamped<bool>>,
    cfg: &Config,
    fence: &Geofence,
) -> bool {
    let Some(pos) = pos else {
        return false;
    };
    if !is_fresh(now_ms, pos.t_ms, cfg.pos_fresh_ms) {
        return false;
    }
    let _ = fence;
    matches!(
        ekf_ok,
        Some(e) if is_fresh(now_ms, e.t_ms, cfg.status_fresh_ms) && e.value
    )
}

pub fn fresh_battery_percent(now_ms: u32, battery: &BatteryInputs, cfg: &Config) -> Option<f32> {
    if let Some(r) = battery.remaining_wh
        && is_fresh(now_ms, r.t_ms, cfg.battery_fresh_ms)
    {
        return Some(r.value / cfg.usable_capacity_wh * 100.0);
    }
    if let Some(p) = battery.percent
        && is_fresh(now_ms, p.t_ms, cfg.battery_fresh_ms)
    {
        return Some(p.value);
    }
    None
}

pub fn fresh_remaining_wh(now_ms: u32, battery: &BatteryInputs, cfg: &Config) -> Option<f32> {
    if let Some(r) = battery.remaining_wh
        && is_fresh(now_ms, r.t_ms, cfg.battery_fresh_ms)
    {
        return Some(r.value);
    }
    if let Some(p) = battery.percent
        && is_fresh(now_ms, p.t_ms, cfg.battery_fresh_ms)
    {
        return Some(p.value / 100.0 * cfg.usable_capacity_wh);
    }
    None
}

pub fn fresh_voltage(now_ms: u32, battery: &BatteryInputs, cfg: &Config) -> Option<f32> {
    battery
        .voltage_v
        .filter(|v| is_fresh(now_ms, v.t_ms, cfg.battery_fresh_ms))
        .map(|v| v.value)
}

pub fn is_confirmed_disarm(now_ms: u32, armed: Option<Stamped<bool>>, cfg: &Config) -> bool {
    matches!(armed, Some(a) if is_fresh(now_ms, a.t_ms, cfg.status_fresh_ms) && !a.value)
}

pub fn fresh_landed(
    now_ms: u32,
    landed: Option<Stamped<LandedState>>,
    cfg: &Config,
) -> Option<LandedState> {
    landed
        .filter(|l| is_fresh(now_ms, l.t_ms, cfg.status_fresh_ms))
        .map(|l| l.value)
}

pub fn fresh_gps(now_ms: u32, gps: Option<GpsSample>, cfg: &Config) -> Option<GpsSample> {
    gps.filter(|&g| is_fresh(now_ms, g.t_ms, cfg.gps_fresh_ms))
}
