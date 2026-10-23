// ██╗    ██╗ ██████╗ ██████╗ ██████╗ ███████╗
// ██║    ██║██╔═══██╗██╔══██╗██╔══██╗██╔════╝
// ██║ █╗ ██║██║   ██║██████╔╝██║  ██║███████╗
// ██║███╗██║██║   ██║██╔══██╗██║  ██║╚════██║
// ╚███╔███╔╝╚██████╔╝██║  ██║██████╔╝███████║
//  ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: words

//! The words of every crate together (decisions §34 25.16): behaviour that needs words from more
//! than one command crate — the completion over the whole catalog.
//!
//! Owns:          nothing but its tests (words/tests.rs).
//! Doesn't own:   each crate's own words' tests (beside them).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 25.16.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
