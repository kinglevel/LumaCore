// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `receiver`.
//!
//! Owns:          the unit tests of receiver.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11 "Step 4c hardening".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::super::source::data_packet;
use super::*;

#[test]
fn parses_what_the_sender_encodes() {
    let cid = Uuid::from_u128(7);
    let mut data = [0u8; 512];
    data[0] = 1;
    data[511] = 2;
    let p = data_packet(&cid, "Guest desk", 6, 150, 9, 0, &data);
    let d = parse_data_packet(&p).unwrap();
    assert_eq!(d.cid, cid);
    assert_eq!(d.name.as_str(), "Guest desk");
    assert_eq!(d.sync, 0);
    assert_eq!((d.priority, d.sequence, d.universe), (150, 9, 6));
    assert_eq!((d.slots[0], d.slots[511], d.start_code), (1, 2, 0));
    assert!(!d.preview && !d.terminated);
    let end = data_packet(&cid, "x", 6, 0, 10, OPTION_TERMINATED, &data);
    assert!(parse_data_packet(&end).unwrap().terminated);
    assert!(parse_data_packet(&p[..100]).is_none());
    let mut discovery = p.clone();
    discovery[21] = 8;
    assert!(parse_data_packet(&discovery).is_none());
    let sync = super::super::source::sync_packet(&cid, 42, 7);
    assert_eq!(parse_sync_packet(&sync), Some((cid, 42, 7)));
    assert!(parse_sync_packet(&p).is_none());
    assert!(parse_data_packet(&sync).is_none());
}

#[test]
fn received_priority_above_200_is_clamped() {
    let cid = Uuid::from_u128(7);
    let p = data_packet(&cid, "Rogue", 1, 255, 1, 0, &[0; 512]);
    assert_eq!(parse_data_packet(&p).unwrap().priority, 200);
}
