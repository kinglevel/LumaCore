// ████████╗ █████╗  ██████╗ ███████╗
// ╚══██╔══╝██╔══██╗██╔════╝ ██╔════╝
//    ██║   ███████║██║  ███╗███████╗
//    ██║   ██╔══██║██║   ██║╚════██║
//    ██║   ██║  ██║╚██████╔╝███████║
//    ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
// FILE: tags

//! The tag tree (decisions §12 "Tags hardened"): a tag is a node with child tags, one parent
//! each, numbered within its parent (`Tag 1.2.1` = All › Lx1 › Right); every show object carries
//! GUID links to its tags in its `NodeMeta.tags`. This crate holds the tree, its paths and moves,
//! and the member index read from the show's identity walk; the words are `lumacore-cmd-tags`'.
//!
//! Owns:          Tag, TagAction, TagPath, TagError, the tree operations, members, ERRORS.
//! Doesn't own:   where the tree is stored (ShowData.tags, lumacore-tree), the words (lumacore-cmd-tags),
//!                the resolver (lumacore-objects).
//! Entry points:  tag::{Tag, TagPath, find, path_of, insert, move_under, siblings, members}.
//! Decisions:     §12 "Tags hardened", §21 "6d" (GUID links), §34 21.

pub mod errors;
pub mod tag;

pub use errors::{ERRORS, TagError};
pub use tag::{Member, Tag, TagAction, TagPath, TagTree};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog (decisions §34 21): its error catalog.
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
