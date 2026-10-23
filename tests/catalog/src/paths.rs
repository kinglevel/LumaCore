// ██████╗  █████╗ ████████╗██╗  ██╗███████╗
// ██╔══██╗██╔══██╗╚══██╔══╝██║  ██║██╔════╝
// ██████╔╝███████║   ██║   ███████║███████╗
// ██╔═══╝ ██╔══██║   ██║   ██╔══██║╚════██║
// ██║     ██║  ██║   ██║   ██║  ██║███████║
// ╚═╝     ╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝
// FILE: paths

//! The golden record of the object tree on the whole catalog (decisions §31, §34 25.15): every
//! path a user can reach, through the tree shell, List and the objects' trees.
//!
//! Owns:          nothing but its tests (paths/tests.rs).
//! Doesn't own:   the typed paths (the kernel's tree_path), the tree shell (lumacore-cmd-treeshell).
//! Entry points:  the #[test] fns.
//! Decisions:     §31, §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
