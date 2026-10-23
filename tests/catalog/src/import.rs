// ██╗███╗   ███╗██████╗  ██████╗ ██████╗ ████████╗
// ██║████╗ ████║██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝
// ██║██╔████╔██║██████╔╝██║   ██║██████╔╝   ██║
// ██║██║╚██╔╝██║██╔═══╝ ██║   ██║██╔══██╗   ██║
// ██║██║ ╚═╝ ██║██║     ╚██████╔╝██║  ██║   ██║
// ╚═╝╚═╝     ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝
// FILE: import

//! Import Show across the crates (decisions §30 "12d part 5", §34 25.16): parts and single objects
//! by GUID in one Oops step, Network and Station among them.
//!
//! Owns:          nothing but its tests (import/tests.rs).
//! Doesn't own:   each crate's own words' tests.
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d part 5", §34 25.16.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
