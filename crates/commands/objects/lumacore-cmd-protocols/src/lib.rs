// ██████╗ ██████╗  ██████╗ ████████╗ ██████╗  ██████╗ ██████╗ ██╗     ███████╗
// ██╔══██╗██╔══██╗██╔═══██╗╚══██╔══╝██╔═══██╗██╔════╝██╔═══██╗██║     ██╔════╝
// ██████╔╝██████╔╝██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║     ███████╗
// ██╔═══╝ ██╔══██╗██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║     ╚════██║
// ██║     ██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╗╚██████╔╝███████╗███████║
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝ ╚═════╝ ╚══════╝╚══════╝
// FILE: protocols

//! The protocol lines' shared words (decisions §11, §34 25.7, 25.17, "34f7c hardened" 9): the one
//! set of line handlers and properties every protocol's object plugs into (`line_object!`,
//! `dmx_line_verbs!` — each protocol's object is its own crate's under
//! `commands/objects/protocols/`) and the DMX Protocols menu (`DMX` is the kernel's object). A line names its
//! Network (the station's, `lumacore-cmd-station`, which this crate sits on); the drivers are
//! `lumacore-lines` and the protocol crates.
//!
//! Owns:          the shared line handlers (`lines`, `props`), the DMX Protocols menu.
//! Doesn't own:   each protocol's object (lumacore-cmd-sacn, -artnet, -osc), the Network and the station's NICs (cmd-station), the drivers (lumacore-lines).
//! Entry points:  register, line_object!, dmx_line_verbs!, ROWS.
//! Decisions:     §11, §30 "12d", §34 25.7, 25.17, "34f7c hardened".

pub mod lines;
mod menus;
pub mod props;
mod rows;

pub use rows::{HOOKS, OBJECTS, ROWS};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Adds the DMX Protocols menu to the catalog, after the station's.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_cmd_station::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(ROWS);
    r.commands(ROWS.iter().map(|c| &c.decl));
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
    r.menus(menus::MENUS);
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

/// A test binary's catalog: the station's and the lines' (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
