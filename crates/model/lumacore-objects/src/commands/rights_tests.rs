//! Rights levels (decisions §30 "12d hardening"): the ladder, one level per
//! user, every check declared — through the real gate and dispatch.

use super::dispatch;
use crate::{AppState, DispatchResult};
use lumacore_tree::UserRights;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}


/// A show with one user per level, each named after it.
fn ladder() -> AppState {
    let mut state = AppState::new();
    for r in UserRights::ALL.iter().filter(|r| **r != UserRights::Admin) {
        let out = run(&mut state, &format!("Store User \"{0}\" /Rights {0}", r.label()));
        assert!(!out.contains("Error"), "{out}");
    }
    state
}

#[test]
fn a_guest_keeps_their_own_device_password_colour_and_look() {
    let mut state = ladder();
    state.user = "Guest".into();
    for line in [
        "Set User \"Guest\" \"Password\" \"pw\"",
        "Set User \"Guest\" \"Paint\" \"Teal\"",
        "Set UserProfile Theme \"Base\" Night",
        "Oops",
    ] {
        let out = run(&mut state, line);
        assert!(!out.contains("E-RIGHTS"), "{line}: {out}");
    }
    // Another user's password and colour are Admin's (an ownership rule).
    let out = run(&mut state, "Set User \"Program\" \"Paint\" \"Teal\"");
    assert!(out.contains("E-RIGHTS-001") && out.contains("needs Admin — you are Guest"), "{out}");
    // No programmer for a Guest.
    let out = run(&mut state, "Fixture 1");
    assert!(out.contains("needs Playback — you are Guest"), "{out}");
}

#[test]
fn only_admin_grants_levels_and_admin_stays_admin() {
    let mut state = ladder();
    state.user = "Setup".into();
    let out = run(&mut state, "Set User \"Guest\" \"Rights\" Program");
    assert!(out.contains("E-RIGHTS-001"), "{out}");
    state.user = "Admin".into();
    let out = run(&mut state, "Set User \"Guest\" \"Rights\" Playback");
    assert!(out.contains("rights Playback"), "{out}");
    assert_eq!(state.root.show_data.rights_of("Guest"), Some(UserRights::Playback));
    assert!(run(&mut state, "Set User \"Admin\" \"Rights\" Setup").contains("E-USER-005"));
    assert!(run(&mut state, "Set User \"Guest\" \"Rights\" \"Boss\"").contains("E-SET-003"));
}

#[test]
fn a_semicolon_line_is_refused_whole() {
    let mut state = ladder();
    state.user = "Program".into();
    let before = state.root.show_data.users.len();
    let out = run(&mut state, "Store Group 7; Store User \"Z\"");
    assert!(out.contains("E-RIGHTS-001") && out.contains("nothing in"), "{out}");
    assert_eq!(state.root.show_data.users.len(), before);
}

/// A quoted `"/Force"` is a value (§15 Quoting), never Admin's flag — in a line of several
/// statements too (it was a flag there: that path read unquoted words; decisions §31).
#[test]
fn a_quoted_force_never_raises_the_level() {
    let mut state = ladder();
    state.switch_to_existing_user("Program").expect("the Program user");
    let single = run(&mut state, "Claim Group 1 \"/Force\"");
    assert!(!single.contains("E-RIGHTS-001"), "{single}");
    let several = run(&mut state, "List Group; Claim Group 1 \"/Force\"");
    assert!(!several.contains("E-RIGHTS-001"), "{several}");
    // The bare flag still needs Admin, in either shape.
    let bare = run(&mut state, "List Group; Claim Group 1 /Force");
    assert!(bare.contains("E-RIGHTS-001") && bare.contains("nothing in"), "{bare}");
}
