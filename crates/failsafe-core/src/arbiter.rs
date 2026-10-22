use crate::config::Config;
use crate::types::{Action, ActiveSet, Reason};

#[derive(Clone, Copy, Debug)]
pub struct ArbiterState {
    pub prev_output: Action,
    pub prev_reason: Reason,
    pub recovery_since_ms: Option<u32>,
}

impl Default for ArbiterState {
    fn default() -> Self {
        Self {
            prev_output: Action::Nominal,
            prev_reason: Reason::None,
            recovery_since_ms: None,
        }
    }
}

pub struct ArbiterOutput {
    pub action: Action,
    pub reason: Reason,
}

#[allow(clippy::too_many_arguments)]
pub fn apply_persistence(
    state: &mut ArbiterState,
    now_ms: u32,
    cfg: &Config,
    requested: Action,
    reason: Reason,
    latches_force_land: bool,
    clock_fault: bool,
    freeze_timers: bool,
    feasibility_reset_recovery: bool,
) -> ArbiterOutput {
    if feasibility_reset_recovery {
        state.recovery_since_ms = None;
    }

    if clock_fault || freeze_timers {
        return ArbiterOutput {
            action: state.prev_output,
            reason: state.prev_reason,
        };
    }

    let mut out_action = requested;
    let out_reason = reason;

    if latches_force_land && out_action.rank() < Action::Land.rank() {
        out_action = Action::Land;
        if out_reason.rank_priority() < Reason::BatteryCritical.display_priority() {
            // keep display reason from latches via active set — caller sets reason
        }
    }

    if requested.rank() >= state.prev_output.rank() {
        state.recovery_since_ms = None;
        state.prev_output = out_action;
        state.prev_reason = out_reason;
        return ArbiterOutput {
            action: out_action,
            reason: out_reason,
        };
    }

    if state.recovery_since_ms.is_none() {
        state.recovery_since_ms = Some(now_ms);
    }
    let since = state.recovery_since_ms.unwrap_or(now_ms);
    let recovered = crate::time::age_ms(now_ms, since)
        .map(|a| a >= cfg.recovery_hold_ms)
        .unwrap_or(false);

    if recovered {
        state.prev_output = requested;
        state.prev_reason = reason;
        state.recovery_since_ms = None;
        ArbiterOutput {
            action: requested,
            reason,
        }
    } else {
        ArbiterOutput {
            action: state.prev_output,
            reason: state.prev_reason,
        }
    }
}

impl Reason {
    fn rank_priority(self) -> u8 {
        self.display_priority()
    }
}

pub fn apply_feasibility(action: Action, reason: Reason, position_valid: bool) -> (Action, Reason) {
    if (action == Action::Hold || action == Action::ReturnToLaunch) && !position_valid {
        (Action::Land, Reason::PositionInvalid)
    } else {
        (action, reason)
    }
}

pub fn latches_active(_active: &ActiveSet, triggers: &crate::triggers::TriggerState) -> bool {
    triggers.battery_critical_latched || triggers.energy_infeasible_latched
}

pub fn output_with_latches(
    action: Action,
    reason: Reason,
    active: &ActiveSet,
    triggers: &crate::triggers::TriggerState,
) -> (Action, Reason) {
    if triggers.battery_critical_latched {
        return (Action::Land.max_rank(action), Reason::BatteryCritical);
    }
    if triggers.energy_infeasible_latched {
        return (Action::Land.max_rank(action), Reason::EnergyRtlInfeasible);
    }
    let _ = active;
    (action, reason)
}

trait ActionExt {
    fn max_rank(self, other: Action) -> Action;
}

impl ActionExt for Action {
    fn max_rank(self, other: Action) -> Action {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }
}
