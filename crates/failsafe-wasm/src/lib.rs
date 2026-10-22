use failsafe_sim::SimRunner;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmSim {
    inner: SimRunner,
}

#[wasm_bindgen]
impl WasmSim {
    #[wasm_bindgen(constructor)]
    pub fn new(scenario_toml: &str) -> Result<WasmSim, JsError> {
        let inner = SimRunner::from_toml(scenario_toml).map_err(|e| JsError::new(&e))?;
        Ok(WasmSim { inner })
    }

    pub fn set_controls(&mut self, json: &str) -> Result<(), JsError> {
        let v: serde_json::Value =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let gcs = v.get("gcs_cut").and_then(|x| x.as_bool()).unwrap_or(false);
        let rc = v.get("rc_cut").and_then(|x| x.as_bool()).unwrap_or(false);
        let gps = v.get("gps_kill").and_then(|x| x.as_bool()).unwrap_or(false);
        let ekf = v
            .get("ekf_fault")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        let drain = v.get("drain").and_then(|x| x.as_f64()).unwrap_or(0.01) as f32;
        self.inner.set_controls(gcs, rc, gps, ekf, drain);
        Ok(())
    }

    pub fn tick(&mut self) -> Result<String, JsError> {
        let (row, decision) = self.inner.tick();
        let frame = FrameJson {
            t_ms: row.t_ms,
            n: row.n,
            e: row.e,
            alt_m: row.alt_m,
            action: row.action,
            reason: row.reason,
            margin_wh: row.margin_wh,
            disarm_eligible: row.disarm_eligible,
            active: format!("{:?}", decision.active),
        };
        serde_json::to_string(&frame).map_err(|e| JsError::new(&e.to_string()))
    }
}

#[derive(Serialize)]
struct FrameJson {
    t_ms: u32,
    n: f32,
    e: f32,
    alt_m: f32,
    action: String,
    reason: String,
    margin_wh: Option<f32>,
    disarm_eligible: bool,
    active: String,
}
