//  ██████╗ ███████╗ ██████╗███████╗██████╗     ████████╗███████╗███████╗████████╗███████╗
// ██╔═══██╗██╔════╝██╔════╝██╔════╝╚════██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║   ██║███████╗██║     ███████╗ █████╔╝       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║   ██║╚════██║██║     ╚════██║██╔═══╝        ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ╚██████╔╝███████║╚██████╗███████║███████╗       ██║   ███████╗███████║   ██║   ███████║
//  ╚═════╝ ╚══════╝ ╚═════╝╚══════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: osc52 tests

//! Tests of `clipboard`: the OSC 52 encoder and the `CopyText` event on the wire.
//!
//! Owns:          the unit tests of clipboard.rs.
//! Doesn't own:   the TUI's door (lumacore-terminal `repl::clipboard`).
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
use crate::{ShellServerPacket, UiEvent};

#[test]
fn base64_matches_the_rfc_vectors() {
    for (plain, want) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(plain.as_bytes()), want, "{plain}");
    }
    // UTF-8 goes as its bytes.
    assert_eq!(base64("Ø".as_bytes()), "w5g=");
}

#[test]
fn osc52_wraps_the_text_for_the_terminal() {
    assert_eq!(osc52("Group 1").unwrap(), "\x1b]52;c;R3JvdXAgMQ==\x07");
    assert_eq!(osc52(""), Err(CopyRefused::Empty));
}

#[test]
fn osc52_refuses_a_text_past_the_cap_and_says_why() {
    assert!(osc52(&"x".repeat(OSC52_MAX_TEXT)).is_ok());
    let long = "x".repeat(OSC52_MAX_TEXT + 1);
    let refused = osc52(&long).unwrap_err();
    assert_eq!(refused, CopyRefused::TooLong { len: OSC52_MAX_TEXT + 1, max: OSC52_MAX_TEXT });
    let said = refused.to_string();
    assert!(said.contains("Too long to copy") && said.contains("75001"), "{said}");
}

#[test]
fn copy_text_round_trips_as_a_shell_packet() {
    let event = UiEvent::CopyText { label: "GUID".into(), text: "0f3c-…".into() };
    let line = serde_json::to_string(&ShellServerPacket::Ui { event: event.clone() }).unwrap();
    assert!(line.contains(r#""action":"copy_text""#), "{line}");
    let back: ShellServerPacket = serde_json::from_str(&line).unwrap();
    assert!(matches!(back, ShellServerPacket::Ui { event: e } if e == event));
}
