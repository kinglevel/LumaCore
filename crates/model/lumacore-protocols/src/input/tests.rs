// ██╗███╗   ██╗██████╗ ██╗   ██╗████████╗    ████████╗███████╗███████╗████████╗███████╗
// ██║████╗  ██║██╔══██╗██║   ██║╚══██╔══╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║██╔██╗ ██║██████╔╝██║   ██║   ██║          ██║   █████╗  ███████╗   ██║   ███████╗
// ██║██║╚██╗██║██╔═══╝ ██║   ██║   ██║          ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║██║ ╚████║██║     ╚██████╔╝   ██║          ██║   ███████╗███████║   ██║   ███████║
// ╚═╝╚═╝  ╚═══╝╚═╝      ╚═════╝    ╚═╝          ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: input tests

//! Tests of `input`.
//!
//! Owns:          the unit tests of input.rs.
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

use super::*;

fn packet(key: u8, priority: u8, universe: u16, value: u8) -> Packet {
    let mut data = [0u8; 512];
    data[0] = value;
    data[1] = value / 2;
    let ip = Ipv4Addr::new(10, 0, 0, key);
    Packet::dmx(SourceKey::Ip(ip), SourceName::from_ip(ip), ip, priority, universe, data)
}

fn plan(routes: Vec<(u32, u16)>) -> Vec<InputPlan> {
    vec![InputPlan {
        line: 1,
        source: None,
        band: InputPriority::Ltp,
        routes,
    }]
}

#[test]
fn highest_priority_wins_then_htp() {
    let mut m = InputMerger::new();
    let t = Instant::now();
    m.ingest(&[packet(1, 100, 1, 200), packet(2, 100, 1, 50)], t);
    let l = m.layers(&plan(vec![(5, 1)]), t).unwrap();
    assert_eq!((l[0].universe, l[0].data[0]), (5, 200), "HTP at equal priority");
    m.ingest(&[packet(3, 150, 1, 10)], t);
    let l = m.layers(&plan(vec![(5, 1)]), t).unwrap();
    assert_eq!(l[0].data[0], 10, "higher sACN priority wins");
    assert_eq!(m.sources().len(), 3);
}

#[test]
fn target_filters_and_one_universe_feeds_two() {
    let mut m = InputMerger::new();
    let t = Instant::now();
    m.ingest(&[packet(1, 100, 1, 200), packet(2, 100, 1, 50)], t);
    let mut p = plan(vec![(5, 1), (6, 1)]);
    p[0].source = Some(Ipv4Addr::new(10, 0, 0, 2));
    let l = m.layers(&p, t).unwrap();
    assert_eq!(l.len(), 2);
    assert!(l.iter().all(|l| l.data[0] == 50));
}

#[test]
fn loss_terminate_and_change_stamps() {
    let mut m = InputMerger::new();
    let t = Instant::now();
    m.ingest(&[packet(1, 100, 1, 200)], t);
    let first = m.layers(&plan(vec![(5, 1)]), t).unwrap();
    // Nothing new: no change.
    assert!(m.layers(&plan(vec![(5, 1)]), t).is_none());
    // Channel 1 changes, channel 3 doesn't: only changed channels restamp.
    let mut p = packet(1, 100, 1, 200);
    p.data[0] = 7;
    m.ingest(&[p], t);
    let second = m.layers(&plan(vec![(5, 1)]), t).unwrap();
    assert!(second[0].stamps[0] > first[0].stamps[0]);
    assert_eq!(second[0].stamps[2], first[0].stamps[2]);
    // Lost after 2.5 s.
    assert!(
        m.layers(&plan(vec![(5, 1)]), t + SOURCE_TIMEOUT)
            .unwrap()
            .is_empty()
    );
    // Terminated at once.
    m.ingest(&[packet(1, 100, 1, 9)], t);
    assert_eq!(m.layers(&plan(vec![(5, 1)]), t).unwrap().len(), 1);
    let mut end = packet(1, 100, 1, 0);
    end.terminated = true;
    m.ingest(&[end], t);
    assert!(m.layers(&plan(vec![(5, 1)]), t).unwrap().is_empty());
}

fn sacn(seq: u8, value: u8, sync: u16) -> Packet {
    let mut p = packet(1, 100, 1, value);
    p.sequence = Some(seq);
    p.sync = sync;
    p
}

#[test]
fn late_and_duplicate_sequence_numbers_are_dropped() {
    assert!(out_of_order(10, 10) && out_of_order(10, 9) && out_of_order(10, 247));
    assert!(!out_of_order(10, 11) && !out_of_order(10, 245) && !out_of_order(255, 0));
    let mut m = InputMerger::new();
    let t = Instant::now();
    m.ingest(&[sacn(10, 100, 0)], t);
    m.ingest(&[sacn(9, 50, 0)], t); // late: dropped
    assert_eq!(m.layers(&plan(vec![(5, 1)]), t).unwrap()[0].data[0], 100);
    m.ingest(&[sacn(200, 70, 0)], t); // big jump: a restarted source
    assert_eq!(m.layers(&plan(vec![(5, 1)]), t).unwrap()[0].data[0], 70);
}

#[test]
fn synced_data_waits_for_its_sync_packet_then_falls_back() {
    let mut m = InputMerger::new();
    let t = Instant::now();
    let sync = |seq: u8| {
        let mut p = packet(1, 100, 7, 0);
        p.is_sync = true;
        p.sequence = Some(seq);
        p
    };
    // No sync heard yet: data is taken at once.
    m.ingest(&[sacn(1, 10, 7)], t);
    assert_eq!(m.layers(&plan(vec![(5, 1)]), t).unwrap()[0].data[0], 10);
    // Sync alive: new data waits for the sync packet.
    m.ingest(&[sync(1)], t);
    m.ingest(&[sacn(2, 90, 7)], t);
    assert!(m.layers(&plan(vec![(5, 1)]), t).is_none(), "held until sync");
    m.ingest(&[sync(2)], t);
    assert_eq!(m.layers(&plan(vec![(5, 1)]), t).unwrap()[0].data[0], 90);
    // Sync goes silent: held data is taken after 2.5 s.
    m.ingest(&[sacn(3, 40, 7)], t);
    let later = t + SOURCE_TIMEOUT;
    m.ingest(&[sacn(4, 40, 7)], later);
    assert_eq!(m.layers(&plan(vec![(5, 1)]), later).unwrap()[0].data[0], 40);
}

#[test]
fn source_names_need_no_heap() {
    assert_eq!(SourceName::from_bytes(b"Desk A\0\0junk").as_str(), "Desk A");
    assert_eq!(SourceName::from_ip(Ipv4Addr::new(10, 0, 0, 12)).as_str(), "10.0.0.12");
    assert_eq!(SourceName::from_bytes(&[b'x'; 80]).as_str().len(), 64);
}
