#![forbid(unsafe_code)]

mod cache;
mod map;
mod replay;

pub use cache::SampleCache;
pub use map::{AdapterConfig, map_message};
pub use replay::{ReplayRecord, read_hex_fixture, read_rec_file};

use failsafe_core::Telemetry;

pub fn telemetry_from_replay(
    records: &[ReplayRecord],
    cfg: &AdapterConfig,
    now_ms: u32,
) -> Telemetry {
    let mut cache = SampleCache::new(now_ms);
    for rec in records {
        if rec.host_ms <= now_ms {
            map_message(&rec.frame, cfg, &mut cache, rec.host_ms);
        }
    }
    cache.snapshot(now_ms)
}
