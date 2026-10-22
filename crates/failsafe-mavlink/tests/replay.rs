#![forbid(unsafe_code)]

use failsafe_mavlink::{AdapterConfig, SampleCache, map_message, read_hex_fixture};
use std::path::PathBuf;

#[test]
fn handchecked_heartbeat_hex_decodes() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/heartbeat_handchecked.hex");
    let frame = read_hex_fixture(&path).expect("read hex");
    let cfg = AdapterConfig::default();
    let mut cache = SampleCache::new(0);
    map_message(&frame, &cfg, &mut cache, 1000);
    assert!(cache.gcs_heartbeat_ms.is_none());
}

#[test]
fn spike_heartbeat_vehicle_armed() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/heartbeat_handchecked.hex");
    let frame = read_hex_fixture(&path).expect("read hex");
    let cfg = AdapterConfig {
        vehicle_sysid: 1,
        vehicle_compid: 2,
        gcs_sysid: 255,
        ..AdapterConfig::default()
    };
    let mut cache = SampleCache::new(0);
    map_message(&frame, &cfg, &mut cache, 1);
    assert!(cache.armed.is_some());
}
