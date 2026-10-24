//  ██████╗███╗   ███╗██████╗     ████████╗ █████╗  ██████╗ ███████╗
// ██╔════╝████╗ ████║██╔══██╗    ╚══██╔══╝██╔══██╗██╔════╝ ██╔════╝
// ██║     ██╔████╔██║██║  ██║       ██║   ███████║██║  ███╗███████╗
// ██║     ██║╚██╔╝██║██║  ██║       ██║   ██╔══██║██║   ██║╚════██║
// ╚██████╗██║ ╚═╝ ██║██████╔╝       ██║   ██║  ██║╚██████╔╝███████║
//  ╚═════╝╚═╝     ╚═╝╚═════╝        ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
// FILE: cmd tags

//! The Tag object's words (decisions §12 "Tags hardened"): Store, Label, Delete (with / keep its
//! child tags), Move (nests a tag, or renumbers it), Assign / Unassign (tags any show object),
//! List, Info and the default verb, which selects a tag's fixtures — or, with no path and objects
//! after it, opens the tag picker on them (`Tag Group 3`, decisions 0297). The tree is `lumacore-tags`';
//! its ids read as a path (`Tag 1.2.1`), walked by the one resolver through this crate's
//! `PathDef` row.
//!
//! Owns:          the Tag object (`TAG`), its path row (`PATHS`), each verb, the tag links' edits.
//! Doesn't own:   the tree and its operations (lumacore-tags), the resolver (lumacore-objects).
//! Entry points:  register, TAG, PATHS, OBJECTS, armed.
//! Decisions:     §12 "Tags hardened", §34 21.

mod armed;
mod assign;
mod delete;
mod label;
mod links;
mod list;
mod members;
mod nest;
mod picker;
mod props;
mod select;
mod settings;
mod store;
mod tag;

pub use armed::{Armed, armed, arming};
pub use settings::TABS;
pub use tag::{PATHS, TAG};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The Tag object, for the catalog.
pub static OBJECTS: &[&lumacore_objects::objects::ObjectDef] = &[&TAG];

/// The Tag object and its path row, after the kernel's.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_objects::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(PATHS);
    r.rows(TABS);
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

/// A test binary's catalog: the kernel's and the tags' (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
