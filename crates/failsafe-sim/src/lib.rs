#![forbid(unsafe_code)]

mod model;
mod scenario;
mod trace;

pub use model::DroneModel;
pub use scenario::{ExpectEntry, ScenarioFile, build_config, build_geofence, load_scenario};
pub use trace::{TraceRow, decision_to_row};

use failsafe_core::{
    Action, BatteryInputs, Decision, GpsFix, GpsSample, LandedState, PositionSample, Stamped,
    Supervisor, Telemetry,
};

pub const DT_MS: u32 = 50;

pub struct SimRunner {
    pub supervisor: Supervisor,
    pub model: DroneModel,
    pub scenario: ScenarioFile,
    pub t_ms: u32,
    event_idx: usize,
}

impl SimRunner {
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        let scenario = load_scenario(toml_str).map_err(|e| e.to_string())?;
        let fence = build_geofence(&scenario.fence).map_err(|e| format!("{e:?}"))?;
        let cfg = build_config(&scenario.config);
        let sup = Supervisor::new(cfg, fence).map_err(|e| format!("{e:?}"))?;
        let cap = sup.config().usable_capacity_wh;
        Ok(Self {
            supervisor: sup,
            model: DroneModel::new(cap * 0.85),
            scenario,
            t_ms: 0,
            event_idx: 0,
        })
    }

    pub fn set_controls(
        &mut self,
        gcs_cut: bool,
        rc_cut: bool,
        gps_kill: bool,
        ekf_fault: bool,
        drain: f32,
    ) {
        self.model.gcs_cut = gcs_cut;
        self.model.rc_cut = rc_cut;
        self.model.gps_kill = gps_kill;
        self.model.ekf_fault = ekf_fault;
        self.model.set_drain(drain);
    }

    pub fn tick(&mut self) -> (TraceRow, Decision) {
        while self.event_idx < self.scenario.events.len()
            && self.scenario.events[self.event_idx].at_ms <= self.t_ms
        {
            let ev = &self.scenario.events[self.event_idx];
            self.model.apply_event(&ev.kind);
            if ev.kind == "battery_drain"
                && let Some(v) = ev.value
            {
                self.model.set_drain(v);
            }
            self.event_idx += 1;
        }

        let telemetry = self.build_telemetry();
        let decision = self.supervisor.step(&telemetry);
        let dt_s = DT_MS as f32 / 1000.0;
        self.model.step_physics(dt_s, &decision);
        let row = decision_to_row(self.t_ms, self.model.pos, &decision);
        self.t_ms += DT_MS;
        (row, decision)
    }

    pub fn run_to_end(&mut self) -> Vec<TraceRow> {
        let end = self.scenario.meta.duration_ms;
        let mut rows = Vec::new();
        while self.t_ms <= end {
            let (row, _) = self.tick();
            rows.push(row);
        }
        rows
    }

    pub fn snapshot_telemetry(&self) -> Telemetry {
        self.build_telemetry()
    }

    fn build_telemetry(&self) -> Telemetry {
        let now = self.t_ms;
        let pos = self.model.pos;
        let in_air = pos.alt_m() > 1.0;
        Telemetry {
            now_ms: now,
            position: Some(PositionSample {
                t_ms: now,
                pos,
                vel: self.model.vel,
            }),
            gps: if self.model.gps_kill {
                Some(GpsSample {
                    t_ms: now,
                    fix: GpsFix::None,
                    sats: None,
                    hdop: None,
                })
            } else {
                Some(GpsSample {
                    t_ms: now,
                    fix: GpsFix::Fix3D,
                    sats: Some(12),
                    hdop: Some(1.0),
                })
            },
            battery: BatteryInputs {
                voltage_v: Some(Stamped {
                    t_ms: now,
                    value: 14.8,
                }),
                percent: Some(Stamped {
                    t_ms: now,
                    value: (self.model.battery_wh / self.supervisor.config().usable_capacity_wh)
                        * 100.0,
                }),
                remaining_wh: Some(Stamped {
                    t_ms: now,
                    value: self.model.battery_wh,
                }),
            },
            gcs_heartbeat_ms: if self.model.gcs_cut { None } else { Some(now) },
            armed: Some(Stamped {
                t_ms: now,
                value: true,
            }),
            landed: Some(Stamped {
                t_ms: now,
                value: if in_air {
                    LandedState::InAir
                } else {
                    LandedState::OnGround
                },
            }),
            rc_ok: Some(Stamped {
                t_ms: now,
                value: !self.model.rc_cut,
            }),
            ekf_ok: Some(Stamped {
                t_ms: now,
                value: !self.model.ekf_fault,
            }),
        }
    }
}

pub fn parse_action(s: &str) -> Option<Action> {
    match s {
        "Nominal" => Some(Action::Nominal),
        "Warn" => Some(Action::Warn),
        "Hold" => Some(Action::Hold),
        "ReturnToLaunch" => Some(Action::ReturnToLaunch),
        "Land" => Some(Action::Land),
        _ => None,
    }
}
