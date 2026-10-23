// ███████╗ ██████╗ ██╗   ██╗██████╗  ██████╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝██╔═══██╗██║   ██║██╔══██╗██╔════╝██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ███████╗██║   ██║██║   ██║██████╔╝██║     █████╗         ██║   █████╗  ███████╗   ██║   ███████╗
// ╚════██║██║   ██║██║   ██║██╔══██╗██║     ██╔══╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ███████║╚██████╔╝╚██████╔╝██║  ██║╚██████╗███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: source tests

//! Tests of `source`.
//!
//! Owns:          the unit tests of source.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §24 "Step 9 hardening".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn artdmx_header_and_port_address() {
    let mut data = [0u8; 512];
    data[0] = 0xFF;
    data[511] = 0x42;
    let p = artdmx_packet(0x0123, 9, &data);
    assert_eq!(p.len(), 530);
    assert_eq!(&p[0..8], b"Art-Net\0");
    assert_eq!(&p[8..12], &[0x00, 0x50, 0x00, 0x0E]);
    assert_eq!(p[12], 9);
    assert_eq!((p[14], p[15]), (0x23, 0x01)); // SubUni, Net
    assert_eq!(&p[16..18], &[0x02, 0x00]);
    assert_eq!((p[18], p[529]), (0xFF, 0x42));
}

#[test]
fn sequence_wraps_to_one_never_zero() {
    assert_eq!(next_sequence(0), 1);
    assert_eq!(next_sequence(255), 1);
}

#[test]
fn poll_reply_gives_output_port_addresses() {
    let mut p = vec![0u8; 239];
    p[..8].copy_from_slice(b"Art-Net\0");
    p[8..10].copy_from_slice(&OP_POLL_REPLY.to_le_bytes());
    p[10..14].copy_from_slice(&[10, 0, 0, 50]);
    p[18] = 1; // Net
    p[19] = 2; // SubNet
    p[26..30].copy_from_slice(b"Node");
    p[173] = 2; // two ports
    p[174] = 0x80; // port 1 outputs DMX
    p[175] = 0x40; // port 2 is an input
    p[190] = 3;
    p[191] = 4;
    let node = parse_poll_reply(&p).unwrap();
    assert_eq!(node.ip, Ipv4Addr::new(10, 0, 0, 50));
    assert_eq!(node.short_name, "Node");
    assert_eq!(node.outputs, vec![0x0123]);
    assert_eq!(node.refresh_hz, Some(44));
    p[226..228].copy_from_slice(&100u16.to_be_bytes());
    assert_eq!(parse_poll_reply(&p).unwrap().refresh_hz, Some(100));
    assert_eq!(parse_poll_reply(&p[..200]).unwrap().refresh_hz, None);
    assert!(parse_poll_reply(&p[..100]).is_none());
    let sync = artsync_packet();
    assert_eq!(&sync[8..12], &[0x00, 0x52, 0x00, 0x0E]);
    assert_eq!(artpoll_packet().len(), 14);
}

fn loopback() -> Nic {
    Nic {
        name: "lo".into(),
        ip: Ipv4Addr::LOCALHOST,
        netmask: Ipv4Addr::new(255, 0, 0, 0),
        broadcast: Ipv4Addr::new(127, 255, 255, 255),
        index: None,
    }
}

#[test]
fn a_node_with_two_binds_keeps_every_port() {
    let ip = Ipv4Addr::new(10, 1, 0, 9);
    let mut src = ArtNetSource::new(&loopback()).unwrap();
    // Eight ports: two replies, BindIndex 1 and 2.
    for reply in poll_reply_packets(ip, "Node8", &[1, 2, 3, 4, 5, 6, 7, 8]) {
        src.note_node(parse_poll_reply(&reply).unwrap());
    }
    for pa in 1..=8 {
        assert_eq!(src.targets(pa, None), vec![ip], "port {pa}");
    }
    let nodes = src.nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].outputs, (1..=8).collect::<Vec<u16>>());
    // A reply without BindIndex is bind 1.
    let mut old = poll_reply_packets(ip, "Old", &[9]).remove(0);
    old.truncate(200);
    assert_eq!(parse_poll_reply(&old).unwrap().bind_index, 1);
}

#[test]
fn a_failing_target_never_starves_the_next() {
    // Bound to loopback, a send to a routed address fails at once.
    let nic = loopback();
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    s.bind(&SocketAddr::V4(SocketAddrV4::new(nic.ip, 0)).into()).unwrap();
    let mut src = ArtNetSource::new(&nic).unwrap();
    src.socket = s.into();
    let bad = Ipv4Addr::new(192, 0, 2, 1);
    let good = Ipv4Addr::LOCALHOST;
    src.prepare([(bad, 1), (good, 1)].into_iter());
    let result = src.send(1, &[0; 512], &[bad, good]);
    assert!(result.is_err(), "{result:?}");
    // The second target was still sent to (its sequence moved on).
    assert_eq!(src.sequences[&(good, 1)], 1);
}

#[test]
fn artdmx_round_trips() {
    let mut data = [0u8; 512];
    data[3] = 99;
    let (pa, got) = parse_artdmx(&artdmx_packet(0x0123, 1, &data)).unwrap();
    assert_eq!((pa, got[3]), (0x0123, 99));
    assert!(parse_artdmx(&artpoll_packet()).is_none());
    assert!(is_poll(&artpoll_packet()));
}

#[test]
fn poll_replies_list_inputs_four_per_bind() {
    let ip = Ipv4Addr::new(10, 1, 0, 9);
    // 0.0.1–0.0.5 and 1.2.3: two binds for SubNet 0, one for 1.2.
    let replies = poll_reply_packets(ip, "LumaCore", &[1, 2, 3, 4, 5, 0x0123]);
    assert_eq!(replies.len(), 3);
    let nodes: Vec<ArtNetNode> = replies.iter().map(|r| parse_poll_reply(r).unwrap()).collect();
    assert_eq!(nodes[0].outputs, vec![1, 2, 3, 4]);
    assert_eq!(nodes[1].outputs, vec![5]);
    assert_eq!(nodes[2].outputs, vec![0x0123]);
    assert_eq!((nodes[0].ip, nodes[0].short_name.as_str()), (ip, "LumaCore"));
    assert_eq!((replies[0][211], replies[1][211]), (1, 2));
}
