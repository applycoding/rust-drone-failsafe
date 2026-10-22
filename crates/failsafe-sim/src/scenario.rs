use failsafe_core::{Config, ConvexPolygon, Geofence, Point2};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ScenarioFile {
    pub meta: Meta,
    pub fence: FenceDto,
    pub config: Option<ConfigDto>,
    #[serde(default)]
    pub events: Vec<ScenarioEvent>,
    #[serde(default)]
    pub expect: Vec<ExpectEntry>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub name: String,
    pub duration_ms: u32,
}

#[derive(Debug, Deserialize)]
pub struct FenceDto {
    pub vertices: Vec<[f32; 2]>,
    pub min_alt_m: f32,
    pub max_alt_m: f32,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfigDto {
    pub usable_capacity_wh: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioEvent {
    pub at_ms: u32,
    pub kind: String,
    #[serde(default)]
    pub value: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpectEntry {
    pub at_ms: u32,
    pub action: String,
}

pub fn load_scenario(toml_str: &str) -> Result<ScenarioFile, toml::de::Error> {
    toml::from_str(toml_str)
}

pub fn build_geofence(dto: &FenceDto) -> Result<Geofence, failsafe_core::FenceError> {
    let pts: Vec<Point2> = dto
        .vertices
        .iter()
        .map(|v| Point2 { n: v[0], e: v[1] })
        .collect();
    let poly = ConvexPolygon::new(&pts)?;
    Geofence::new(poly, dto.min_alt_m, dto.max_alt_m)
}

pub fn build_config(dto: &Option<ConfigDto>) -> Config {
    let mut cfg = Config::default();
    if let Some(c) = dto
        && let Some(cap) = c.usable_capacity_wh
    {
        cfg.usable_capacity_wh = cap;
    }
    cfg
}
