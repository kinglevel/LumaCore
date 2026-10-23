//! lumacore-patch: Conflict-aware patch engine for LumaCore
//!
//! Manages fixture-to-universe address assignments.
//! Detects and prevents DMX address conflicts.
//! Provides the address map for visual representation.

pub mod address_map;
pub mod check;
pub mod errors;
pub mod patch;

pub use address_map::UniverseAddressMap;
pub use errors::PatchError;
pub use patch::{FixtureListEntry, PatchEngine};

/// This crate's rows in the catalog, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_tree::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(errors::ERRORS);
}

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
