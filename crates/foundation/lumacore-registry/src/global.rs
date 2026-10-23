//  ██████╗ ██╗      ██████╗ ██████╗  █████╗ ██╗
// ██╔════╝ ██║     ██╔═══██╗██╔══██╗██╔══██╗██║
// ██║  ███╗██║     ██║   ██║██████╔╝███████║██║
// ██║   ██║██║     ██║   ██║██╔══██╗██╔══██║██║
// ╚██████╔╝███████╗╚██████╔╝██████╔╝██║  ██║███████╗
//  ╚═════╝ ╚══════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: global
//! The one catalog of the process (decisions §34 21): installed once at start — by the server,
//! the sweep, gen-docs, or a crate's tests through [`install_with`] — and read everywhere with
//! [`get`]. Frozen for the run: no lock, no version. The first install wins; a second (two
//! servers in one test process) builds the same catalog and is dropped.
//!
//! Owns:          the process-wide catalog, install, install_with, get, installed
//! Doesn't own:   the rows (each crate's register), the checks (catalog)
//! Entry points:  install, install_with, get, installed
//! Decisions:     §34 21

use std::sync::OnceLock;

use crate::{Catalog, Registry};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

static CATALOG: OnceLock<Catalog> = OnceLock::new();

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Makes `catalog` the process's; false when one was installed before (that one stays).
pub fn install(catalog: Catalog) -> bool {
    CATALOG.set(catalog).is_ok()
}

/// Builds with `register` (a crate's `register`: it and everything below), freezes and
/// installs, once per process; panics with every problem when it doesn't freeze (a test or a
/// tool — the server reports it instead).
pub fn install_with(register: fn(&mut Registry)) {
    if CATALOG.get().is_some() {
        return;
    }
    let mut r = Registry::new();
    register(&mut r);
    match r.freeze() {
        Ok(catalog) => drop(install(catalog)),
        Err(e) => panic!("{e}"),
    }
}

/// The process's catalog.
///
/// # Panics
/// When nothing installed one yet: the server installs it at start; a test calls
/// `lumacore_registry::install_with(<its crate>::register)` first.
pub fn get() -> &'static Catalog {
    CATALOG.get().expect(
        "the catalog isn't installed (decisions §34 21): the server installs it at start; a test or tool calls \
         lumacore_registry::install_with(<its crate>::register) first",
    )
}

/// Whether a catalog is installed (a debug check that may run before it is).
pub fn installed() -> bool {
    CATALOG.get().is_some()
}
