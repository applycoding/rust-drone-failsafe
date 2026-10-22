#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

mod arbiter;
mod config;
mod disarm;
mod energy;
mod geometry;
mod time;
mod triggers;
mod types;
mod validate;

pub use arbiter::{ArbiterState, apply_feasibility, apply_persistence, output_with_latches};
pub use config::{Config, ConfigError, default_fence, validate_config};
pub use disarm::DisarmGate;
pub use energy::rtl_energy;
pub use geometry::{ConvexPolygon, FenceError, Geofence};
pub use time::age_ms;
pub use triggers::{TriggerState, compute_position_valid};
pub use types::*;
pub use validate::{GlitchState, validate_telemetry};

use crate::arbiter::latches_active;
use crate::energy::rtl_energy as compute_energy;
use crate::validate::{apply_glitch_filter, is_confirmed_disarm, validate_telemetry as validate};

#[derive(Clone, Debug)]
pub struct Supervisor {
    cfg: Config,
    fence: Geofence,
    triggers: TriggerState,
    arbiter: ArbiterState,
    disarm: DisarmGate,
    glitch: GlitchState,
    last_now: Option<u32>,
    epoch_start: u32,
    backwards_streak: u8,
    initialized: bool,
}

impl Supervisor {
    pub fn new(cfg: Config, fence: Geofence) -> Result<Self, ConfigError> {
        validate_config(&cfg, &fence)?;
        Ok(Self {
            cfg,
            fence,
            triggers: TriggerState::default(),
            arbiter: ArbiterState::default(),
            disarm: DisarmGate::default(),
            glitch: GlitchState::default(),
            last_now: None,
            epoch_start: 0,
            backwards_streak: 0,
            initialized: false,
        })
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn fence(&self) -> &Geofence {
        &self.fence
    }

    pub fn step(&mut self, raw: &Telemetry) -> Decision {
        if !self.initialized {
            self.last_now = Some(raw.now_ms);
            self.epoch_start = raw.now_ms;
            self.initialized = true;
        }

        let mut clock_fault = false;
        let mut freeze_timers = false;
        if let Some(last) = self.last_now {
            let delta = raw.now_ms.wrapping_sub(last);
            if delta >= types::HALF_RANGE_MS {
                clock_fault = true;
                freeze_timers = true;
                self.backwards_streak += 1;
                self.disarm.reset();
                if self.backwards_streak >= 3 {
                    self.epoch_start = raw.now_ms;
                    self.backwards_streak = 0;
                    self.triggers.gcs_clear_timer_clear();
                    self.arbiter.recovery_since_ms = None;
                    self.glitch = GlitchState::default();
                    self.disarm.reset();
                }
            } else {
                self.backwards_streak = 0;
                self.last_now = Some(raw.now_ms);
            }
        } else {
            self.last_now = Some(raw.now_ms);
        }

        let validated = validate(raw, &self.cfg, &self.fence);
        let mut telemetry = validated.telemetry;
        let invalid_input = validated.invalid_input;

        if is_confirmed_disarm(telemetry.now_ms, telemetry.armed, &self.cfg) {
            self.triggers.reset_all();
            self.arbiter = ArbiterState::default();
            self.disarm.reset();
            return Decision {
                action: Action::Nominal,
                reason: Reason::None,
                requested: Action::Nominal,
                active: ActiveSet::new(),
                position_valid: false,
                clock_fault,
                energy: compute_energy(
                    &self.cfg,
                    Vec3 {
                        n: 0.0,
                        e: 0.0,
                        d: 0.0,
                    },
                    None,
                ),
                fence_dist_m: None,
                disarm_eligible: false,
            };
        }

        let filtered = apply_glitch_filter(
            &mut self.glitch,
            telemetry.now_ms,
            telemetry.position,
            &self.cfg,
        );
        telemetry.position = filtered;

        let pos_valid = compute_position_valid(
            telemetry.now_ms,
            &telemetry,
            &self.cfg,
            &self.fence,
            filtered,
        );

        let pos_vec = filtered.map(|p| p.pos);

        let active = self.triggers.update(
            telemetry.now_ms,
            &telemetry,
            &self.cfg,
            &self.fence,
            pos_valid,
            pos_vec,
            invalid_input,
            clock_fault,
            freeze_timers,
        );

        let (requested, reason) = active.requested_action();

        let latch_land = latches_active(&active, &self.triggers);
        let (req_action, req_reason) =
            output_with_latches(requested, reason, &active, &self.triggers);

        let persist = apply_persistence(
            &mut self.arbiter,
            telemetry.now_ms,
            &self.cfg,
            req_action,
            req_reason,
            latch_land,
            clock_fault,
            freeze_timers,
            false,
        );

        let (mut action, mut out_reason) = (persist.action, persist.reason);
        let (fa, fr) = apply_feasibility(action, out_reason, pos_valid);
        let feasibility_substitution = fa != action;
        action = fa;
        out_reason = fr;
        if feasibility_substitution {
            self.arbiter.prev_output = action;
            self.arbiter.prev_reason = out_reason;
            self.arbiter.recovery_since_ms = None;
        } else {
            self.arbiter.prev_output = action;
            self.arbiter.prev_reason = out_reason;
        }

        let remaining =
            crate::validate::fresh_remaining_wh(telemetry.now_ms, &telemetry.battery, &self.cfg);
        let energy = pos_vec
            .map(|p| compute_energy(&self.cfg, p, remaining))
            .unwrap_or_else(|| {
                compute_energy(
                    &self.cfg,
                    Vec3 {
                        n: 0.0,
                        e: 0.0,
                        d: 0.0,
                    },
                    remaining,
                )
            });

        let fence_dist_m = pos_vec.map(|p| {
            let pt = Point2 { n: p.n, e: p.e };
            self.fence.poly.signed_distance(pt)
        });

        let disarm_eligible = self.disarm.evaluate(
            telemetry.now_ms,
            &telemetry,
            &self.cfg,
            requested,
            clock_fault,
        );

        Decision {
            action,
            reason: out_reason,
            requested,
            active,
            position_valid: pos_valid,
            clock_fault,
            energy,
            fence_dist_m,
            disarm_eligible,
        }
    }
}
