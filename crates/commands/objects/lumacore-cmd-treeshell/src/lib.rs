// ████████╗██████╗ ███████╗███████╗███████╗██╗  ██╗███████╗██╗     ██╗
// ╚══██╔══╝██╔══██╗██╔════╝██╔════╝██╔════╝██║  ██║██╔════╝██║     ██║
//    ██║   ██████╔╝█████╗  █████╗  ███████╗███████║█████╗  ██║     ██║
//    ██║   ██╔══██╗██╔══╝  ██╔══╝  ╚════██║██╔══██║██╔══╝  ██║     ██║
//    ██║   ██║  ██║███████╗███████╗███████║██║  ██║███████╗███████╗███████╗
//    ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝
// FILE: treeshell

//! The tree shell (decisions §33 18, §34 25.7, §35 19): the fenced Unix words on the object tree
//! and the drives — cd, ls, pwd, cat, head, tail, stat, tree, find, grep, du, wc, diff, and mv /
//! cp / rm, which on the show tree are the Lum line they mean — plus Enter and Exit. Walking the
//! tree (vfs, navigation, Enter's claim, the `ls` listing) is the kernel's; this crate holds the
//! words and what they print.
//!
//! Owns:          the tree shell's rows (`rows`), its words, their printing, mv / cp / rm.
//! Doesn't own:   the tree's walk (lumacore-objects `vfs`, `navigation`, `commands::navigate`).
//! Entry points:  register, ROWS, HOOKS.
//! Decisions:     §33 18, §34 25.7, 25.13, §35 19.

mod count;
mod diff;
mod door;
mod print;
mod read;
mod rows;
mod search;
mod tree;
mod words;

pub use rows::{HOOKS, OBJECTS, ROWS};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Adds the tree shell's rows to the catalog, after the kernel's.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_objects::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(ROWS);
    r.commands(ROWS.iter().map(|c| &c.decl));
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;

/// A test binary's catalog: the kernel's and the tree shell's (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
