use crate::config::Config;
use crate::energy::rtl_energy;
use crate::geometry::Geofence;
use crate::time::{age_ms, is_fresh};
use crate::types::{Action, ActiveSet, GpsFix, LandedState, Reason, Telemetry, Vec3};
use crate::validate::{
    fresh_battery_percent, fresh_gps, fresh_landed, fresh_remaining_wh, fresh_voltage,
    position_valid,
};

#[derive(Clone, Copy, Debug, Default)]
struct Timer {
    since_ms: Option<u32>,
}

impl Timer {
    fn pending(&self) -> bool {
        self.since_ms.is_some()
    }
}

impl Timer {
    fn clear(&mut self) {
        self.since_ms = None;
    }

    fn held_set(&mut self, now: u32, active: bool, hold_ms: u32) -> bool {
        Self::update_set(now, active, self, hold_ms)
    }

    fn update_set(now: u32, active: bool, t: &mut Timer, hold_ms: u32) -> bool {
        if active {
            if t.since_ms.is_none() {
                t.since_ms = Some(now);
            }
            let since = t.since_ms.unwrap_or(now);
            age_ms(now, since).map(|a| a >= hold_ms).unwrap_or(false)
        } else {
            t.clear();
            false
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TriggerState {
    pub battery_critical_latched: bool,
    pub energy_infeasible_latched: bool,
    gcs_clear_timer: Timer,
    rc_clear_timer: Timer,
    gps_deg_set: Timer,
    gps_deg_clear: Timer,
    gps_lost_set: Timer,
    gps_lost_clear: Timer,
    ekf_set: Timer,
    ekf_clear: Timer,
    fence_breach_set: Timer,
    fence_breach_clear: Timer,
    alt_max_set: Timer,
    alt_max_clear: Timer,
    alt_min_set: Timer,
    alt_min_clear: Timer,
    stale_bat_clear: Timer,
    bat_warn_set: Timer,
    bat_warn_clear: Timer,
    bat_low_set: Timer,
    bat_low_clear: Timer,
    bat_crit_set: Timer,
    energy_marg_set: Timer,
    energy_marg_clear: Timer,
    energy_infeas_set: Timer,
    invalid_clear: Timer,
    invalid_active: bool,
    rc_lost_set: Timer,
    gcs_stage: u8,
}

impl TriggerState {
    pub fn gcs_clear_timer_clear(&mut self) {
        self.gcs_clear_timer.clear();
    }
}

impl TriggerState {
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    pub fn clear_latches(&mut self) {
        self.battery_critical_latched = false;
        self.energy_infeasible_latched = false;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        now_ms: u32,
        t: &Telemetry,
        cfg: &Config,
        fence: &Geofence,
        pos_valid: bool,
        pos: Option<Vec3>,
        invalid_input: bool,
        clock_fault: bool,
        freeze_timers: bool,
    ) -> ActiveSet {
        let mut active = ActiveSet::new();

        if clock_fault {
            active.set(Reason::ClockFault, Action::Warn);
            return active;
        }

        if invalid_input {
            self.invalid_active = true;
            active.set(Reason::InvalidInput, Action::Warn);
            if !freeze_timers {
                self.invalid_clear.clear();
            }
        } else if !freeze_timers {
            let cleared = self
                .invalid_clear
                .held_set(now_ms, true, cfg.invalid_input_clear_ms);
            if cleared {
                self.invalid_active = false;
            }
            if self.invalid_active {
                active.set(Reason::InvalidInput, Action::Warn);
            }
        } else if self.invalid_active {
            active.set(Reason::InvalidInput, Action::Warn);
        }

        // GCS link
        let gcs_age = t
            .gcs_heartbeat_ms
            .map(|hb| age_ms(now_ms, hb).unwrap_or(u32::MAX))
            .unwrap_or(u32::MAX);
        if gcs_age > cfg.gcs_rtl_ms {
            self.gcs_stage = 3;
            active.set(Reason::GcsLinkLost, Action::ReturnToLaunch);
        } else if gcs_age > cfg.gcs_hold_ms {
            self.gcs_stage = 2;
            active.set(Reason::GcsLinkLost, Action::Hold);
        } else if gcs_age > cfg.gcs_warn_ms {
            self.gcs_stage = 1;
            active.set(Reason::GcsLinkLost, Action::Warn);
        } else if gcs_age <= cfg.gcs_clear_ms {
            if !freeze_timers {
                let clear = self
                    .gcs_clear_timer
                    .held_set(now_ms, true, cfg.gcs_clear_hold_ms);
                if !clear {
                    if self.gcs_stage == 3 {
                        active.set(Reason::GcsLinkLost, Action::ReturnToLaunch);
                    } else if self.gcs_stage == 2 {
                        active.set(Reason::GcsLinkLost, Action::Hold);
                    } else if self.gcs_stage == 1 {
                        active.set(Reason::GcsLinkLost, Action::Warn);
                    }
                } else {
                    self.gcs_stage = 0;
                }
            }
        } else if !freeze_timers {
            self.gcs_clear_timer.clear();
        }

        // RC
        let rc_bad = cfg.rc_required
            && match t.rc_ok {
                Some(r) if is_fresh(now_ms, r.t_ms, cfg.status_fresh_ms) => !r.value,
                _ => true,
            };
        let rc_lost = if !freeze_timers {
            self.rc_lost_set
                .held_set(now_ms, rc_bad, cfg.rc_lost_set_ms)
        } else {
            rc_bad
        };
        if rc_lost {
            active.set(Reason::RcLost, Action::ReturnToLaunch);
        } else if !freeze_timers {
            let rc_good = matches!(t.rc_ok, Some(r) if is_fresh(now_ms, r.t_ms, cfg.status_fresh_ms) && r.value);
            let cleared = self
                .rc_clear_timer
                .held_set(now_ms, rc_good, cfg.rc_lost_clear_ms);
            if !cleared && self.rc_lost_set.pending() {
                active.set(Reason::RcLost, Action::ReturnToLaunch);
            }
        }

        // GPS degraded / lost
        let gps = fresh_gps(now_ms, t.gps, cfg);
        let degraded = match gps {
            Some(g) => {
                let bad_sats = g.sats.map(|s| s < 8).unwrap_or(true);
                let bad_hdop = g.hdop.map(|h| h > 2.5).unwrap_or(true);
                bad_sats || bad_hdop
            }
            None => true,
        };
        let deg_set = !freeze_timers
            && self
                .gps_deg_set
                .held_set(now_ms, degraded, cfg.gps_degraded_set_ms);
        let deg_ok = match gps {
            Some(g) => {
                let ok_sats = g.sats.map(|s| s >= 10).unwrap_or(false);
                let ok_hdop = g.hdop.map(|h| h <= 2.0).unwrap_or(false);
                ok_sats && ok_hdop
            }
            None => false,
        };
        let deg_cleared = !freeze_timers
            && self
                .gps_deg_clear
                .held_set(now_ms, deg_ok, cfg.gps_degraded_clear_ms);
        if deg_set && !deg_cleared {
            active.set(Reason::GpsDegraded, Action::Warn);
        }

        let gps_lost_cond = match gps {
            Some(g) => g.fix != GpsFix::Fix3D,
            None => true,
        };
        let gl_set = !freeze_timers
            && self
                .gps_lost_set
                .held_set(now_ms, gps_lost_cond, cfg.gps_lost_set_ms);
        let gl_ok = matches!(gps, Some(g) if g.fix == GpsFix::Fix3D);
        let gl_cleared = !freeze_timers
            && self
                .gps_lost_clear
                .held_set(now_ms, gl_ok, cfg.gps_lost_clear_ms);
        if gl_set && !gl_cleared {
            active.set(Reason::GpsLost, Action::Land);
        }

        // EKF
        let ekf_bad = match t.ekf_ok {
            Some(e) if is_fresh(now_ms, e.t_ms, cfg.status_fresh_ms) => !e.value,
            _ => true,
        };
        let ekf_set = !freeze_timers
            && self
                .ekf_set
                .held_set(now_ms, ekf_bad, cfg.ekf_unhealthy_set_ms);
        let ekf_ok =
            matches!(t.ekf_ok, Some(e) if is_fresh(now_ms, e.t_ms, cfg.status_fresh_ms) && e.value);
        let ekf_cleared = !freeze_timers
            && self
                .ekf_clear
                .held_set(now_ms, ekf_ok, cfg.ekf_unhealthy_clear_ms);
        if ekf_set && !ekf_cleared {
            active.set(Reason::EkfUnhealthy, Action::Land);
        }

        // Fence
        if pos_valid && let Some(p) = pos {
            let pt = crate::types::Point2 { n: p.n, e: p.e };
            let sd = fence.poly.signed_distance(pt);
            if sd >= 0.0 && sd < cfg.fence_approach_m {
                active.set(Reason::FenceApproach, Action::Warn);
            }
            let breach = sd < 0.0;
            let br_set = !freeze_timers
                && self
                    .fence_breach_set
                    .held_set(now_ms, breach, cfg.fence_breach_set_ms);
            let br_clear = sd >= cfg.fence_breach_clear_m;
            let br_cleared = !freeze_timers
                && self
                    .fence_breach_clear
                    .held_set(now_ms, br_clear, cfg.fence_breach_clear_ms);
            if br_set && !br_cleared {
                active.set(Reason::FenceBreach, Action::ReturnToLaunch);
            }
            let alt = p.alt_m();
            if alt > fence.max_alt_m {
                let am_set =
                    !freeze_timers && self.alt_max_set.held_set(now_ms, true, cfg.alt_max_set_ms);
                let am_clear = alt < fence.max_alt_m - cfg.alt_max_clear_m;
                let am_cleared = !freeze_timers
                    && self
                        .alt_max_clear
                        .held_set(now_ms, am_clear, cfg.alt_max_clear_hold_ms);
                if am_set && !am_cleared {
                    active.set(Reason::AltitudeMax, Action::ReturnToLaunch);
                }
            } else if !freeze_timers {
                self.alt_max_set.clear();
                self.alt_max_clear.clear();
            }
            let horiz = p.horizontal_dist_home();
            let in_air = fresh_landed(now_ms, t.landed, cfg) == Some(LandedState::InAir);
            let alt_low = alt < fence.min_alt_m && horiz > cfg.alt_min_home_radius_m && in_air;
            let amn_set = !freeze_timers
                && self
                    .alt_min_set
                    .held_set(now_ms, alt_low, cfg.alt_min_set_ms);
            let amn_clear =
                alt > fence.min_alt_m + cfg.alt_min_clear_m || horiz <= cfg.alt_min_home_radius_m;
            let amn_cleared = !freeze_timers
                && self
                    .alt_min_clear
                    .held_set(now_ms, amn_clear, cfg.alt_min_clear_ms);
            if amn_set && !amn_cleared {
                active.set(Reason::AltitudeMin, Action::Warn);
            }
        }

        // Battery stale
        let has_v = fresh_voltage(now_ms, &t.battery, cfg).is_some();
        let has_p = fresh_battery_percent(now_ms, &t.battery, cfg).is_some();
        if !has_v && !has_p {
            active.set(Reason::StaleBattery, Action::ReturnToLaunch);
        } else if !freeze_timers {
            let cleared = self
                .stale_bat_clear
                .held_set(now_ms, true, cfg.stale_battery_clear_ms);
            if !cleared && self.stale_bat_clear.since_ms.is_some() {
                // was stale, clearing
            }
        }

        let pct = fresh_battery_percent(now_ms, &t.battery, cfg);
        let v = fresh_voltage(now_ms, &t.battery, cfg);
        if let Some(p) = pct {
            let warn = p < cfg.battery_warn_pct;
            let w_set = !freeze_timers
                && self
                    .bat_warn_set
                    .held_set(now_ms, warn, cfg.battery_warn_set_ms);
            let w_clr = p > cfg.battery_warn_clear_pct;
            let w_cleared = !freeze_timers
                && self
                    .bat_warn_clear
                    .held_set(now_ms, w_clr, cfg.battery_warn_clear_ms);
            if w_set && !w_cleared {
                active.set(Reason::BatteryWarn, Action::Warn);
            }
            let low = p < cfg.battery_low_pct;
            let l_set = !freeze_timers
                && self
                    .bat_low_set
                    .held_set(now_ms, low, cfg.battery_low_set_ms);
            let l_clr = p > cfg.battery_low_clear_pct;
            let l_cleared = !freeze_timers
                && self
                    .bat_low_clear
                    .held_set(now_ms, l_clr, cfg.battery_low_clear_ms);
            if l_set && !l_cleared {
                active.set(Reason::BatteryLow, Action::ReturnToLaunch);
            }
            let crit_v = v
                .map(|vv| vv < cfg.battery_critical_v_per_cell * cfg.cell_count as f32)
                .unwrap_or(false);
            let crit = p < cfg.battery_critical_pct || crit_v;
            let c_set = !freeze_timers
                && self
                    .bat_crit_set
                    .held_set(now_ms, crit, cfg.battery_critical_set_ms);
            if c_set || self.battery_critical_latched {
                self.battery_critical_latched = self.battery_critical_latched || c_set;
                active.set(Reason::BatteryCritical, Action::Land);
            }
        }

        let remaining = fresh_remaining_wh(now_ms, &t.battery, cfg);
        if let Some(pos_v) = pos {
            let energy = rtl_energy(cfg, pos_v, remaining);
            if let Some(margin) = energy.margin_wh {
                let marginal = margin >= 0.0 && margin < cfg.energy_marginal_wh;
                let m_set = !freeze_timers
                    && self
                        .energy_marg_set
                        .held_set(now_ms, marginal, cfg.energy_marginal_set_ms);
                let m_clr = margin > cfg.energy_marginal_clear_wh;
                let m_cleared = !freeze_timers
                    && self
                        .energy_marg_clear
                        .held_set(now_ms, m_clr, cfg.energy_marginal_clear_ms);
                if m_set && !m_cleared {
                    active.set(Reason::EnergyRtlMarginal, Action::ReturnToLaunch);
                }
                let infeas = margin < 0.0;
                let i_set = !freeze_timers
                    && self.energy_infeas_set.held_set(
                        now_ms,
                        infeas,
                        cfg.energy_infeasible_set_ms,
                    );
                if i_set || self.energy_infeasible_latched {
                    self.energy_infeasible_latched = self.energy_infeasible_latched || i_set;
                    active.set(Reason::EnergyRtlInfeasible, Action::Land);
                }
            }
        }

        active
    }
}

pub fn compute_position_valid(
    now_ms: u32,
    t: &Telemetry,
    cfg: &Config,
    fence: &Geofence,
    filtered_pos: Option<crate::types::PositionSample>,
) -> bool {
    position_valid(now_ms, filtered_pos, t.ekf_ok, cfg, fence)
}
