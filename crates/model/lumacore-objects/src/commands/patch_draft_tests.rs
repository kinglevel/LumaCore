//! The staged patch (decisions §11 "Menus, LumaSession and staged patch"):
//! claiming Patch edits a draft, the rig keeps the live patch, leaving asks
//! keep / discard, others' patch edits wait — through real dispatch.

use super::dispatch;
use crate::AppState;
use crate::patch_draft::live_view;
use crate::DispatchResult;
use crate::claims::Claims;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// The question a line asks (decisions §30 "12b").
fn ask(state: &mut AppState, line: &str) -> crate::dialogs::Ask {
    match dispatch(state, line) {
        DispatchResult::Ask(ask) => *ask,
        _ => panic!("'{line}' must ask"),
    }
}

fn live_universes(state: &AppState) -> Vec<u32> {
    live_view(state, state.root.show_data.clone())
        .devices
        .dmx
        .patch
        .universes
        .keys()
        .copied()
        .collect()
}

fn undo_depth(state: &AppState) -> usize {
    state.undo_stack("Admin").map_or(0, |u| u.len())
}

#[test]
fn patch_draft_keeps_the_rig_on_the_live_patch_until_kept() {
    let mut state = AppState::new();
    run(&mut state, "Claim Root.ShowData.Devices");
    let depth = undo_depth(&state);
    assert!(run(&mut state, "Store Universe 7").contains("Created Universe 7"));
    // The draft has it; the rig (published view) doesn't, and no undo step yet.
    assert!(state.root.show_data.devices.dmx.patch.universes.contains_key(&7));
    assert!(!live_universes(&state).contains(&7));
    assert_eq!(undo_depth(&state), depth);
    // Leaving asks Keep / Discard / Cancel; Cancel runs nothing.
    let leaving = ask(&mut state, "Unclaim Root.ShowData.Devices");
    assert_eq!(leaving.title(), "Leave the draft");
    assert_eq!(leaving.line_of("Cancel"), None);
    assert!(state.is_claimed_by_me("Devices"));
    // Navigating away is refused while the draft has changes.
    crate::commands::navigate::cmd_enter(&mut state, &["patch"]);
    assert!(crate::commands::navigate::cmd_enter(&mut state, &[".."]).contains("draft has changes"));
    // Keep: one undo step, the rig gets it, Patch is released.
    run(&mut state, &leaving.line_of("Save").unwrap());
    assert!(state.patch_draft.is_none() && !state.is_claimed("Devices"));
    assert!(live_universes(&state).contains(&7));
    assert_eq!(undo_depth(&state), depth + 1);
    run(&mut state, "Oops");
    assert!(!state.root.show_data.devices.dmx.patch.universes.contains_key(&7));
}

#[test]
fn patch_draft_discard_puts_the_live_patch_back() {
    let mut state = AppState::new();
    run(&mut state, "Claim Root.ShowData.Devices");
    run(&mut state, "Store Universe 3");
    let discard = ask(&mut state, "Unclaim Root.ShowData.Devices").line_of("Discard").unwrap();
    assert_eq!(discard, "Unclaim Root.ShowData.Devices /Discard");
    run(&mut state, &discard);
    assert!(!state.root.show_data.devices.dmx.patch.universes.contains_key(&3));
    assert!(state.patch_draft.is_none() && !state.is_claimed("Devices"));
    // `/Keep` answers without asking (scripts, one-shot remote commands).
    run(&mut state, "Claim Root.ShowData.Devices");
    run(&mut state, "Store Universe 4");
    assert!(run(&mut state, "Unclaim Root.ShowData.Devices /Keep").contains("kept"));
    assert!(state.root.show_data.devices.dmx.patch.universes.contains_key(&4));
    // An unchanged draft just ends, without asking.
    run(&mut state, "Claim Root.ShowData.Devices");
    assert!(run(&mut state, "Unclaim Root.ShowData.Devices").contains("released"));
}

/// The ✕ while claimed (decisions §34 "34g4 patch entry hardened" 3): `Unclaim …; Menu "Patch"`
/// asks first and every answer closes the dialog after it; a lost holder's draft goes (5).
#[test]
fn closing_asks_and_a_lost_holder_loses_the_draft() {
    let mut state = AppState::new();
    run(&mut state, "Claim Root.ShowData.Devices");
    run(&mut state, "Store Universe 7");
    let close = ask(&mut state, "Unclaim Root.ShowData.Devices; Menu \"Patch\"");
    assert_eq!(close.line_of("Save").as_deref(), Some("Unclaim Root.ShowData.Devices /Keep; Menu \"Patch\""));
    assert_eq!(close.line_of("Discard").as_deref(), Some("Unclaim Root.ShowData.Devices /Discard; Menu \"Patch\""));
    // Still present: nothing happens.
    let me = state.effective_user_name().to_string();
    assert_eq!(state.release_lost_devices(std::slice::from_ref(&me)), None);
    assert!(state.patch_draft.is_some());
    // Gone: the claim is released and the draft discarded, nothing kept.
    assert_eq!(state.release_lost_devices(&[]), Some(me));
    assert!(state.patch_draft.is_none() && !state.is_claimed("Devices"));
    assert!(!state.root.show_data.devices.dmx.patch.universes.contains_key(&7));
    assert!(state.sysmon_pending.iter().any(|(_, m)| m.contains("left the session with the patch claimed: the draft was discarded")));
}

/// R1 review (fce792a): Keep is refused when the draft changes another
/// user's claimed fixture; the draft stays open and nothing is kept.
#[test]
fn patch_draft_keep_respects_claims() {
    let mut state = super::playback_tests::rig(&[1, 2]);
    run(&mut state, "Store User \"Busk\"");
    let as_user = |state: &mut AppState, user: &str| {
        state.acting_user = Some(user.to_string());
        state.root.runtime.user(user);
    };
    as_user(&mut state, "Busk");
    assert!(run(&mut state, "Claim Fixture 1").contains("Claimed"));
    as_user(&mut state, "Admin");
    run(&mut state, "Claim Root.ShowData.Devices");
    // The draft loses Fixture 1 (however it got there).
    state.root.show_data.devices.dmx.patch.remove_fid(1);
    let out = run(&mut state, "Unclaim Root.ShowData.Devices /Keep");
    assert!(out.contains("E-CLAIM-001") && out.contains("Busk"), "{out}");
    assert!(state.patch_draft.is_some(), "the draft stays open");
    assert!(!state.root.show_data.devices.dmx.patch.has_fid(1), "as drafted");
    assert!(live_view(&state, state.root.show_data.clone()).devices.dmx.patch.has_fid(1), "the live patch kept it");
}

/// 34g4 part 2 (decisions §34 "34g4 patch hardened" 8): while Admin holds the Devices draft,
/// another user's live column reaches the rig and the draft (one Oops step of their own); a
/// cell the holder changed waits, a column that isn't live is refused.
#[test]
fn live_columns_reach_the_rig_and_the_draft() {
    let mut state = super::playback_tests::rig(&[1, 2]);
    run(&mut state, "Store User \"Busk\"");
    run(&mut state, "Set User \"Busk\" \"Rights\" Setup");
    let as_user = |state: &mut AppState, user: &str| {
        state.acting_user = Some(user.to_string());
        state.root.runtime.user(user);
    };
    let name = |show: &lumacore_tree::ShowData, n: u32| show.devices.dmx.patch.fixture(n).expect("resolved").meta.identity.name.clone().unwrap_or_default();
    let (was1, was2) = (name(&state.root.show_data, 1), name(&state.root.show_data, 2));
    as_user(&mut state, "Admin");
    run(&mut state, "Claim Root.ShowData.Devices");
    run(&mut state, "Set Fixture 2 \"Name\" \"Drafted\"");
    as_user(&mut state, "Busk");
    let out = run(&mut state, "Set Fixture 1 \"Name\" \"Spot\"");
    assert!(!out.contains("E-"), "{out}");
    let live = live_view(&state, state.root.show_data.clone());
    assert_eq!(name(&live, 1), "Spot", "the rig has it");
    assert_eq!(name(&state.root.show_data, 1), "Spot", "the draft follows");
    // The draft remembers who brought that cell in (the violet dot, round 3).
    let g1 = state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.guid;
    assert_eq!(state.patch_draft.as_ref().unwrap().marks[&g1]["name"], "Busk");
    assert_eq!(name(&live, 2), was2, "the holder's cell stays in the draft");
    // The holder changed that cell: it waits, naming them.
    let out = run(&mut state, "Set Fixture 2 \"Name\" \"Wash\"");
    assert!(out.contains("E-PATCH-008") && out.contains("Admin"), "{out}");
    assert_eq!(name(&live_view(&state, state.root.show_data.clone()), 2), was2);
    // Not a live column: refused while the draft is open.
    let out = run(&mut state, "Set Fixture 1 \"FixtureID\" \"X\"");
    assert!(out.contains("E-PATCH-008"), "{out}");
    // Busk's Oops takes it back from the rig and the draft.
    run(&mut state, "Oops /NoConfirmation");
    assert_eq!(name(&live_view(&state, state.root.show_data.clone()), 1), was1);
    assert_eq!(name(&state.root.show_data, 1), was1);
    // Keep brings the holder's draft to the rig.
    as_user(&mut state, "Admin");
    run(&mut state, "Unclaim Root.ShowData.Devices /Keep");
    assert_eq!(name(&state.root.show_data, 2), "Drafted");
    assert!(state.patch_draft.is_none());
}

/// The lost-holder watch (decisions §34 "34g4 patch entry hardened" 5): the holder must be gone
/// for 5 s — a page reload that comes back in time starts it over, so does another holder.
#[test]
fn the_holder_is_lost_only_after_five_seconds_gone() {
    use crate::claims::{HOLDER_GRACE, HolderWatch};
    let t0 = std::time::Instant::now();
    let at = |ms: u64| t0 + std::time::Duration::from_millis(ms);
    let (bob, none) = (vec!["Bob".to_string()], Vec::<String>::new());
    let mut w = HolderWatch::default();
    assert!(!w.lost(Some("Bob"), &bob, at(0)), "present");
    assert!(!w.lost(Some("Bob"), &none, at(1_000)), "gone: the 5 s start");
    assert!(!w.lost(Some("Bob"), &none, at(5_999)));
    // A reload: back in time, the watch starts over.
    assert!(!w.lost(Some("bob"), &bob, at(6_000)), "names match in any case");
    assert!(!w.lost(Some("Bob"), &none, at(7_000)));
    assert!(!w.lost(Some("Bob"), &none, at(11_999)));
    assert!(w.lost(Some("Bob"), &none, at(7_000) + HOLDER_GRACE), "gone 5 s");
    // Another holder starts their own 5 s; no holder clears it.
    assert!(!w.lost(Some("Ann"), &none, at(13_000)));
    assert!(!w.lost(None, &none, at(14_000)));
    assert!(!w.lost(Some("Ann"), &none, at(18_500)), "cleared by the release in between");
}

/// Without a draft, the live columns are anyone's (Setup): a typed line changes the rig.
#[test]
fn without_a_draft_a_live_column_changes_the_rig() {
    let mut state = super::playback_tests::rig(&[1]);
    let out = run(&mut state, "Set Fixture 1 \"Name\" \"Spot SR\"");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.name.as_deref(), Some("Spot SR"));
}

/// Live locks every other column (§34 "34g4 patch hardened" 9, round 3 item 10: "Claim the draft
/// to change it") — the server side of the Live column lock (the round 3 audit).
#[test]
fn without_a_draft_other_columns_need_the_claim() {
    let mut state = super::playback_tests::rig(&[1]);
    let out = run(&mut state, "Set Fixture 1 \"FixtureID\" \"X\"");
    assert!(out.contains("E-") && out.contains("Claim Root.ShowData.Devices"), "{out}");
}

/// An address is a live column (§34 round 3, 18: Patch to… and Next free address work in Live):
/// `Assign Fixture … At DMX` needs no claim; under another's draft it reaches the rig and the
/// draft follows. A place in the MVR tree still needs the claim.
#[test]
fn an_address_is_live_assign_at_dmx_needs_no_claim() {
    let mut state = super::playback_tests::rig(&[1, 2]);
    let start = |show: &lumacore_tree::ShowData, n: u32| show.devices.dmx.patch.fixture(n).expect("resolved").mvr.breaks[0].start_address;
    let out = run(&mut state, "Assign Fixture 1 At DMX 1.200");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(start(&state.root.show_data, 1), 200);
    let out = run(&mut state, "Unassign Fixture 2 At DMX");
    assert!(!out.contains("E-"), "{out}");
    assert!(state.root.show_data.devices.dmx.patch.fixture(2).expect("resolved").mvr.breaks.is_empty());
    let out = run(&mut state, "Assign Fixture 1 At MvrGroup 1");
    assert!(out.contains("Claim Root.ShowData.Devices"), "{out}");
    // Another holds the draft: Busk's line runs live, the draft follows.
    run(&mut state, "Store User \"Busk\"");
    run(&mut state, "Set User \"Busk\" \"Rights\" Setup");
    run(&mut state, "Claim Root.ShowData.Devices");
    state.acting_user = Some("Busk".to_string());
    state.root.runtime.user("Busk");
    let out = run(&mut state, "Assign Fixture 1 At DMX 1.300");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(start(&live_view(&state, state.root.show_data.clone()), 1), 300, "the rig has it");
    assert_eq!(start(&state.root.show_data, 1), 300, "the draft follows");
}
