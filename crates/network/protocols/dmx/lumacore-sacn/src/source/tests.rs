// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `sacn source`.
//!
//! Owns:          the unit tests of sacn source.rs.
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
fn multicast_addresses() {
    assert_eq!(
        multicast_addr_for_universe(1),
        Ipv4Addr::new(239, 255, 0, 1)
    );
    assert_eq!(
        multicast_addr_for_universe(256),
        Ipv4Addr::new(239, 255, 1, 0)
    );
    assert_eq!(
        multicast_addr_for_universe(63999),
        Ipv4Addr::new(239, 255, 249, 255)
    );
}

#[test]
fn data_packet_layout_follows_e131() {
    let cid = Uuid::from_u128(0x0102);
    let mut data = [0u8; 512];
    data[0] = 0xAB;
    data[511] = 0xCD;
    let p = data_packet(&cid, "LumaCore", 0x0102, 150, 7, 0, &data);
    assert_eq!(p.len(), 638);
    assert_eq!(&p[4..16], b"ASC-E1.17\0\0\0");
    assert_eq!(&p[16..18], &[0x72, 0x6E]); // root flags+length 622
    assert_eq!(&p[18..22], &[0, 0, 0, 4]);
    assert_eq!(&p[22..38], cid.as_bytes());
    assert_eq!(&p[38..40], &[0x72, 0x58]); // framing 600
    assert_eq!(&p[44..52], b"LumaCore");
    assert_eq!(p[108], 150);
    assert_eq!(p[111], 7);
    assert_eq!(&p[113..115], &[0x01, 0x02]);
    assert_eq!(&p[115..117], &[0x72, 0x0B]); // DMP 523
    assert_eq!(&p[117..125], &[0x02, 0xA1, 0, 0, 0, 1, 0x02, 0x01]);
    assert_eq!((p[125], p[126], p[637]), (0, 0xAB, 0xCD));
}

#[test]
fn sync_packet_and_in_place_send_layout() {
    let cid = Uuid::from_u128(7);
    let p = sync_packet(&cid, 9, SYNC_UNIVERSE);
    assert_eq!(&p[16..18], (0x7000u16 | 33).to_be_bytes()); // root 33
    assert_eq!(&p[18..22], &[0, 0, 0, 8]);
    assert_eq!(&p[38..40], (0x7000u16 | 11).to_be_bytes()); // framing 11
    assert_eq!(&p[40..44], &[0, 0, 0, 1]);
    assert_eq!(p[44], 9);
    assert_eq!(&p[45..47], &SYNC_UNIVERSE.to_be_bytes());

    let mut src = SacnSource::new(Ipv4Addr::LOCALHOST, cid, "T").unwrap();
    let mut data = [0u8; 512];
    data[0] = 5;
    let _ = src.send(300, &data, 150, Destination::Unicast(Ipv4Addr::LOCALHOST), true);
    let expected = data_packet(&cid, "T", 300, 150, 1, 0, &data);
    assert_eq!(&src.packet[..109], &expected[..109]);
    assert_eq!(&src.packet[109..111], &SYNC_UNIVERSE.to_be_bytes());
    assert_eq!(&src.packet[111..], &expected[111..]);
}

#[test]
fn discovery_packet_lists_universes() {
    let cid = Uuid::nil();
    let p = discovery_packet(&cid, "x", &[1, 2, 300]);
    assert_eq!(p.len(), 126);
    assert_eq!(&p[18..22], &[0, 0, 0, 8]);
    assert_eq!(&p[112..114], (0x7000u16 | 14).to_be_bytes());
    assert_eq!(&p[120..126], &[0, 1, 0, 2, 1, 44]);
}
