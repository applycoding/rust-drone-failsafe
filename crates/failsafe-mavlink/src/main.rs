#![forbid(unsafe_code)]

use clap::Parser;
use failsafe_mavlink::{AdapterConfig, SampleCache, map_message, read_rec_file};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    fixture: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let records = read_rec_file(&args.fixture)?;
    let cfg = AdapterConfig::default();
    let mut cache = SampleCache::new(0);
    for rec in &records {
        map_message(&rec.frame, &cfg, &mut cache, rec.host_ms);
    }
    println!(
        "gcs_hb={:?} armed={:?}",
        cache.gcs_heartbeat_ms, cache.armed
    );
    Ok(())
}
