// ████████╗██╗    ██╗    ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██║    ██║    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   ██║ █╗ ██║       ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██║███╗██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ╚███╔███╔╝       ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝    ╚══╝╚══╝        ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tw tests

//! Tests of `tree_window` on the demo rig: a place from a path or a GUID (whole, a prefix, a gone
//! one), the tree's rows opened down to the place with NAV's filter and the bookmark badges, the
//! inspector as JSON with its properties as settings cells (Group 1, Fixture 101, DataPool 1),
//! and the object Follow = Selected jumps to.
//!
//! Owns:          the unit tests of tree_window.rs.
//! Doesn't own:   the inspector's rows (inspector/tests.rs), the feeds (lumacore-http), drawing (e2e tree-window.spec.ts).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0292-tree-window-nav-in-the-web-and-app.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::{TreeParams, follow_target, inspector, place, tree};

const GROUP_1: &str = "Root.ShowData.DataPools.1.Groups.1";
const FIXTURE_101: &str = "Root.ShowData.Devices.Dmx.Patch.Fixtures.101";

fn group_guid(state: &lumacore_objects::AppState, no: u32) -> uuid::Uuid {
    state.root.show_data.datapools[&1].groups[&no].meta.guid()
}

/// A place is a typed path from the root, a whole GUID or a prefix of one; a gone GUID says so.
#[test]
fn a_place_is_a_path_or_a_guid() {
    let mut state = lumacore_objects::demo::fresh_state();
    assert_eq!(place(&mut state, GROUP_1).as_deref(), Ok(GROUP_1));
    assert_eq!(place(&mut state, "ShowData.DataPools.1.Groups.1").as_deref(), Ok(GROUP_1), "from the root");
    assert_eq!(place(&mut state, "").as_deref(), Ok(super::HOME));
    let guid = group_guid(&state, 1).to_string();
    assert_eq!(place(&mut state, &guid).as_deref(), Ok(GROUP_1));
    assert_eq!(place(&mut state, &guid[..super::GUID_PREFIX]).as_deref(), Ok(GROUP_1), "a prefix");
    let gone = place(&mut state, "00000000-dead-beef-0000-000000000000").unwrap_err();
    assert!(gone.starts_with("No object with GUID"), "{gone}");
    assert!(place(&mut state, "Root.ShowData.NoSuchNode").is_err());
}

/// The tree opens down to the place by itself; the rows carry depth, slot, kind and whether open.
#[test]
fn the_tree_opens_down_to_the_place() {
    let mut state = lumacore_objects::demo::fresh_state();
    let t = tree(&mut state, &TreeParams { path: GROUP_1.into(), ..TreeParams::default() });
    assert_eq!(t["path"], GROUP_1);
    let rows = t["rows"].as_array().expect("rows");
    let row = |p: &str| rows.iter().find(|r| r["path"] == p).unwrap_or_else(|| panic!("no {p}"));
    assert_eq!(row("Root.ShowData")["open"], true);
    assert_eq!(row("Root.ShowData.DataPools.1.Groups")["open"], true);
    let g = row(GROUP_1);
    assert_eq!(g["no"], 1);
    assert_eq!(g["type"], "Group");
    assert_eq!(g["target"], "Group 1");
    assert_eq!(g["depth"], 4);
    assert_eq!(g["guid"], group_guid(&state, 1).to_string());
    let groups = state.root.show_data.datapools[&1].groups.len();
    assert_eq!(rows.iter().filter(|r| r["type"] == "Group").count(), groups, "the whole pool");
    assert!(rows.iter().all(|r| !r["path"].as_str().unwrap_or("").starts_with("Root.ShowData.Devices.Dmx.Patch.Fixtures.")), "a closed branch sends nothing");
}

/// NAV's filter: a number keeps that slot in the open branches; the rest are counted as hidden.
#[test]
fn a_number_filters_to_its_slot() {
    let mut state = lumacore_objects::demo::fresh_state();
    let t = tree(&mut state, &TreeParams { path: FIXTURE_101.into(), search: "101".into(), ..TreeParams::default() });
    let objects: Vec<&str> = t["rows"].as_array().unwrap().iter().filter(|r| r["kind"] == "object").filter_map(|r| r["path"].as_str()).collect();
    assert!(objects.contains(&FIXTURE_101), "{objects:?}");
    assert!(objects.iter().all(|p| p.ends_with(".101")), "{objects:?}");
    assert!(t["hidden"].as_u64().unwrap() > 1000, "the rig's other fixtures are hidden");
}

/// A bookmark shows as its letter on the row and in the inspector.
#[test]
fn a_bookmark_is_a_badge() {
    let mut state = lumacore_objects::demo::fresh_state();
    let guid = group_guid(&state, 1);
    let user = state.effective_user_name().to_string();
    state.root.show_data.user_mut(&user).expect("the user").bookmarks.set('a', guid);
    let t = tree(&mut state, &TreeParams { path: GROUP_1.into(), ..TreeParams::default() });
    let row = t["rows"].as_array().unwrap().iter().find(|r| r["path"] == GROUP_1).unwrap().clone();
    assert_eq!(row["marks"], "a");
    assert_eq!(inspector(&mut state, GROUP_1)["marks"], "a");
}

/// Group 1's inspector: identity, its properties as settings cells with their Set and Reset
/// lines, its fixtures as links.
#[test]
fn group_1s_inspector_has_settings_cells() {
    let mut state = lumacore_objects::demo::fresh_state();
    let i = inspector(&mut state, GROUP_1);
    assert_eq!(i["target"], "Group 1");
    assert_eq!(i["guid"], group_guid(&state, 1).to_string());
    assert_eq!(i["path"], GROUP_1);
    let rows = i["rows"].as_array().unwrap();
    let mode = rows.iter().find(|r| r["name"] == "Mode").expect("Mode");
    assert_eq!(mode["section"], "property");
    assert_eq!(mode["field"]["line"], "Set Group 1 \"Mode\" {value}");
    assert_eq!(mode["field"]["input"]["type"], "choice");
    let name = rows.iter().find(|r| r["name"] == "Name").expect("Name");
    assert_eq!(name["field"]["changed"], true);
    assert_eq!(name["field"]["reset"], "Reset Group 1 \"Name\"");
    assert!(rows.iter().any(|r| r["section"] == "link" && r["link"].as_str().is_some_and(|l| l.starts_with("Root.ShowData.Devices.Dmx.Patch.Fixtures."))));
}

/// Fixture 101: its facts and the groups that use it, each a place to jump to.
#[test]
fn fixture_101_links_the_groups_that_use_it() {
    let mut state = lumacore_objects::demo::fresh_state();
    let i = inspector(&mut state, FIXTURE_101);
    assert_eq!(i["target"], "Fixture 101");
    let rows = i["rows"].as_array().unwrap();
    assert!(rows.iter().any(|r| r["name"] == "Used by" && r["link"] == GROUP_1), "{rows:#?}");
    assert_eq!(rows.iter().find(|r| r["name"] == "PosX").unwrap()["field"]["line"], "Set Fixture 101 \"PosX\" {value}");
}

/// DataPool 1: its pools, each openable.
#[test]
fn datapool_1_lists_its_pools() {
    let mut state = lumacore_objects::demo::fresh_state();
    let i = inspector(&mut state, "Root.ShowData.DataPools.1");
    assert_eq!(i["target"], "DataPool 1");
    let groups = i["rows"].as_array().unwrap().iter().find(|r| r["name"] == "Groups").cloned().unwrap();
    assert_eq!(groups["link"], "Root.ShowData.DataPools.1.Groups");
    let gone = inspector(&mut state, "deadbeef");
    assert!(gone["missing"].as_str().is_some_and(|m| m.starts_with("No object with GUID")), "{gone}");
}

/// Follow = Selected: the newest object a line named; a fixture means the selection's last.
#[test]
fn follow_goes_to_the_newest_pick() {
    let mut state = lumacore_objects::demo::fresh_state();
    let user = state.effective_user_name().to_string();
    let g3 = group_guid(&state, 3);
    let f101 = state.root.show_data.fixture_guid(101).unwrap();
    let f102 = state.root.show_data.fixture_guid(102).unwrap();
    let u = state.root.show_data.user_mut(&user).unwrap();
    u.usage.entries.clear();
    u.usage.record(&g3.to_string(), 1_000);
    let (path, key) = follow_target(&state).expect("a target");
    assert_eq!(path, "Root.ShowData.DataPools.1.Groups.3");
    let u = state.root.show_data.user_mut(&user).unwrap();
    u.usage.record(&f101.to_string(), 2_000);
    u.programmer.selected_fixtures = vec![f101, f102];
    let (path, key2) = follow_target(&state).expect("a target");
    assert_eq!(path, "Root.ShowData.Devices.Dmx.Patch.Fixtures.102", "the selection's last");
    assert_ne!(key, key2);
    let t = tree(&mut state, &TreeParams { follow: true, ..TreeParams::default() });
    assert_eq!(t["follow"]["path"], "Root.ShowData.Devices.Dmx.Patch.Fixtures.102");
}
