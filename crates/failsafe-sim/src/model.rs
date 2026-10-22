use failsafe_core::{Action, Decision, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct DroneModel {
    pub pos: Vec3,
    pub vel: Vec3,
    pub battery_wh: f32,
    pub gcs_cut: bool,
    pub rc_cut: bool,
    pub gps_kill: bool,
    pub ekf_fault: bool,
    pub drain_rate_wh_per_s: f32,
}

impl DroneModel {
    pub fn new(battery_wh: f32) -> Self {
        Self {
            pos: Vec3 {
                n: 0.0,
                e: 0.0,
                d: -30.0,
            },
            vel: Vec3 {
                n: 2.0,
                e: 0.0,
                d: 0.0,
            },
            battery_wh,
            gcs_cut: false,
            rc_cut: false,
            gps_kill: false,
            ekf_fault: false,
            drain_rate_wh_per_s: 0.01,
        }
    }

    pub fn apply_event(&mut self, kind: &str) {
        match kind {
            "cut_gcs" => self.gcs_cut = true,
            "cut_rc" => self.rc_cut = true,
            "kill_gps" => self.gps_kill = true,
            "ekf_fault" => self.ekf_fault = true,
            "reset" => {
                self.gcs_cut = false;
                self.rc_cut = false;
                self.gps_kill = false;
                self.ekf_fault = false;
            }
            _ => {}
        }
    }

    pub fn set_drain(&mut self, rate: f32) {
        self.drain_rate_wh_per_s = rate;
    }

    pub fn step_physics(&mut self, dt_s: f32, decision: &Decision) {
        let mut accel_n = 0.0f32;
        match decision.action {
            Action::ReturnToLaunch => {
                let d = self.pos.horizontal_dist_home();
                if d > 1.0 {
                    accel_n = -self.pos.n / d * 2.0;
                }
            }
            Action::Land if self.pos.alt_m() > 0.5 => {
                self.vel.d = 0.5;
            }
            _ => {}
        }
        self.vel.n += accel_n * dt_s;
        self.pos.n += self.vel.n * dt_s;
        self.pos.e += self.vel.e * dt_s;
        self.pos.d += self.vel.d * dt_s;
        self.battery_wh = (self.battery_wh - self.drain_rate_wh_per_s * dt_s).max(0.0);
    }
}
