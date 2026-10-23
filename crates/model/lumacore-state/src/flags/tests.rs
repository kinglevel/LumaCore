// ███████╗██╗      █████╗  ██████╗ ███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝██║     ██╔══██╗██╔════╝ ██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// █████╗  ██║     ███████║██║  ███╗███████╗       ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══╝  ██║     ██╔══██║██║   ██║╚════██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║     ███████╗██║  ██║╚██████╔╝███████║       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝     ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: flags tests

//! Tests of `flags`.
//!
//! Owns:          the unit tests of flags.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §31 "Flags are declared", §34 15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn handlers_read_their_flags_by_declaration() {
    let args = ["3", "\"/nc\"", "/o", "/Port=8000", "/Network", "Rig", "/Tab", "Lines", "In", "/nc"];
    assert!(has(&args, &OVERWRITE) && has(&args, &NO_CONFIRMATION));
    assert!(!has(&["nc"], &NO_CONFIRMATION), "a bare word is no flag");
    assert!(!has(&["\"/nc\""], &NO_CONFIRMATION), "a quoted one neither");
    let port = FlagDef::new("Port", "").value("port");
    let network = FlagDef::new("Network", "").value("network");
    let tab = FlagDef::new("Tab", "").rest("tab");
    assert_eq!(value(&args, &port).as_deref(), Some("8000"));
    assert_eq!(value(&args, &network).as_deref(), Some("Rig"));
    assert_eq!(rest_value(&args, &tab).as_deref(), Some("Lines In"));
    assert_eq!(value(&["/Network", "/o"], &network), None, "the next word is a flag");
    assert_eq!(positional(&args, &[&network]), vec!["3", "\"/nc\"", "Lines", "In"]);
}
