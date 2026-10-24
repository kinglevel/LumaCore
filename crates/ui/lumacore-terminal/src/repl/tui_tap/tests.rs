// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The tag's tap rules on the rig (decisions 0297): Assign + a tag's square arms the line, a pool
//! cell finishes it and it comes back armed, another square switches the tag, an empty line runs
//! the square's action, and the `armed_tag` event follows.
//!
//! Owns:          the tests of tui_tap.rs.
//! Doesn't own:   what an armed line is (lumacore-cmd-tags' own tests).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0297-tag-squares-and-picker.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn tap(object: &str, no: u32, id: Option<&str>, action: Option<&str>) -> crate::Tap {
    crate::Tap { object: object.into(), no, id: id.map(str::to_string), hold: false, action: action.map(str::to_string), window: None }
}

fn armed(events: &[UiEvent]) -> Option<(String, usize)> {
    events.iter().rev().find_map(|e| match e {
        UiEvent::ArmedTag { tag, carriers, .. } => Some((tag.clone(), carriers.len())),
        _ => None,
    })
}

#[test]
fn a_tags_square_arms_assign_and_the_line_stays_for_the_next_cell() {
    let mut state = crate::core::demo::fresh_state();
    state.user = "Admin".into();
    let mut app = TuiApp::new(&state);
    app.run_ui_command(&mut state, "Store Tag 1 \"FOH\"");
    app.run_ui_command(&mut state, "Store Tag 2 \"Stage\"");
    app.take_ui_events(&mut state);

    app.set_input("Assign ");
    app.handle_tap(&mut state, &tap("Tag", 0, Some("1"), None));
    assert_eq!(app.get_input(), "Assign Tag 1 At ");
    assert_eq!(armed(&app.take_ui_events(&mut state)), Some(("1".into(), 0)));

    app.handle_tap(&mut state, &tap("Group", 3, None, None));
    assert_eq!(app.get_input(), "Assign Tag 1 At ", "armed again for the next cell");
    assert_eq!(armed(&app.take_ui_events(&mut state)), Some(("1".into(), 1)), "Group 3 carries it now");
    app.handle_tap(&mut state, &tap("Sequence", 1, None, None));
    assert_eq!(armed(&app.take_ui_events(&mut state)), Some(("1".into(), 2)));

    app.handle_tap(&mut state, &tap("Tag", 0, Some("2"), None));
    assert_eq!(app.get_input(), "Assign Tag 2 At ", "another square switches the tag");
    app.set_input("");
    assert_eq!(armed(&app.take_ui_events(&mut state)), Some((String::new(), 0)), "no longer armed");
    assert_eq!(armed(&app.take_ui_events(&mut state)), None, "told once");

    // Unassign arms the same way.
    app.set_input("Unassign");
    app.handle_tap(&mut state, &tap("Tag", 0, Some("1"), None));
    app.handle_tap(&mut state, &tap("Group", 3, None, None));
    assert_eq!(armed(&app.take_ui_events(&mut state)), Some(("1".into(), 1)));
}

#[test]
fn an_empty_line_runs_the_squares_action_and_another_verb_takes_the_tag() {
    let mut state = crate::core::demo::fresh_state();
    state.user = "Admin".into();
    let mut app = TuiApp::new(&state);
    app.run_ui_command(&mut state, "Store Tag 1 \"FOH\"");
    app.handle_tap(&mut state, &tap("Tag", 0, Some("1"), Some("Set Tag 1 \"Note\" \"tapped\"")));
    assert_eq!(app.get_input(), "");
    let note = state.root.show_data.tags.get(&1).and_then(|t| t.meta.annotation.note.clone());
    assert_eq!(note.as_deref(), Some("tapped"), "the action ran");
    app.set_input("Move ");
    app.handle_tap(&mut state, &tap("Tag", 0, Some("1"), None));
    assert_eq!(app.get_input(), "Move Tag 1", "another verb takes the tag by its path");
}

/// Esc only closes (the user's rule, decisions 0297): an armed line stays through it.
#[test]
fn esc_never_clears_the_line() {
    let mut state = crate::core::demo::fresh_state();
    state.user = "Admin".into();
    let mut app = TuiApp::new(&state);
    app.set_input("Assign Tag 1 At ");
    app.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Esc, crossterm::event::KeyModifiers::NONE), &mut state);
    app.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Esc, crossterm::event::KeyModifiers::NONE), &mut state);
    assert_eq!(app.get_input(), "Assign Tag 1 At ");
}

