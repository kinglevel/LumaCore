//  ██████╗ ██████╗      ██╗███████╗ ██████╗████████╗███████╗
// ██╔═══██╗██╔══██╗     ██║██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║   ██║██████╔╝     ██║█████╗  ██║        ██║   ███████╗
// ██║   ██║██╔══██╗██   ██║██╔══╝  ██║        ██║   ╚════██║
// ╚██████╔╝██████╔╝╚█████╔╝███████╗╚██████╗   ██║   ███████║
//  ╚═════╝ ╚═════╝  ╚════╝ ╚══════╝ ╚═════╝   ╚═╝   ╚══════╝
// FILE: objects

//! The registry tables and every object (decisions §34 7, 15, 16): the command families (`commands::registry::rows`: commands and objects), the command
//! registry, the objects and their verbs, dispatch, and what reads the registry's contents —
//! resolve, rights, claims, the Lum vocabulary, menus, settings, suggestions, the tree views.
//! It names nothing of the station above it (server, net, terminal, LumaSession, protocols,
//! engines): LumaSession work leaves through the outbox, station work as a `DispatchResult`.
//!
//! Owns:          the modules listed below; the state crate's modules are re-exported by name.
//! Doesn't own:   AppState and the change path (`lumacore-state`), the station (`lumacore`).
//! Entry points:  commands::dispatch, objects::all, commands::registry.
//! Decisions:     §15, §23 "Step 8", §34 7, §34 15, §34 16.

pub mod attribute_board;
pub mod claims;
pub mod navigation;
pub mod commands;
pub mod demo;
pub mod lum;
pub mod devices;
pub mod menu_defs;
pub mod objects;
pub mod protocol_plan;
pub mod reference;
pub mod resolve;
pub mod references;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod ownership_tests;
pub mod rights;
pub mod self_ref;
pub mod settings_props;
pub mod tools;
pub mod tree_rows;
pub mod vfs;
pub mod vfs_runtime;

pub use navigation::nav_children_at;

// The state crate (decisions §34 7, 15): its modules by name, so `crate::…` paths reach them.
pub use lumacore_state::{
    app_state,
    identifying,
    change,
    consent,
    defs,
    device_table,
    dialogs, drives,
    errors,
    flags,
    gesture,
    look,
    oops,
    parser,
    passwords,
    patch_draft,
    pools,
    programmers,
    props,
    recovery,
    session_link,
    session_log,
    show_feedback,
    stations,
    text,
    tokens,
    tree_path, mvr_review,
};
pub use lumacore_state::{AppState, DispatchResult, FixtureTypeSource, INTERNAL_ROOT_USER, ReplContext, Reply, UndoKind, build_info, parse_show, set_git_commit};

#[cfg(test)]
#[path = "dialogs_tests.rs"]
mod dialogs_tests;
#[cfg(test)]
#[path = "tree_path_tests.rs"]
mod tree_path_tests;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_state::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.tools(tools::GROUPS, tools::TOOLS);
    commands::registry::register(r);
    r.menus(menu_defs::MENUS);
    r.menus(objects::patch::menu::MENUS);
    r.rows(objects::root::PATHS);
    r.rows(objects::patch::menu::CONTEXT);
    r.rows(commands::groups::context::CONTEXT);
    r.rows(objects::preset::CONTEXT);
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(|r| {
        crate::register(r);
        // The protocols the demo show's lines speak (its load refuses a missing one, E-SHOW-007).
        lumacore_sacn::register(r);
        lumacore_artnet::register(r);
        lumacore_osc::register(r);
        // The stand-in words for the machinery tests (decisions §34 25.16).
        r.rows(crate::commands::test_words::TEST_ROWS);
        r.commands(crate::commands::test_words::TEST_ROWS.iter().map(|c| &c.decl));
    });
}
