use failsafe_core::{
    BatteryInputs, GpsFix, GpsSample, LandedState, PositionSample, Stamped, Telemetry,
};

#[derive(Clone, Debug)]
pub struct SampleCache {
    pub now_ms: u32,
    pub position: Option<PositionSample>,
    pub gps: Option<GpsSample>,
    pub battery: BatteryInputs,
    pub gcs_heartbeat_ms: Option<u32>,
    pub armed: Option<Stamped<bool>>,
    pub landed: Option<Stamped<LandedState>>,
    pub rc_ok: Option<Stamped<bool>>,
    pub ekf_ok: Option<Stamped<bool>>,
}

impl SampleCache {
    pub fn new(now_ms: u32) -> Self {
        Self {
            now_ms,
            position: None,
            gps: None,
            battery: BatteryInputs {
                voltage_v: None,
                percent: None,
                remaining_wh: None,
            },
            gcs_heartbeat_ms: None,
            armed: None,
            landed: None,
            rc_ok: None,
            ekf_ok: None,
        }
    }

    pub fn snapshot(&self, now_ms: u32) -> Telemetry {
        Telemetry {
            now_ms,
            position: self.position,
            gps: self.gps,
            battery: self.battery,
            gcs_heartbeat_ms: self.gcs_heartbeat_ms,
            armed: self.armed,
            landed: self.landed,
            rc_ok: self.rc_ok,
            ekf_ok: self.ekf_ok,
        }
    }
}

pub fn map_fix(fix_type: u8) -> GpsFix {
    match fix_type {
        0 | 1 => GpsFix::None,
        2 => GpsFix::Fix2D,
        _ => GpsFix::Fix3D,
    }
}
