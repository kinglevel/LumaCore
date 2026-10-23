//! Behaviour of the one-namespace registry through real dispatch and
//! completion (decisions.md §15).

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn modes(state: &AppState) -> lumacore_tree::Modes {
    state
        .root
        .runtime
        .user_ref(state.effective_user_name())
        .map(|u| u.modes)
        .unwrap_or_default()
}

#[test]
fn mode_objects_take_their_verb_first_and_are_per_user() {
    let mut state = AppState::new();
    let out = run(&mut state, "On Blind");
    assert!(out.contains("Blind On"), "{out}");
    assert!(modes(&state).blind);
    run(&mut state, "Blind"); // the bare word toggles
    assert!(!modes(&state).blind);
    run(&mut state, "Toggle Freeze");
    run(&mut state, "On hi");
    assert!(modes(&state).freeze && modes(&state).highlight);
    run(&mut state, "Off Highlight");
    assert!(!modes(&state).highlight);
    // The postfix form points at the verb-first one.
    let out = run(&mut state, "Blind On");
    assert!(
        out.contains("On Blind") && out.contains("E-CMD-005"),
        "{out}"
    );
    assert!(!modes(&state).blind);
    assert!(run(&mut state, "Toggle").contains("E-CMD-004"));
    // Another user has their own modes.
    let admin = state.effective_user_name().to_string();
    assert!(state.root.runtime.user("Guest").modes == lumacore_tree::Modes::default());
    assert!(state.root.runtime.user_ref(&admin).unwrap().modes.freeze);
    // Visible in the tree.
    let tree = run(&mut state, "List Root.Runtime.Users");
    assert!(tree.contains(&admin), "{tree}");
}

fn patch_fixtures(state: &mut AppState, ids: &[u32]) {
    for &id in ids {
        state.root.show_data.devices.dmx.patch.insert_fid(id, lumacore_fixtures::Fixture::new(Some(format!("F{id}")),
                uuid::Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                1,
                1,
            ),
        );
    }
}

#[test]
fn on_knocks_in_and_off_knocks_out() {
    let mut state = AppState::new();
    patch_fixtures(&mut state, &[1, 2]);
    run(&mut state, "Fixture 1 Thru 2");
    run(&mut state, "At 50");
    let g1 = state.root.show_data.fixture_guid(1).unwrap();
    let g2 = state.root.show_data.fixture_guid(2).unwrap();
    state.current_programmer_mut().unwrap().deactivate_values();
    // Bare On = the selection's values become active.
    let out = run(&mut state, "On");
    assert!(out.contains("activated"), "{out}");
    let prog = state.current_programmer().unwrap();
    assert!(prog.is_value_active(g1, "Dimmer") && prog.is_value_active(g2, "Dimmer"));
    // Off Fixture 1 knocks only fixture 1 out: values and selection.
    run(&mut state, "Off Fixture 1");
    let prog = state.current_programmer().unwrap();
    assert!(prog.get_value(g1, "Dimmer").is_none() && !prog.is_selected(g1));
    assert!(prog.get_value(g2, "Dimmer").is_some() && prog.is_selected(g2));
    // Bare Off knocks the selection out and deselects it.
    run(&mut state, "Off");
    let prog = state.current_programmer().unwrap();
    assert!(prog.values.is_empty() && prog.selected_fixtures.is_empty());
}

#[test]
fn unterminated_quote_points_at_the_quote() {
    let mut state = AppState::new();
    let out = run(&mut state, "Label Group 1 \"Front");
    assert!(out.contains("Unterminated quote"), "{out}");
    assert!(out.contains('^'), "{out}");
}

#[test]
fn objects_without_call_show_their_list() {
    let mut state = AppState::new();
    let out = run(&mut state, "selection");
    assert!(!out.contains("doesn't take"), "{out}");
}

#[test]
fn list_reaches_tree_nodes_by_name() {
    let mut state = AppState::new();
    for node in ["patch", "playback", "root"] {
        let out = run(&mut state, &format!("list {node}"));
        assert!(!out.contains("Unknown list target"), "list {node}: {out}");
    }
    let out = run(&mut state, "list nonsense");
    assert!(out.contains("Unknown list target"), "{out}");
}

#[test]
fn verb_without_object_lists_what_takes_it() {
    let mut state = AppState::new();
    let out = run(&mut state, "Copy");
    assert!(out.contains("Copy what?") && out.contains("Group"), "{out}");
}

#[test]
fn rights_and_class_follow_aliases_and_prefixes() {
    use crate::rights::authorize as authorize_command;
    use crate::commands::registry::Rights;
    assert!(
        authorize_command(Rights::Guest, "lis").is_ok(),
        "lis = List is guest"
    );
    assert!(
        authorize_command(Rights::Guest, "cls").is_ok(),
        "cls = Clear is guest"
    );
    assert!(authorize_command(Rights::Guest, "store").is_err());
    assert!(authorize_command(Rights::Program, "testask").is_err());
    assert!(authorize_command(Rights::Admin, "testinternal").is_err());
}

#[test]
fn short_forms_resolve_everywhere() {
    use super::resolve_verb;
    assert_eq!(resolve_verb("u").unwrap(), "update");
    assert_eq!(resolve_verb("hi").unwrap(), "highlight");
    assert_eq!(resolve_verb("ft").unwrap(), "fixturetype");
    assert_eq!(resolve_verb("fix").unwrap(), "fixture");
    assert_eq!(resolve_verb("logo").unwrap(), "logout");
}

#[test]
fn errors_carry_codes_that_info_error_explains() {
    let mut state = AppState::new();
    let out = run(&mut state, "frobnicate");
    assert!(out.contains("[E-CMD-001]"), "{out}");
    let info = run(&mut state, "info error e-cmd-1");
    assert!(info.contains("Unknown word"), "{info}");
    let list = run(&mut state, "list error patch");
    assert!(
        list.contains("E-PATCH-002") && !list.contains("E-CMD-001"),
        "{list}"
    );
}

#[test]
fn set_writes_declared_properties_and_refuses_the_rest() {
    // A Live Patch edit (no claim): live and undoable.
    let mut state = AppState::new();
    let mut ev = Vec::new();
    for _ in 0..2 {
        lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    }
    let out = run(&mut state, "set universe 2 \"name\" \"Stage Left\"");
    assert!(out.contains("Stage Left"), "{out}");
    let name = state.root.show_data.devices.dmx.patch.universes[&2]
        .meta
        .identity.name
        .clone();
    assert_eq!(name.as_deref(), Some("Stage Left"));
    // Undoable through the change path.
    run(&mut state, "oops");
    assert_ne!(
        state.root.show_data.devices.dmx.patch.universes[&2]
            .meta
            .name(),
        Some("Stage Left")
    );

    let out = run(&mut state, "set universe 2 \"guid\" 123");
    assert!(out.contains("[E-SET-002]"), "{out}");
    let out = run(&mut state, "set universe 2 \"frobs\" 1");
    assert!(out.contains("[E-SET-001]"), "{out}");
    let out = run(&mut state, "set universe 9 \"name\" \"x\"");
    assert!(out.contains("[E-CMD-007]"), "{out}");
    let out = run(&mut state, "set universe 2 \"locked\" \"maybe\"");
    assert!(out.contains("[E-SYN-014]"), "{out}");
    let out = run(
        &mut state,
        "set root.showdata.devices.dmx.patch.universes.2 \"hidden\" on",
    );
    assert!(
        state.root.show_data.devices.dmx.patch.universes[&2].meta.annotation.hidden,
        "{out}"
    );
}

/// `Root.Drives` is entered only by tree-shell words or a path that names
/// it (decisions §29, finding 5): Show-reach lines never touch the disk.
#[test]
fn show_lines_stay_off_the_disk() {
    use crate::commands::registry::{Reach, line_reach};
    let mut state = AppState::new();
    for line in ["quicksheet", "List", "List Root", "Watch Show", "Info"] {
        if line_reach(line) != Reach::Show {
            continue;
        }
        lumacore_luma::drive::set_off(true);
        let _ = dispatch(&mut state, line);
        lumacore_luma::drive::set_off(false);
        let refused = lumacore_luma::drive::take_refused();
        assert!(refused.is_empty(), "{line} reached the disk: {refused:?}");
    }
    assert_eq!(line_reach("List Drives"), Reach::Drive);
    assert_eq!(line_reach("List Root.Drives.Home"), Reach::Drive);
}

