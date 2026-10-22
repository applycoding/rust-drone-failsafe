use crate::config::Config;
use crate::time::is_fresh;
use crate::types::{Action, LandedState, Telemetry};
use crate::validate::fresh_landed;

#[derive(Clone, Copy, Debug, Default)]
pub struct DisarmGate {
    hold_since_ms: Option<u32>,
}

impl DisarmGate {
    pub fn reset(&mut self) {
        self.hold_since_ms = None;
    }

    pub fn evaluate(
        &mut self,
        now_ms: u32,
        t: &Telemetry,
        cfg: &Config,
        requested: Action,
        clock_fault: bool,
    ) -> bool {
        if clock_fault {
            self.reset();
            return false;
        }

        if requested.rank() < Action::Land.rank() {
            self.reset();
            return false;
        }

        let armed_ok = match t.armed {
            Some(a) if is_fresh(now_ms, a.t_ms, cfg.status_fresh_ms) && a.value => true,
            _ => {
                self.reset();
                return false;
            }
        };

        let on_ground = fresh_landed(now_ms, t.landed, cfg) == Some(LandedState::OnGround);
        if !armed_ok || !on_ground {
            // Clears immediately when airborne or landing evidence is lost/stale.
            self.reset();
            return false;
        }

        if self.hold_since_ms.is_none() {
            self.hold_since_ms = Some(now_ms);
        }
        let since = self.hold_since_ms.unwrap_or(now_ms);
        matches!(
            crate::time::age_ms(now_ms, since),
            Some(age) if age >= cfg.disarm_hold_ms
        )
    }
}
