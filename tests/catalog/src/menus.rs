// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗███████╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║██╔════╝
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║███████╗
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║╚════██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝███████║
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
// FILE: menus

//! Every menu of the catalog, whatever crate adds it (decisions §30 "12d", §34 25.15): its JSON
//! (tabs, columns, the slab) and every column's property on its object.
//!
//! Owns:          nothing but its tests (menus/tests.rs).
//! Doesn't own:   the checks' subjects (each crate's rows).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d", §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
