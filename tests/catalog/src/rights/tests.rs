// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Every line's level through the declarations, the gate at each level and the help (moved from
//! the kernel's rights tests: the table names lines of every family).
//!
//! Owns:          the rights table (LINES).
//! Doesn't own:   the gate (the kernel's rights), each word's level (its row).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d hardening", §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::dispatch;
use lumacore_objects::commands::registry::Rights;
use lumacore_objects::rights::{check_statement, needed};
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tree::UserRights;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// A show with one user per level, each named after it.
fn ladder() -> AppState {
    let mut state = AppState::new();
    for r in UserRights::ALL.iter().filter(|r| **r != UserRights::Admin) {
        let out = run(
            &mut state,
            &format!("Store User \"{0}\" /Rights {0}", r.label()),
        );
        assert!(!out.contains("Error"), "{out}");
    }
    state
}

fn words(line: &str) -> Vec<String> {
    lumacore_objects::commands::tokens::words(
        &lumacore_objects::commands::tokens::lex(line).unwrap(),
    )
}

/// The level each line needs (what the gate reads from the declarations).
const LINES: &[(&str, Rights)] = &[
    // Anyone: look, the device, their own look and password.
    ("List Fixture", Rights::Guest),
    ("Info Show", Rights::Guest),
    ("DeskLock", Rights::Guest),
    ("Toggle FullScreen", Rights::Guest),
    ("On NormalTheme", Rights::Guest),
    ("Sidebars", Rights::Guest),
    ("View 1", Rights::Guest),
    ("Set UserProfile Theme \"Base\" Night", Rights::Guest),
    ("Set User \"Guest\" \"Password\" \"x\"", Rights::Guest),
    ("Set UserProfile \"ClearBuffer\" On", Rights::Guest),
    ("Oops", Rights::Guest),
    // Playback: run the show, use the programmer, arrange their screen.
    ("Fixture 1", Rights::Playback),
    ("At 50", Rights::Playback),
    ("Go", Rights::Playback),
    ("On Blind", Rights::Playback),
    ("Group 1", Rights::Playback),
    ("Store View 3", Rights::Playback),
    ("Set Window 1 \"Title\" \"x\"", Rights::Playback),
    ("Screenshot", Rights::Playback),
    // Presets: update existing presets, save the same show.
    ("Update Preset 4.1", Rights::Presets),
    ("Store Preset 4.1 /Merge", Rights::Presets),
    ("SaveShow", Rights::Presets),
    // Program: build the show, the live patch.
    ("Store Group 1", Rights::Program),
    ("Store Preset 4.1", Rights::Program),
    ("Store Cue 1", Rights::Program),
    ("Label Fixture 1 \"Spot\"", Rights::Program),
    ("Set Fixture 1 \"Name\" \"Spot\"", Rights::Program),
    ("Claim Fixture 1", Rights::Program),
    ("Park Fixture 1", Rights::Program),
    ("SaveShow \"Other\"", Rights::Program),
    ("Claim DataPools", Rights::Program),
    // Setup: the rig, the station, the LumaSession.
    ("Claim Root.ShowData.Devices", Rights::Setup),
    ("Store Fixture 9 At DMX 1.001 /FixtureType 1", Rights::Setup),
    ("Assign Fixture 1 At DMX 1.001", Rights::Setup),
    ("Delete Fixture 1", Rights::Setup),
    ("Move Fixture 1 At 10", Rights::Setup),
    ("Store Sacn 1", Rights::Setup),
    ("Set Sacn 1 \"Rate\" 40", Rights::Setup),
    ("Set Sacn 1 \"Name\" \"Rig\"", Rights::Setup),
    ("Assign Universe 1 At Sacn 1 /Universe 1", Rights::Setup),
    ("Assign Sacn 1 At Universe 1", Rights::Setup),
    ("Unassign Sacn 1", Rights::Setup),
    ("Store Network \"Rig\"", Rights::Setup),
    // Identity: declared per setting (decisions §30 "Remote station edits").
    ("Set Station \"Name\" \"FOH\"", Rights::Admin),
    ("Set Station \"Online\" Off /NoConfirmation", Rights::Admin),
    ("Set Station \"Location\" \"Booth\"", Rights::Setup),
    // Naming a station: that station checks (this one's own rows in the handler).
    ("Set Station \"Tower\" \"Priority\" High", Rights::Guest),
    ("Set Interface \"en1\" \"HTTP\" No", Rights::Setup),
    ("Join LumaSession \"Festival\"", Rights::Setup),
    ("Leave LumaSession", Rights::Setup),
    ("Claim LumaSession", Rights::Setup),
    ("Delete Station \"FOH-2\"", Rights::Setup),
    ("Set Show \"MergeDefault\" Merge", Rights::Setup),
    ("Set Show \"Autosave\" 5", Rights::Setup),
    ("LoadShow \"demo\"", Rights::Setup),
    ("Shutdown", Rights::Setup),
    ("Restart", Rights::Setup),
    ("Clear Programmer User 2", Rights::Setup),
    ("On DeskLock Station", Rights::Setup),
    // Admin: users and profiles, /Force, NewShow, CleanStart, Debug.
    ("Store User \"X\"", Rights::Admin),
    ("Delete User \"X\"", Rights::Admin),
    ("Set User \"Guest\" \"Rights\" Setup", Rights::Admin),
    ("Set User \"Guest\" \"Name\" \"Y\"", Rights::Admin),
    ("Store UserProfile \"Y\"", Rights::Admin),
    ("Import UserProfile \"Y\"", Rights::Admin),
    ("Claim Group 1 /Force", Rights::Admin),
    ("NewShow \"x\"", Rights::Admin),
    ("CleanStart", Rights::Admin),
    ("Debug", Rights::Admin),
    ("Set Osc 1 \"User\" \"Admin\"", Rights::Admin),
    // Never a user's.
    ("HissMe", Rights::Internal),
];

#[test]
fn every_line_needs_its_declared_level() {
    for (line, level) in LINES {
        let w = words(line);
        let args: Vec<&str> = w[1..].iter().map(String::as_str).collect();
        assert_eq!(needed(&w[0], &args).0, *level, "{line}");
    }
}

#[test]
fn the_gate_passes_each_level_up_to_its_own() {
    let mut state = ladder();
    for user in UserRights::ALL {
        state.user = user.label().into();
        for (line, level) in LINES {
            let w = words(line);
            let args: Vec<&str> = w[1..].iter().map(String::as_str).collect();
            let got = check_statement(&state, &w[0], &args);
            assert_eq!(
                got.is_ok(),
                Rights::from(user) >= *level,
                "{} running {line}: {got:?}",
                user.label()
            );
            if let Err(e) = got
                && *level != Rights::Internal
            {
                assert!(e.contains("E-RIGHTS-001"), "{e}");
                assert!(
                    e.contains(&format!(
                        "needs {} — you are {}",
                        level.label(),
                        user.label()
                    )),
                    "{e}"
                );
            }
        }
    }
}

#[test]
fn help_names_the_levels() {
    let mut state = ladder();
    assert!(run(&mut state, "Help CleanStart").contains("Needs:  Admin"));
    let out = run(&mut state, "Help Sacn");
    assert!(out.contains("Guest: ") && out.contains("Setup: "), "{out}");
}
