//  ██████╗ ██████╗ ██████╗ ██╗   ██╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝██╔═══██╗██╔══██╗╚██╗ ██╔╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║     ██║   ██║██████╔╝ ╚████╔╝        ██║   █████╗  ███████╗   ██║   ███████╗
// ██║     ██║   ██║██╔═══╝   ╚██╔╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ╚██████╗╚██████╔╝██║        ██║          ██║   ███████╗███████║   ██║   ███████║
//  ╚═════╝ ╚═════╝ ╚═╝        ╚═╝          ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: copy tests

//! Tests of `repl::clipboard`: the door queues a copy for the client, the
//! console writes it as OSC 52.
//!
//! Owns:          the unit tests of clipboard.rs.
//! Doesn't own:   the encoder's tests (lumacore-wire `clipboard/tests.rs`).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/wire-crate-clients

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn copy_queues_the_text_for_the_client() {
    let mut outbox = vec![UiEvent::FocusTerminal];
    copy(&mut outbox, "GUID", "6f1d0c52-0000-4000-8000-000000000101");
    copy(&mut outbox, "Path", "");
    assert_eq!(
        outbox,
        vec![
            UiEvent::FocusTerminal,
            UiEvent::CopyText { label: "GUID".into(), text: "6f1d0c52-0000-4000-8000-000000000101".into() },
        ]
    );
}

#[test]
fn a_copy_from_a_key_reaches_the_client_with_the_keys_events() {
    let mut state = crate::core::demo::fresh_state();
    let mut app = crate::repl::TuiApp::new(&state);
    copy(&mut app.ui_outbox, "Set line", r#"Set Fixture 101 "Name" "Spot""#);
    let events = app.take_ui_events(&mut state);
    assert!(
        events.iter().any(|e| matches!(e, UiEvent::CopyText { label, text } if label == "Set line" && text.starts_with("Set Fixture 101"))),
        "{events:?}"
    );
}

#[test]
fn the_console_writes_osc52_and_keeps_the_other_events() {
    let mut outbox = vec![UiEvent::FocusTerminal];
    let mut requests = Vec::new();
    copy(&mut outbox, "GUID", "abc");
    copy(&mut requests, "Long", &"x".repeat(lumacore_wire::clipboard::OSC52_MAX_TEXT + 1));
    let mut out = Vec::new();
    let said = write_to_own_terminal([&mut outbox, &mut requests], &mut out);
    assert_eq!(out, b"\x1b]52;c;YWJj\x07");
    assert_eq!(said[0], "Copied GUID");
    assert!(said[1].starts_with("Couldn't copy Long: Too long"), "{said:?}");
    assert_eq!(outbox, vec![UiEvent::FocusTerminal]);
    assert!(requests.is_empty());
}
