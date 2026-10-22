use failsafe_core::Decision;
use serde::Serialize;

#[derive(Serialize)]
pub struct TraceRow {
    pub t_ms: u32,
    pub n: f32,
    pub e: f32,
    pub alt_m: f32,
    pub action: String,
    pub reason: String,
    pub margin_wh: Option<f32>,
    pub disarm_eligible: bool,
}

pub fn decision_to_row(t_ms: u32, pos: failsafe_core::Vec3, d: &Decision) -> TraceRow {
    TraceRow {
        t_ms,
        n: pos.n,
        e: pos.e,
        alt_m: pos.alt_m(),
        action: format!("{:?}", d.action),
        reason: format!("{:?}", d.reason),
        margin_wh: d.energy.margin_wh,
        disarm_eligible: d.disarm_eligible,
    }
}
