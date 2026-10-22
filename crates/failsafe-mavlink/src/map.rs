use crate::cache::{SampleCache, map_fix};
use failsafe_core::{GpsSample, LandedState, PositionSample, Stamped, Vec3};
use mavlink::dialects::common::MavMessage;
use mavlink_core::{MavlinkReader, MavlinkVersion};

pub struct AdapterConfig {
    pub vehicle_sysid: u8,
    pub vehicle_compid: u8,
    pub gcs_sysid: u8,
    pub home_lat_deg: f64,
    pub home_lon_deg: f64,
    pub usable_capacity_wh: f32,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            vehicle_sysid: 1,
            vehicle_compid: 1,
            gcs_sysid: 255,
            home_lat_deg: 47.0,
            home_lon_deg: -122.0,
            usable_capacity_wh: 40.0,
        }
    }
}

pub fn map_message(frame: &[u8], cfg: &AdapterConfig, cache: &mut SampleCache, host_ms: u32) {
    let mut reader = MavlinkReader::new(frame);
    if let Ok((header, msg)) = reader.read_message::<MavMessage>(MavlinkVersion::V2) {
        if header.system_id == cfg.gcs_sysid && header.component_id == 0 {
            if matches!(msg, MavMessage::HEARTBEAT(_)) {
                cache.gcs_heartbeat_ms = Some(host_ms);
            }
            return;
        }
        if header.system_id != cfg.vehicle_sysid || header.component_id != cfg.vehicle_compid {
            return;
        }
        match msg {
            MavMessage::HEARTBEAT(hb) => {
                let armed = hb.base_mode.bits() & 0x80_u8 != 0;
                cache.armed = Some(Stamped {
                    t_ms: host_ms,
                    value: armed,
                });
            }
            MavMessage::SYS_STATUS(st) => {
                let rc_bit = 0x10000u32;
                let ahrs_bit = 0x200000u32;
                let rc_ok = st.onboard_control_sensors_present.bits() & rc_bit != 0
                    && st.onboard_control_sensors_enabled.bits() & rc_bit != 0
                    && st.onboard_control_sensors_health.bits() & rc_bit != 0;
                let ekf_ok = st.onboard_control_sensors_present.bits() & ahrs_bit != 0
                    && st.onboard_control_sensors_enabled.bits() & ahrs_bit != 0
                    && st.onboard_control_sensors_health.bits() & ahrs_bit != 0;
                cache.rc_ok = Some(Stamped {
                    t_ms: host_ms,
                    value: rc_ok,
                });
                cache.ekf_ok = Some(Stamped {
                    t_ms: host_ms,
                    value: ekf_ok,
                });
                if st.voltage_battery != u16::MAX {
                    cache.battery.voltage_v = Some(Stamped {
                        t_ms: host_ms,
                        value: st.voltage_battery as f32 / 1000.0,
                    });
                }
                if st.battery_remaining >= 0 {
                    cache.battery.percent = Some(Stamped {
                        t_ms: host_ms,
                        value: st.battery_remaining as f32,
                    });
                }
            }
            MavMessage::BATTERY_STATUS(bs) => {
                if bs.energy_consumed >= 0 {
                    let consumed_wh = bs.energy_consumed as f32 / 36.0;
                    cache.battery.remaining_wh = Some(Stamped {
                        t_ms: host_ms,
                        value: cfg.usable_capacity_wh - consumed_wh,
                    });
                }
            }
            MavMessage::GLOBAL_POSITION_INT(gp) => {
                let (n, e) = equirectangular_m(
                    cfg.home_lat_deg,
                    cfg.home_lon_deg,
                    gp.lat as f64 / 1e7,
                    gp.lon as f64 / 1e7,
                );
                let d = -(gp.relative_alt as f32) / 1000.0;
                cache.position = Some(PositionSample {
                    t_ms: host_ms,
                    pos: Vec3 { n, e, d },
                    vel: Vec3 {
                        n: gp.vx as f32 / 100.0,
                        e: gp.vy as f32 / 100.0,
                        d: gp.vz as f32 / 100.0,
                    },
                });
            }
            MavMessage::GPS_RAW_INT(gps) => {
                cache.gps = Some(GpsSample {
                    t_ms: host_ms,
                    fix: map_fix(gps.fix_type as u8),
                    sats: if gps.satellites_visible == 255 {
                        None
                    } else {
                        Some(gps.satellites_visible)
                    },
                    hdop: if gps.eph == u16::MAX {
                        None
                    } else {
                        Some(gps.eph as f32 / 100.0)
                    },
                });
            }
            MavMessage::EXTENDED_SYS_STATE(es) => {
                let landed = match es.landed_state {
                    mavlink::dialects::common::MavLandedState::MAV_LANDED_STATE_ON_GROUND => {
                        LandedState::OnGround
                    }
                    mavlink::dialects::common::MavLandedState::MAV_LANDED_STATE_IN_AIR => {
                        LandedState::InAir
                    }
                    mavlink::dialects::common::MavLandedState::MAV_LANDED_STATE_TAKEOFF => {
                        LandedState::Takeoff
                    }
                    mavlink::dialects::common::MavLandedState::MAV_LANDED_STATE_LANDING => {
                        LandedState::Landing
                    }
                    _ => LandedState::Undefined,
                };
                cache.landed = Some(Stamped {
                    t_ms: host_ms,
                    value: landed,
                });
            }
            _ => {}
        }
    }
}

fn equirectangular_m(home_lat: f64, home_lon: f64, lat: f64, lon: f64) -> (f32, f32) {
    const R: f64 = 6_371_000.0;
    let d_lat = (lat - home_lat).to_radians();
    let d_lon = (lon - home_lon).to_radians();
    let n = d_lat * R;
    let e = d_lon * R * home_lat.to_radians().cos();
    (n as f32, e as f32)
}
