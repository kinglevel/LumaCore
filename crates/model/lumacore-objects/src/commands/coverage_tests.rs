//! Step 6a (decisions §21 "Step 6 hardening"): undo/replication cover the
//! whole show, programmers belong to users, undo lives in Runtime, shows
//! carry their format — through real dispatch.

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;

pub(super) fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn as_user(state: &mut AppState, user: &str) {
    state.user = user.to_string();
}

pub(super) fn with_fixture() -> AppState {
    let mut state = AppState::new();
    let mut ev = Vec::new();
    lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    let ft = lumacore_fixtures::fixture_type::FixtureType {
        meta: lumacore_tree::node::NodeMeta::new(Some("Dim".into())),
        manufacturer: "M".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![lumacore_fixtures::fixture_type::Revision {
            meta: lumacore_tree::node::NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![lumacore_fixtures::fixture_type::Mode {
                meta: lumacore_tree::node::NodeMeta::new(None),
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
    let guid = ft.meta.identity.guid;
    let show = &mut state.root.show_data;
    show.devices.dmx.patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    lumacore_patch::PatchEngine::patch_fixture(show, guid, 0, 0, 1, 1, None).unwrap();
    // Like an import: the type's attributes are defined with it.
    lumacore_tree::attribute_resolver::sync(show);
    state
}

#[test]
fn coverage_creating_and_deleting_users_and_profiles_is_undone() {
    let mut state = AppState::new();
    run(&mut state, "Store User \"LD\" /Rights Program");
    let profiles = state.root.show_data.user_profiles.len();
    assert!(state.root.show_data.user("LD").is_some());
    assert!(run(&mut state, "Oops").contains("Undone"), "user creation is undoable");
    assert!(state.root.show_data.user("LD").is_none());
    assert_eq!(state.root.show_data.user_profiles.len(), profiles - 1, "its new profile too");
    // Delete and bring back.
    run(&mut state, "Store User \"LD\" /Rights Program");
    run(&mut state, "Delete User \"LD\" /NoConfirmation");
    assert!(state.root.show_data.user("LD").is_none());
    run(&mut state, "Oops");
    assert!(state.root.show_data.user("LD").is_some());
}

#[test]
fn coverage_programmers_belong_to_users_not_profiles() {
    let mut state = with_fixture();
    // Two users on one profile keep separate programmers.
    run(&mut state, "Store User \"Phone\" /Rights Program /Profile \"Admin\"");
    run(&mut state, "Fixture 1");
    as_user(&mut state, "Phone");
    assert_eq!(
        state.current_programmer().unwrap().selected_fixtures.len(),
        0,
        "Phone shares Admin's profile but not Admin's selection"
    );
    // Moving to another profile keeps the user's work.
    run(&mut state, "Fixture 1");
    as_user(&mut state, "Admin");
    run(&mut state, "Set User \"Phone\" \"Profile\" \"Debug\"");
    assert_eq!(state.root.show_data.user("Phone").unwrap().programmer.selected_fixtures.len(), 1);
    // A profile no longer has a programmer; an old show's is dropped.
    let json = serde_json::to_value(state.root.show_data.profile_for_user("Admin").unwrap()).unwrap();
    assert!(json.get("programmer").is_none() && json.get("undo_stack").is_none());
}

#[test]
fn coverage_undo_lives_in_runtime_and_is_not_saved() {
    let mut state = with_fixture();
    run(&mut state, "Fixture 1 At 50");
    assert!(state.undo_stack("Admin").is_some_and(|u| !u.is_empty()));
    let saved = serde_json::to_string(&state.root.show_data).unwrap();
    assert!(!saved.contains("Fixture 1 At 50"), "undo frames are not in the show");
    let tree = run(&mut state, "List Root.Runtime.Users.1");
    assert!(tree.contains("Undo"), "{tree}");
    // What the tree shows there (`cat Root.Runtime.Users.1.Undo`) is the tree shell's test.
    let undo = format!("{:?}", state.undo_stack("Admin"));
    assert!(undo.contains("Fixture 1 At 50"), "{undo}");
}

#[test]
fn coverage_clear_programmer_user_is_undone_by_the_affected_user() {
    let mut state = with_fixture();
    as_user(&mut state, "Debug");
    run(&mut state, "Fixture 1 At 40");
    as_user(&mut state, "Admin");
    run(&mut state, "Clear Programmer User 2");
    assert!(state.root.show_data.users[&2].programmer.values.is_empty());
    let steps = |s: &AppState, u: &str| s.undo_stack(u).map_or(0, |u| u.len());
    let admin_steps = steps(&state, "Admin");
    as_user(&mut state, "Debug");
    assert!(run(&mut state, "Oops").contains("Undone"));
    assert!(!state.root.show_data.users[&2].programmer.values.is_empty());
    assert_eq!(steps(&state, "Admin"), admin_steps, "nothing filed on the Admin");
}

#[test]
fn coverage_oops_is_replicated() {
    let mut state = with_fixture();
    run(&mut state, "Fixture 1 At 50");
    state.replication_outbox.clear();
    run(&mut state, "Oops");
    assert!(
        state.replication_outbox.iter().any(|op| op.command == "Oops"),
        "an undo goes out like any change"
    );
}

#[test]
fn coverage_shows_carry_their_format_and_newer_ones_are_refused() {
    use crate::parse_show;
    let mut show = lumacore_tree::ShowData::new();
    show.show_settings.saved_with_commit = "abc123".into();
    let json = serde_json::to_value(&show).unwrap();
    assert_eq!(json["show_settings"]["format_version"], lumacore_tree::FORMAT_VERSION);
    assert!(parse_show(&json.to_string()).is_ok());
    let mut newer = json.clone();
    newer["show_settings"]["format_version"] = (lumacore_tree::FORMAT_VERSION + 1).into();
    let err = parse_show(&newer.to_string()).unwrap_err();
    assert!(err.contains("E-SHOW-001") && err.contains("abc123"), "{err}");
    // Older formats are refused too: shows only move forward (§21 "6d").
    let mut older = json.clone();
    older["show_settings"]
        .as_object_mut()
        .unwrap()
        .remove("format_version");
    assert!(parse_show(&older.to_string()).unwrap_err().contains("E-SHOW-002"));
    // An object without a GUID, or two sharing one, is refused.
    let mut no_guid = json.clone();
    no_guid["users"]["1"]["meta"].as_object_mut().unwrap().remove("guid");
    assert!(parse_show(&no_guid.to_string()).unwrap_err().contains("E-SHOW-003"));
    let mut shared = json;
    shared["users"]["2"]["meta"]["guid"] = shared["users"]["1"]["meta"]["guid"].clone();
    let err = parse_show(&shared.to_string()).unwrap_err();
    assert!(err.contains("E-SHOW-003") && err.contains("Users.1"), "{err}");
}
