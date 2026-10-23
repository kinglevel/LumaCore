// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `fixtype`.
//!
//! Owns:          the unit tests of fixtype.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §21 "6d", §30 "12a2", §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn fixture_type_new_creates_empty_revisions() {
    let ft = FixtureType::new(
        "Robe".to_string(),
        Some("Mac Aura XB".to_string()),
        FixtureCategory::Wash,
    );
    assert_eq!(ft.name(), Some("Mac Aura XB"));
    assert_eq!(ft.manufacturer, "Robe");
    assert!(ft.revisions.is_empty());
    assert!(ft.latest_revision().is_none());
}

#[test]
fn fixture_type_node_type_tag() {
    let ft = FixtureType::new(
        "".to_string(),
        None,
        FixtureCategory::Other("X".to_string()),
    );
    assert_eq!(ft.node_type(), "FixtureType");
}
