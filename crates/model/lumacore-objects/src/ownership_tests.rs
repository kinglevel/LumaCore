//! Ownership (decisions §4, §10, §21 "6d hardening"): editors claim, others'
//! claims block plain commands, taking one asks its owner (10 s), Admins
//! force, claims follow the object (GUID), `cd Claims` + `List`.

use crate::AppState;
use crate::commands::dispatch;
use crate::DispatchResult;
use crate::claims::Claims;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => String::new(),
    }
}

fn as_user(state: &mut AppState, user: &str) {
    state.acting_user = Some(user.to_string());
    state.root.runtime.user(user);
}

/// Admin and Busk (a Programmer) logged in; Group 1 and 2 stored.
fn two_users() -> AppState {
    let mut state = AppState::new();
    run(&mut state, "Store User \"Busk\"");
    let fixture = lumacore_fixtures::Fixture::new(
        None,
        uuid::Uuid::new_v4(),
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        1,
        1,
    );
    state.root.show_data.devices.dmx.patch.insert_fid(1, fixture);
    as_user(&mut state, "Admin");
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 1 \"A\"");
    run(&mut state, "Store Group 2 \"B\"");
    run(&mut state, "Clear");
    as_user(&mut state, "Busk");
    as_user(&mut state, "Admin");
    state
}

#[test]
fn ownership_an_editor_claims_and_others_are_refused() {
    let mut state = two_users();
    as_user(&mut state, "Busk");
    run(&mut state, "Edit Group 1");
    assert!(state.is_claimed_by_me("Group 1"), "opening the editor claims it");
    // Each Shell has its own context: Admin isn't in Busk's editor.
    let busk_context = std::mem::replace(&mut state.context, crate::ReplContext::Fixture);
    as_user(&mut state, "Admin");
    let out = run(&mut state, "Label Group 1 \"Mine now\"");
    assert!(out.contains("E-CLAIM-001") && out.contains("Busk"), "{out}");
    assert!(run(&mut state, "Edit Group 1").contains("claimed by Busk"));
    // Group 2 is free.
    assert!(!run(&mut state, "Label Group 2 \"Free\"").contains("E-CLAIM"));
    // Busk closes the editor: the claim goes.
    as_user(&mut state, "Busk");
    state.context = busk_context;
    // `..` closes the editor (as `Exit`, the tree shell's word).
    run(&mut state, "..");
    assert!(!state.is_claimed("Group 1"));
}

#[test]
fn ownership_a_claim_follows_its_object_when_it_moves() {
    let mut state = two_users();
    as_user(&mut state, "Busk");
    run(&mut state, "Claim Group 2");
    as_user(&mut state, "Admin");
    run(&mut state, "Move Group 2 At 8");
    // Moving is refused while Busk holds it; once released and moved, a new
    // claim names the new number.
    assert!(state.root.show_data.datapools[&1].groups.contains_key(&2));
    as_user(&mut state, "Busk");
    assert!(state.claims_of("Busk").contains(&"Group 2".to_string()));
}

#[test]
fn ownership_taking_a_claim_asks_the_owner() {
    let mut state = two_users();
    as_user(&mut state, "Busk");
    run(&mut state, "Claim Group 1");
    as_user(&mut state, "Admin");
    let out = run(&mut state, "Claim Group 1");
    assert!(out.contains("asked them"), "{out}");
    assert_eq!(state.root.runtime.claim_asks.len(), 1);
    // Busk keeps it.
    as_user(&mut state, "Busk");
    assert!(run(&mut state, "Claim Group 1 /Keep").contains("You keep Group 1"));
    assert!(state.root.runtime.claim_asks.is_empty());
    assert_eq!(state.claim_holder("Group 1"), Some("Busk"));
    // Asked again, Busk hands it over.
    as_user(&mut state, "Admin");
    run(&mut state, "Claim Group 1");
    as_user(&mut state, "Busk");
    run(&mut state, "Unclaim Group 1");
    assert_eq!(state.claim_holder("Group 1"), Some("Admin"));
}

#[test]
fn ownership_no_answer_hands_over_and_admins_force() {
    let mut state = two_users();
    as_user(&mut state, "Busk");
    run(&mut state, "Claim Group 1");
    as_user(&mut state, "Admin");
    run(&mut state, "Claim Group 1");
    // A wall-clock step doesn't hand it over: the session clock decides.
    state.root.runtime.claim_asks[0].deadline_ms = 0;
    assert!(!state.expire_claim_asks());
    assert_eq!(state.claim_holder("Group 1"), Some("Busk"));
    // Ten seconds pass without an answer.
    state.root.runtime.claim_asks[0].due_session_ms = 0;
    assert!(state.expire_claim_asks());
    assert_eq!(state.claim_holder("Group 1"), Some("Admin"));
    // An Admin takes Group 2 from Busk at once.
    as_user(&mut state, "Busk");
    run(&mut state, "Claim Group 2");
    as_user(&mut state, "Admin");
    assert!(run(&mut state, "Claim Group 2 /Force").contains("/Force"));
    assert_eq!(state.claim_holder("Group 2"), Some("Admin"));
    // A Programmer can't force.
    as_user(&mut state, "Busk");
    assert!(run(&mut state, "Claim Group 2 /Force").contains("E-RIGHTS-001"));
}

#[test]
fn ownership_an_owner_that_is_logged_out_hands_over_at_once() {
    let mut state = two_users();
    as_user(&mut state, "Busk");
    run(&mut state, "Claim Group 1");
    state.root.runtime.users.remove("Busk");
    as_user(&mut state, "Admin");
    assert!(run(&mut state, "Claim Group 1").contains("isn't logged in"));
    assert_eq!(state.claim_holder("Group 1"), Some("Admin"));
}

#[test]
fn ownership_cd_never_claims_and_claims_is_a_place() {
    let mut state = two_users();
    state.navigation_path = crate::navigation::Navigation::resolve_target(&state, "Root.ShowData.Devices.Dmx.Patch").expect("cd");
    assert!(!state.is_claimed("Devices"), "cd only navigates");
    run(&mut state, "Claim Group 1");
    state.navigation_path = crate::navigation::Navigation::resolve_target(&state, "Claims").expect("cd");
    let out = run(&mut state, "List");
    assert!(out.contains("Group 1"), "{out}");
    assert!(run(&mut state, "List Claims").contains("Group 1"));
}

/// R1-44: the claim check is on the outcome — whatever shape the line has
/// (Thru, a Copy target, a Move, an owner-first path), a line that
/// would change another user's claimed object is rolled back.
#[test]
fn ownership_every_line_shape_is_refused_on_a_claimed_object() {
    let mut state = two_users();
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 3 \"C\"");
    run(&mut state, "Clear");
    // Group 2 differs from Group 3, so copying it over Group 3 changes it.
    state.root.show_data.datapools.get_mut(&1).unwrap().groups.get_mut(&2).unwrap().members.clear();
    as_user(&mut state, "Busk");
    assert!(run(&mut state, "Claim Group 3").contains("Claimed"), "Busk claims Group 3");
    as_user(&mut state, "Admin");
    let groups = |s: &AppState| serde_json::to_string(&s.root.show_data.datapools[&1].groups).unwrap();
    let before = groups(&state);
    for line in [
        "Delete Group 1 Thru 5 /nc",
        "Copy Group 2 At 3 /Overwrite",
        "Move Group 3 At 9",
        "Label Group 3 \"x\"",
        "Set Group 3 \"Mode\" Scaling",
    ] {
        let out = crate::text::strip_ansi(&run(&mut state, line));
        assert!(out.contains("E-CLAIM-001") && out.contains("Busk"), "'{line}': {out}");
        assert_eq!(groups(&state), before, "'{line}' changed the groups");
    }
    // Lines that leave Group 3 alone still run.
    assert!(!run(&mut state, "Label Group 2 \"Free\"").contains("Error"));
    // Busk changes their own claimed object.
    as_user(&mut state, "Busk");
    assert!(!run(&mut state, "Label Group 3 \"Mine\"").contains("Error"));
}


#[test]
fn ownership_oops_on_a_claimed_object_keeps_the_step() {
    let mut state = two_users();
    run(&mut state, "Label Group 1 \"Renamed\"");
    as_user(&mut state, "Busk");
    assert!(run(&mut state, "Claim Group 1").contains("Claimed"), "Busk claims Group 1");
    as_user(&mut state, "Admin");
    let steps = |s: &AppState| s.undo_stack("Admin").map_or(0, |u| u.frames.len());
    let before = steps(&state);
    let out = crate::text::strip_ansi(&run(&mut state, "Oops /nc"));
    assert!(out.contains("E-CLAIM-001") && out.contains("Busk"), "{out}");
    assert_eq!(steps(&state), before, "the refused Oops keeps the step");
    assert_eq!(state.root.show_data.datapools[&1].groups[&1].meta.identity.name.as_deref(), Some("Renamed"));
    // Once the claim is gone the same step undoes.
    as_user(&mut state, "Busk");
    run(&mut state, "Unclaim Group 1");
    as_user(&mut state, "Admin");
    run(&mut state, "Oops /nc");
    assert_eq!(steps(&state), before - 1);
}
