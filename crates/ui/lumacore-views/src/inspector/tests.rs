// ██╗███╗   ██╗███████╗██████╗     ████████╗███████╗███████╗████████╗███████╗
// ██║████╗  ██║██╔════╝██╔══██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║██╔██╗ ██║███████╗██████╔╝       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║██║╚██╗██║╚════██║██╔═══╝        ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║██║ ╚████║███████║██║            ██║   ███████╗███████║   ██║   ███████║
// ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝            ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: insp tests

//! Tests of `inspector` on the demo rig: Fixture 101 and Group 1 show their identity, facts,
//! properties and links; a datapool shows its pools; a node its counts.
//!
//! Owns:          the unit tests of inspector.rs.
//! Doesn't own:   drawing and keys (lumacore-terminal's NAV tests).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::{Section, view};

fn value<'a>(i: &'a super::Inspector, name: &str) -> &'a super::Row {
    i.rows.iter().find(|r| r.name == name).unwrap_or_else(|| panic!("no {name} in {:#?}", i.rows))
}

/// Fixture 101: Slot, Kind, GUID, Path, its type / mode / location, its declared properties
/// with their Set lines, and the groups that use it, each a path NAV follows.
#[test]
fn fixture_101_has_identity_facts_properties_and_links() {
    let mut state = lumacore_objects::demo::fresh_state();
    let path = "Root.ShowData.Devices.Dmx.Patch.Fixtures.101";
    let i = view(&mut state, path);
    assert_eq!(i.target.as_deref(), Some("Fixture 101"));
    assert_eq!(value(&i, "Slot").value, "101");
    assert_eq!(value(&i, "Kind").value, "Fixture");
    assert_eq!(value(&i, "Path").value, path);
    let guid = state.root.show_data.fixture_guid(101).expect("fixture 101");
    assert_eq!(value(&i, "GUID").value, guid.to_string());
    assert!(value(&i, "Type").value.contains("PHOTON"), "{:?}", value(&i, "Type"));
    assert!(value(&i, "Location").value.starts_with("Universe 7"), "{:?}", value(&i, "Location"));
    let posx = value(&i, "PosX");
    assert_eq!(posx.section, Section::Property);
    assert_eq!(posx.set.as_deref(), Some("Set Fixture 101 \"PosX\" "));
    let used: Vec<_> = i.rows.iter().filter(|r| r.name == "Used by").collect();
    assert!(used.iter().any(|r| r.value.starts_with("Group 1") && r.link.as_deref() == Some("Root.ShowData.DataPools.1.Groups.1")), "{used:#?}");
}

/// Group 1: its properties from the registry, a changed one marked with its Reset line, and
/// its members as links.
#[test]
fn group_1_marks_changed_properties_and_links_its_fixtures() {
    let mut state = lumacore_objects::demo::fresh_state();
    let i = view(&mut state, "Root.ShowData.DataPools.1.Groups.1");
    assert_eq!(i.target.as_deref(), Some("Group 1"));
    assert_eq!(i.name.as_deref(), Some("Photon"));
    let name = value(&i, "Name");
    assert!(name.changed, "a named group differs from the empty default");
    assert_eq!(name.reset.as_deref(), Some("Reset Group 1 \"Name\""));
    let note = value(&i, "Note");
    assert!(!note.changed && note.reset.is_none(), "{note:?}");
    let members = state.root.show_data.datapools[&1].groups[&1].members.len();
    assert_eq!(value(&i, "Fixtures").value, members.to_string());
    let links = i.rows.iter().filter(|r| r.name == "Links to" && r.link.is_some()).count();
    assert_eq!(links, members, "{:#?}", i.rows);
}

/// DataPool 1 shows its pools with their counts (as `Info DataPool 1`), each one openable.
#[test]
fn a_datapool_shows_its_pools() {
    let mut state = lumacore_objects::demo::fresh_state();
    let i = view(&mut state, "Root.ShowData.DataPools.1");
    assert_eq!(i.target.as_deref(), Some("DataPool 1"));
    assert!(i.guid.is_some());
    let groups = value(&i, "Groups");
    assert_eq!(groups.value, "77 groups");
    assert_eq!(groups.link.as_deref(), Some("Root.ShowData.DataPools.1.Groups"));
}

/// A node shows its path and what it holds.
#[test]
fn a_node_shows_its_counts() {
    let mut state = lumacore_objects::demo::fresh_state();
    let i = view(&mut state, "Root.ShowData.Devices.Dmx.Patch.Fixtures");
    assert_eq!(i.target, None);
    assert_eq!(value(&i, "Kind").value, "Node");
    assert!(value(&i, "Holds").value.ends_with("fixtures patched"));
}
