// ██╗  ██╗███████╗██████╗ ███╗   ██╗███████╗██╗
// ██║ ██╔╝██╔════╝██╔══██╗████╗  ██║██╔════╝██║
// █████╔╝ █████╗  ██████╔╝██╔██╗ ██║█████╗  ██║
// ██╔═██╗ ██╔══╝  ██╔══██╗██║╚██╗██║██╔══╝  ██║
// ██║  ██╗███████╗██║  ██║██║ ╚████║███████╗███████╗
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
// FILE: kernel

//! The kernel's machinery on real words from several crates (decisions §34 25.16): the change
//! path, the demo show, the reference check, the menu listing — until stand-in words cover them.
//!
//! Owns:          nothing but its tests (kernel/tests.rs).
//! Doesn't own:   the kernel's own machinery tests (its stand-in words).
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
