#![forbid(unsafe_code)]

use mavlink::dialects::common::MavMessage;
use mavlink_core::{MavlinkReader, MavlinkVersion};

const HEARTBEAT_V2: &[u8] = &[
    mavlink::MAV_STX_V2,
    0x09,
    0,
    0,
    239,
    1,
    2,
    0x00,
    0x00,
    0x00,
    0x05,
    0x00,
    0x00,
    0x00,
    0x02,
    0x03,
    0x59,
    0x03,
    0x03,
    46,
    115,
];

#[test]
fn heartbeat_round_trip_v2() {
    let mut reader = MavlinkReader::new(HEARTBEAT_V2);
    let (_header, msg) = reader
        .read_message::<MavMessage>(MavlinkVersion::V2)
        .expect("decode heartbeat");
    assert!(matches!(msg, MavMessage::HEARTBEAT(_)));
}
