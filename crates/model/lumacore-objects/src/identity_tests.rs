//! Strict identity (decisions §21 "6d hardening"): every object in the
//! show has its own GUID, every stored link resolves, links survive renames
//! and moves, copies are new objects, and replication/undo name objects by
//! GUID.

use crate::AppState;
use crate::commands::dispatch;
use crate::DispatchResult;
use serde_json::Value;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => String::new(),
    }
}

/// A show with a bit of everything, made with commands.
fn full_show() -> AppState {
    let mut state = AppState::new();
    let patch = &mut state.root.show_data.devices.dmx.patch;
    let universe = lumacore_fixtures::Universe::new(Some("Main".into()));
    let ug = universe.meta.identity.guid;
    patch.universes.insert(1, universe);
    let ft = lumacore_fixtures::FixtureType {
        meta: lumacore_tree::NodeMeta::new(Some("Dim".into())),
        manufacturer: "Test".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![lumacore_fixtures::Revision {
            meta: lumacore_tree::NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![lumacore_fixtures::Mode {
                meta: lumacore_tree::NodeMeta::new(None),
                channel_functions: vec![lumacore_fixtures::fixture_type::ChannelFunction {
                    attribute_name: "Dimmer".into(),
                    attribute_type: lumacore_fixtures::fixture_type::AttributeType::Dimmer,
                    byte_count: 1,
                    resolved: true,
                    ..Default::default()
                }],
                dmx_footprint: 1,
                geometry: String::new(),
                cells: Vec::new(),
                relations: Vec::new(),
            }],
        }],
    };
    let (ftg, (rev, mode)) = (ft.meta.identity.guid, ft.mode_guids(0, 0).unwrap());
    patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    for id in 1..=4u32 {
        patch.insert_fid(id, lumacore_fixtures::Fixture::new(None, ftg, rev, mode, ug, id as u16, 1),
        );
    }
    state.root.show_data.after_load();
    for line in [
        "Fixture 1 Thru 2",
        "Store Group 1 \"Front\"",
        "At 50",
        "Store Sequence 1 Cue 1",
        "Select Sequence 1",
        "At 80",
        "Store Cue 2",
        "Clear",
        "Clear",
        "Clear",
        "Store Network \"Rig\"",
        "Store Output 1 /Network \"Rig\"",
        "Assign Universe 1 At Output 1 /Universe 1",
        "Store Input 2 /Protocol Osc /Network \"Rig\" /Port 9000 /User \"Admin\"",
        "Store User \"Busk\"",
        "Park DMX 1.10 At 5",
        "Go",
    ] {
        run(&mut state, line);
    }
    state
}

/// Every JSON object kept in a numbered pool (a map keyed by numbers) must
/// carry a GUID — so a pool walked as plain data can't hide objects.
/// Windows are a screen's content (numbered inside it, §21 "6d").
fn pools_without_guid(v: &Value, path: &str, out: &mut Vec<String>) {
    match v {
        Value::Object(map) => {
            let numbered = !map.is_empty() && map.keys().all(|k| k.parse::<u64>().is_ok());
            let windows = path.ends_with(".windows");
            for (k, child) in map {
                let p = format!("{path}.{k}");
                if numbered
                    && !windows
                    && child.is_object()
                    && child.get("guid").is_none()
                    && child.get("meta").and_then(|m| m.get("guid")).is_none()
                {
                    out.push(p.clone());
                }
                pools_without_guid(child, &p, out);
            }
        }
        Value::Array(list) => {
            for (i, child) in list.iter().enumerate() {
                pools_without_guid(child, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

#[test]
fn identity_every_object_has_a_unique_guid_and_every_link_resolves() {
    let state = full_show();
    let report = state.root.show_data.identity();
    assert!(report.is_clean(), "{:#?}", crate::references::problems(&report));
    assert!(report.objects > 20 && report.links > 10, "{} objects, {} links", report.objects, report.links);
    let json = serde_json::to_value(&state.root.show_data).unwrap();
    let mut missing = Vec::new();
    pools_without_guid(&json, "show", &mut missing);
    assert!(missing.is_empty(), "objects without a GUID: {missing:?}");
}

#[test]
fn identity_copies_are_new_objects() {
    let mut state = full_show();
    run(&mut state, "Copy Group 1 At 7");
    let groups = &state.root.show_data.datapools[&1].groups;
    assert_ne!(groups[&1].meta.identity.guid, groups[&7].meta.identity.guid);
    assert!(state.root.show_data.identity().duplicates.is_empty());
}

#[test]
fn identity_undo_and_replication_name_pool_entries_by_guid() {
    use crate::commands::oops::{restore_subtrees, snapshot_subtrees};
    use lumacore_tree::ShowSubtree;
    let mut state = full_show();
    let no = state.root.show_data.user_no("Busk").unwrap();
    let guid = state.root.show_data.users[&no].meta.identity.guid;
    let before = snapshot_subtrees(&state, &[ShowSubtree::User(guid)]).unwrap();
    // Renumber the user (as another station's change would arrive): the
    // snapshot still means this user, wherever it sits.
    let moved = state.root.show_data.users.remove(&no).unwrap();
    state.root.show_data.users.insert(40, moved);
    restore_subtrees(&mut state, &before).unwrap();
    let users = &state.root.show_data.users;
    assert_eq!(users[&no].meta.identity.guid, guid);
    assert!(!users.contains_key(&40), "the object left its old slot");
    // A slot held by another object is never overwritten.
    let admin_no = state.root.show_data.user_no("Admin").unwrap();
    let clash = state.root.show_data.users[&no].clone();
    let data = serde_json::to_vec(&(admin_no, &clash)).unwrap();
    let snap = lumacore_tree::SubtreeSnapshot {
        subtree: ShowSubtree::User(guid),
        data,
    };
    assert!(restore_subtrees(&mut state, &[snap]).unwrap_err().contains("holds another object"));
}

