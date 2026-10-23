// ██████╗ ██╗ ██████╗ ██╗  ██╗████████╗███████╗
// ██╔══██╗██║██╔════╝ ██║  ██║╚══██╔══╝██╔════╝
// ██████╔╝██║██║  ███╗███████║   ██║   ███████╗
// ██╔══██╗██║██║   ██║██╔══██║   ██║   ╚════██║
// ██║  ██║██║╚██████╔╝██║  ██║   ██║   ███████║
// ╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: rights

//! The rights every line needs, whatever crate adds its word (decisions §30 "12d hardening", §34
//! 25.15): one table of lines and levels, through the real gate and the help.
//!
//! Owns:          nothing but its tests (rights/tests.rs).
//! Doesn't own:   the checks' subjects (each crate's rows).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d hardening", §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
