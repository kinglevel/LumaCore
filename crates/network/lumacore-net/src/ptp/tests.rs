// ██████╗ ████████╗██████╗     ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗╚══██╔══╝██╔══██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██████╔╝   ██║   ██████╔╝       ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔═══╝    ██║   ██╔═══╝        ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║        ██║   ██║            ██║   ███████╗███████║   ██║   ███████║
// ╚═╝        ╚═╝   ╚═╝            ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: ptp tests

//! Tests of `ptp`.
//!
//! Owns:          the unit tests of ptp.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §24 "Step 9 hardening", §34 20.6.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn reads_pmc_output() {
    let out = "sending: GET TIME_STATUS_NP
	001122.fffe.334455-1 seq 0 RESPONSE MANAGEMENT TIME_STATUS_NP
		master_offset              -12
		ingress_time               1600000000000000000
		gmPresent                  true
		gmIdentity                 001122.fffe.aabbcc
	001122.fffe.334455-1 seq 1 RESPONSE MANAGEMENT PORT_DATA_SET
		portIdentity            001122.fffe.334455-1
		portState               SLAVE
	001122.fffe.334455-1 seq 2 RESPONSE MANAGEMENT CURRENT_DATA_SET
		stepsRemoved     1
		offsetFromMaster -12.0
		meanPathDelay    520.0
";
    let s = parse(out).unwrap();
    assert_eq!(s.offset_ns, -12);
    assert_eq!(s.path_delay_ns, 520);
    assert_eq!(s.grandmaster, "001122.fffe.aabbcc");
    assert!(s.gm_present);
    assert_eq!(s.port_state, "SLAVE");
    assert_eq!(s.steps_removed, 1);
    assert!(parse("sending: GET TIME_STATUS_NP\n").is_none());
}
