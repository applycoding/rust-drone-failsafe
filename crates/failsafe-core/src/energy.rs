use crate::config::Config;
use crate::types::{EnergyEstimate, Vec3};
use libm::fmaxf;

pub fn rtl_energy(cfg: &Config, pos: Vec3, remaining_wh: Option<f32>) -> EnergyEstimate {
    let d = pos.horizontal_dist_home();
    let h = pos.alt_m();
    let h_rtl = cfg.h_rtl;

    let climb_m = fmaxf(0.0, h_rtl - h);
    let e_climb_j = (climb_m / cfg.v_climb) * cfg.p_climb;

    let e_cruise_j = (d / cfg.v_cruise) * cfg.p_cruise * cfg.k_wind;

    let descent_m = fmaxf(h, h_rtl);
    let e_desc_j = (descent_m / cfg.v_desc) * cfg.p_desc;

    let rtl_wh = (e_climb_j + e_cruise_j + e_desc_j) / 3600.0;
    let land_wh = (h / cfg.v_desc) * cfg.p_desc / 3600.0;

    let margin_wh = remaining_wh.map(|r| r - rtl_wh - cfg.reserve_wh);
    let insufficient_for_landing = remaining_wh.map(|r| r < land_wh).unwrap_or(false);

    EnergyEstimate {
        rtl_wh,
        land_wh,
        reserve_wh: cfg.reserve_wh,
        remaining_wh,
        margin_wh,
        insufficient_for_landing,
    }
}
