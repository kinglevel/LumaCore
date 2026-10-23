//! lumacore-gdtf: GDTF file parser and importer for LumaCore
//!
//! Parses DIN SPEC 15800 GDTF files and converts them into lumacore-tree types.
//! Uses a two-pass resolver: physical channels first, virtual channels second.
//!
//! # Entry point
//!
//! ```no_run
//! use lumacore_gdtf::import_gdtf;
//! use lumacore_base::TreeNode;
//! use std::path::Path;
//!
//! let (fixture_type, warnings) = import_gdtf(Path::new("my_fixture.gdtf"))
//!     .expect("import should succeed");
//! println!("Imported: {:?}", fixture_type.name());
//! for w in &warnings {
//!     println!("Warning: {w}");
//! }
//! ```

pub mod channel_map;
pub mod details;
pub mod errors;
pub mod importer;
pub mod index;
pub mod resolver;
pub mod validator;

// Public re-exports
pub use channel_map::ChannelMap;
pub use details::{FileDetails, FileRevision, file_details, thumbnail};
pub use errors::{GdtfError, GdtfWarning};
pub use importer::{file_hash, import_gdtf, import_gdtf_bytes, refresh_revision};
pub use index::{load_index, save_index};
pub use resolver::{ResolutionResult, VirtualChannelResolver};
pub use validator::validate_din_spec;

/// This crate's rows in the catalog, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
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
