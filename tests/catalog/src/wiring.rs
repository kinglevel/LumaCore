// ██╗    ██╗██╗██████╗ ██╗███╗   ██╗ ██████╗
// ██║    ██║██║██╔══██╗██║████╗  ██║██╔════╝
// ██║ █╗ ██║██║██████╔╝██║██╔██╗ ██║██║  ███╗
// ██║███╗██║██║██╔══██╗██║██║╚██╗██║██║   ██║
// ╚███╔███╔╝██║██║  ██║██║██║ ╚████║╚██████╔╝
//  ╚══╝╚══╝ ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝
// FILE: wiring

//! The rig wiring with the lines, users and patch around it (decisions §34 25.16, 25.17): lines
//! that need the station's words and another crate's.
//!
//! Owns:          nothing but its tests (wiring/tests.rs).
//! Doesn't own:   the station's own wiring tests (lumacore-cmd-station).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 25.16, 25.17.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
