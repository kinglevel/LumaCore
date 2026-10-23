//! Declared flags (decisions §31): a flag nobody declares is refused with E-CMD-018 before
//! anything runs — on one line, on a line of several statements, and when a key is saved.

use super::dispatch;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(ask) => format!("ASK {}", ask.view().text),
        _ => String::new(),
    }
}

/// Fixtures 1 and 2 (bare, no type) and Group 3 holding both.
fn with_group() -> AppState {
    let mut state = AppState::new();
    for id in [1, 2] {
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
    state.root.show_data.after_load();
    run(&mut state, "Fixture 1 Thru 2");
    let out = run(&mut state, "Store Group 3");
    assert!(state.user_groups().contains_key(&3), "{out}");
    state
}

#[test]
fn an_unknown_flag_is_refused_and_named() {
    let mut state = with_group();
    let out = run(&mut state, "Store Group 3 /Overwite");
    assert!(out.contains("E-CMD-018") && out.contains("'/Overwite'"), "{out}");
    assert!(out.contains("Store Group") && out.contains("/Overwrite") && out.contains("/Merge"), "names what it takes: {out}");
    // A flag of another verb of the same object is unknown here too.
    assert!(run(&mut state, "Delete Group 3 /Merge").contains("E-CMD-018"));
}

#[test]
fn nothing_in_a_line_runs_when_one_statement_has_an_unknown_flag() {
    let mut state = with_group();
    let out = run(&mut state, "Delete Group 3 /nc; Label Group 3 \"x\" /Foo");
    assert!(out.contains("E-CMD-018"), "{out}");
    assert!(state.user_groups().contains_key(&3), "the Delete didn't run");
}

#[test]
fn a_quoted_slash_word_is_a_value_and_globals_go_everywhere() {
    let mut state = with_group();
    let out = run(&mut state, "Label Group 3 \"/Foo\"");
    assert!(!out.contains("E-CMD-018"), "{out}");
    for line in ["List Group /nc", "List Group /NoOops", "Fixture 1 At 50 /v", "List Group /Watch=1000"] {
        assert!(!run(&mut state, line).contains("E-CMD-018"), "{line}");
    }
}

/// `nc` without its slash is a word, not `/nc` (decisions §31): four handlers took it as the
/// confirmation (`trim_start_matches('/')`).
#[test]
fn a_bare_nc_is_no_confirmation() {
    let mut state = AppState::new();
    let before = state.debug_mode;
    let out = run(&mut state, "TestAsk nc");
    assert!(out.starts_with("ASK"), "asks, doesn't toggle: {out}");
    assert_eq!(state.debug_mode, before);
    run(&mut state, "TestAsk /nc");
    assert_ne!(state.debug_mode, before, "the flag toggles at once");
}

/// A declared short form works where the handler reads the declaration: `/o` is `/Overwrite`
/// (Store Group compared "overwrite" and missed it).
#[test]
fn a_short_form_is_its_flag() {
    let mut state = with_group();
    assert_eq!(state.user_groups()[&3].members.len(), 2);
    run(&mut state, "ResetAll");
    run(&mut state, "Fixture 1");
    // A typed word has one name, no short form (lum.md "One name per word", §33 3d).
    assert!(run(&mut state, "Store Group 3 /o").contains("E-SYN-006"));
    let out = run(&mut state, "Store Group 3 /Overwrite");
    assert!(!out.starts_with("ASK"), "{out}");
    assert_eq!(state.user_groups()[&3].members.len(), 1, "overwritten: {out}");
}

/// Replies are typed (decisions §31): a refusal comes back failed with its code, and the
/// journal records the code from the reply, not from its text.
#[test]
fn a_refusal_is_a_typed_reply_and_the_journal_keeps_its_code() {
    let mut state = with_group();
    match dispatch(&mut state, "Store Group 5 /Foo") {
        DispatchResult::Output(reply) => {
            assert!(reply.failed, "{reply}");
            assert_eq!(reply.code.as_deref(), Some("E-CMD-018"));
        }
        _ => panic!("expected output"),
    }
    match dispatch(&mut state, "List Group") {
        DispatchResult::Output(reply) => assert!(!reply.failed && reply.code.is_none(), "{reply}"),
        _ => panic!("expected output"),
    }
    let out = run(&mut state, "Label Group 99 \"x\"");
    let last = state.root.runtime.journal.entries().last().expect("a journal entry");
    assert!(
        matches!(&last.result, lumacore_tree::runtime::JournalResult::Error(Some(code)) if code.starts_with("E-")),
        "{out} / {:?}",
        last.result
    );
}
