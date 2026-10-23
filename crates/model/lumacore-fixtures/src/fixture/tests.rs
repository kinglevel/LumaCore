// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `fixture`.
//!
//! Owns:          the unit tests of fixture.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §21 "6d", §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn test_universe_guid(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn make_fixture(universe_n: u128, addr: u16, channels: u16) -> Fixture {
    Fixture::new(
        None,
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        test_universe_guid(universe_n),
        addr,
        channels,
    )
}

#[test]
fn fixture_single_break_created() {
    let ug = test_universe_guid(1);
    let f = make_fixture(1, 1, 24);
    assert_eq!(f.mvr.breaks.len(), 1);
    let b = f.primary_break().unwrap();
    assert_eq!(b.universe_id, ug);
    assert_eq!(b.start_address, 1);
    assert_eq!(b.channel_count, 24);
}

#[test]
fn fixture_break_end_address() {
    let ug = test_universe_guid(1);
    let b = FixtureBreak::new(ug, 1, 24);
    assert_eq!(b.end_address(), 24);

    let b2 = FixtureBreak::new(ug, 100, 10);
    assert_eq!(b2.end_address(), 109);
}

#[test]
fn fixture_total_channels_single_break() {
    let f = make_fixture(1, 1, 30);
    assert_eq!(f.total_channels(), 30);
}

#[test]
fn fixture_multi_break_total_channels() {
    let mut f = make_fixture(1, 1, 300);
    f.mvr.breaks
        .push(FixtureBreak::new(test_universe_guid(2), 1, 300));
    assert_eq!(f.total_channels(), 600);
}

#[test]
fn fixture_node_type_tag() {
    let f = make_fixture(1, 1, 1);
    assert_eq!(f.node_type(), "Fixture");
}
