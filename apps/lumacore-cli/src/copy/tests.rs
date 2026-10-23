//  ██████╗ ██████╗ ██████╗ ██╗   ██╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝██╔═══██╗██╔══██╗╚██╗ ██╔╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║     ██║   ██║██████╔╝ ╚████╔╝        ██║   █████╗  ███████╗   ██║   ███████╗
// ██║     ██║   ██║██╔═══╝   ╚██╔╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ╚██████╗╚██████╔╝██║        ██║          ██║   ███████╗███████║   ██║   ███████║
//  ╚═════╝ ╚═════╝ ╚═╝        ╚═╝          ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: copy tests

//! Tests of the CLI's `copy`: what it writes to its terminal.
//!
//! Owns:          the unit tests of copy.rs.
//! Doesn't own:   the encoder's own tests (lumacore-wire `clipboard/tests.rs`).
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
use lumacore_wire::clipboard::OSC52_MAX_TEXT;

#[test]
fn a_copy_writes_osc52_to_the_terminal() {
    let mut out = Vec::new();
    assert_eq!(write_osc52(&mut out, "Root.ShowData.Groups"), Ok(()));
    assert_eq!(out, b"\x1b]52;c;Um9vdC5TaG93RGF0YS5Hcm91cHM=\x07");
}

#[test]
fn a_text_past_the_cap_writes_nothing_and_says_why() {
    let mut out = Vec::new();
    let said = write_osc52(&mut out, &"g".repeat(OSC52_MAX_TEXT + 1)).unwrap_err();
    assert!(out.is_empty());
    assert!(said.starts_with("Too long to copy through the terminal"), "{said}");
}

#[test]
fn the_session_clipboard_writes_the_terminal_whatever_the_desktop() {
    // A CLI without a desktop (the native side tried and absent, as over SSH): the
    // terminal still gets the sequence. (The test never opens the real desktop
    // clipboard: it would overwrite the developer's.)
    let mut clipboard = UserClipboard { native: None, tried: true };
    let mut out = Vec::new();
    let copied = clipboard.copy(&mut out, "abc");
    assert_eq!(copied, Copied { terminal: Ok(()), native: false });
    assert_eq!(out, b"\x1b]52;c;YWJj\x07");
}
