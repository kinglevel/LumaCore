//! The station's dialogs (decisions §30 "12b"): asks become board dialogs
//! for the asking client, are answered once, close everywhere first, and
//! time out to their default.

use crate::dialogs::*;
use crate::AppState;
use lumacore_wire::{
    ButtonKind, DialogAnswer, InputKind,
};
use crate::DispatchResult;
use crate::commands::dispatch;
use crate::claims::Claims;

fn ask(state: &mut AppState, line: &str) -> Ask {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => *a,
        _ => panic!("'{line}' must ask"),
    }
}

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// A station with Fixture 1 patched and selected (groups store from it).
fn with_fixture() -> AppState {
    let mut state = AppState::new();
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
    run(&mut state, "Fixture 1");
    state
}

fn who(user: &str, device: Option<&str>) -> Who {
    Who {
        user: user.into(),
        device: device.map(str::to_string),
        client: None,
    }
}

/// A shell on `device` opens what `line` asks.
fn open_on(state: &mut AppState, device: &str, line: &str) -> String {
    let a = ask(state, line);
    state.current_device = Some(device.to_string());
    open(state, a);
    state.current_device = None;
    for_client(state, &who("Admin", Some(device))).0.last().unwrap().id.clone()
}

#[test]
fn a_lines_ask_goes_only_to_its_device_and_the_answer_runs_once() {
    let mut state = AppState::new();
    let id = open_on(&mut state, "tab-1", "TestAsk");
    // Only that device sees it; another device or user doesn't.
    assert_eq!(for_client(&state, &who("Admin", Some("tab-1"))).0.len(), 1);
    assert!(for_client(&state, &who("Admin", Some("tab-2"))).0.is_empty());
    assert!(for_client(&state, &who("LD", Some("tab-1"))).0.is_empty());
    // The answer closes it first and hands back the line to run.
    let line = answer(&mut state, &who("Admin", Some("tab-1")), &DialogAnswer { id: id.clone(), button: 0, ..Default::default() });
    assert_eq!(line.unwrap().as_deref(), Some("TestAsk /NoConfirmation"));
    assert!(for_client(&state, &who("Admin", Some("tab-1"))).0.is_empty());
    // A second answer finds it gone; the closed notice says who.
    assert!(answer(&mut state, &who("Admin", Some("tab-1")), &DialogAnswer { id, button: 0, ..Default::default() }).is_err());
    let closed = for_client(&state, &who("Admin", Some("tab-1"))).1;
    assert_eq!(closed[0].by, "Admin");
    assert_eq!(closed[0].answer, "Toggle");
}

#[test]
fn cancel_runs_nothing_and_a_one_shot_path_is_told_the_flag() {
    let mut state = AppState::new();
    let a = ask(&mut state, "TestAsk");
    assert!(a.refusal().contains("/NoConfirmation") && a.refusal().contains("E-CMD-010"));
    let id = open_on(&mut state, "tab-1", "TestAsk");
    let cancel = for_client(&state, &who("Admin", Some("tab-1"))).0[0].cancel.unwrap();
    let line = answer(&mut state, &who("Admin", Some("tab-1")), &DialogAnswer { id, button: cancel, ..Default::default() });
    assert_eq!(line.unwrap(), None);
    assert!(!state.debug_mode);
}

#[test]
fn a_countdown_presses_the_default_and_a_touch_stops_it() {
    let mut state = AppState::new();
    let a = Ask::new("Shutdown", "Shut down?")
        .button("Shutdown", ButtonKind::Primary, "Shutdown /NoConfirmation")
        .cancel("Cancel")
        .countdown(0, 0);
    state.current_device = Some("tab-1".into());
    open(&mut state, a.clone());
    let due = take_due(&mut state);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].line.as_deref(), Some("Shutdown /NoConfirmation"));
    assert_eq!(for_client(&state, &who("Admin", Some("tab-1"))).1[0].by, "time");
    // Paused: it waits.
    open(&mut state, a);
    let id = for_client(&state, &who("Admin", Some("tab-1"))).0[0].id.clone();
    pause(&mut state, &who("Admin", Some("tab-1")), &id);
    assert!(take_due(&mut state).is_empty());
    assert!(for_client(&state, &who("Admin", Some("tab-1"))).0[0].countdown.as_ref().unwrap().paused);
}

#[test]
fn a_dialog_whose_client_is_gone_is_dropped_later_not_at_once() {
    let mut state = AppState::new();
    open_on(&mut state, "tab-1", "TestAsk");
    forget_gone(&mut state, |_| false);
    assert_eq!(for_client(&state, &who("Admin", Some("tab-1"))).0.len(), 1, "a reload finds it again");
}

#[test]
fn inputs_fill_the_template_and_a_bad_number_keeps_it_open() {
    let mut state = with_fixture();
    run(&mut state, "Store Group 4");
    // Label without a name asks; the empty answer clears with /Clear.
    let a = ask(&mut state, "Label Group 4");
    assert_eq!(a.view().inputs[0].name, "name");
    state.current_device = Some("tab-1".into());
    open(&mut state, a);
    let id = for_client(&state, &who("Admin", Some("tab-1"))).0[0].id.clone();
    let mut named = DialogAnswer { id: id.clone(), button: 0, ..Default::default() };
    named.inputs.insert("name".into(), "Front \"warm\"; Oops".into());
    let line = answer(&mut state, &who("Admin", Some("tab-1")), &named).unwrap().unwrap();
    assert_eq!(line, "Label Group 4 \"Front warm Oops\"", "quotes and statement breaks never get through");
    assert!(run(&mut state, &line).contains("labeled"));
    let a = ask(&mut state, "Label Group 4");
    assert_eq!(a.view().inputs[0].value, "Front warm Oops", "the current name is the start value");
    assert_eq!(a.line_of("Please").as_deref(), Some("Label Group 4 \"Front warm Oops\""));
    let mut empty = DialogAnswer { button: 0, ..Default::default() };
    empty.inputs.insert("name".into(), String::new());
    assert_eq!(a.line_for(0, &empty).unwrap().as_deref(), Some("Label Group 4 /Clear"));
    assert!(run(&mut state, "Label Group 4 /Clear").contains("cleared"));
    // Move without At asks a number; letters are refused.
    let m = ask(&mut state, "Move Group 4");
    let mut to = DialogAnswer { button: 0, ..Default::default() };
    to.inputs.insert("to".into(), "7; Delete Group 1".into());
    assert!(m.line_for(0, &to).unwrap_err().contains("E-CMD-008"));
    to.inputs.insert("to".into(), "7".into());
    assert_eq!(m.line_for(0, &to).unwrap().as_deref(), Some("Move Group 4 At 7"));
}

#[test]
fn store_over_a_used_group_and_a_password_login_ask() {
    let mut state = with_fixture();
    let first = run(&mut state, "Store Group 2");
    let second = dispatch(&mut state, "Store Group 2");
    let DispatchResult::Ask(a) = second else {
        panic!("{first} / {}", match second { DispatchResult::Output(s) => s, _ => (String::new()).into() });
    };
    assert_eq!(a.line_of("Overwrite").as_deref(), Some("Store Group 2 /Overwrite"));
    assert!(a.refusal().contains("/Merge, /Overwrite or /Remove"));
    run(&mut state, "Store User \"LD\" /Password \"pw\"");
    let login = ask(&mut state, "Login \"LD\"");
    assert!(matches!(login.view().inputs[0].kind, InputKind::Password));
    assert!(login.refusal().contains("Login \"LD\" \"password\""));
}

fn as_user(state: &mut AppState, user: &str) {
    state.acting_user = Some(user.to_string());
    state.root.runtime.user(user);
}

#[test]
fn a_claim_ask_goes_to_the_owners_clients_and_remember_answers_later_asks() {
    let mut state = with_fixture();
    run(&mut state, "Store User \"LD\" /Rights Program");
    as_user(&mut state, "Admin");
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 3");
    as_user(&mut state, "LD");
    let claimed = run(&mut state, "Claim Group 3");
    // Admin asks for it: LD's clients get the dialog, Admin's don't.
    as_user(&mut state, "Admin");
    let asked = run(&mut state, "Claim Group 3");
    assert_eq!(state.root.runtime.claim_asks.len(), 1, "{claimed} / {asked}");
    let ld = for_client(&state, &who("LD", Some("ld-tab"))).0;
    assert_eq!(ld.len(), 1);
    assert!(ld[0].id.starts_with("claim:"));
    assert!(for_client(&state, &who("Admin", Some("a-tab"))).0.is_empty());
    // Hand over, remembered: the line carries /Remember.
    let mut a = DialogAnswer { id: ld[0].id.clone(), button: 0, ..Default::default() };
    a.toggles.insert("remember".into(), true);
    let line = answer(&mut state, &who("LD", Some("ld-tab")), &a).unwrap().unwrap();
    assert_eq!(line, "Unclaim Group 3 /Remember");
    as_user(&mut state, "LD");
    run(&mut state, &line);
    assert_eq!(state.claim_holder("Group 3"), Some("Admin"));
    // LD takes it back; Admin asks again and gets it at once (remembered).
    as_user(&mut state, "Admin");
    run(&mut state, "Unclaim Group 3");
    as_user(&mut state, "LD");
    run(&mut state, "Claim Group 3");
    as_user(&mut state, "Admin");
    assert!(run(&mut state, "Claim Group 3").contains("hands over to you this session"));
    assert_eq!(state.claim_holder("Group 3"), Some("Admin"));
}
