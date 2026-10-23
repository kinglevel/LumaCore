// ██╗     ██╗   ██╗███╗   ███╗ █████╗ ███████╗███████╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
// ██║     ██║   ██║████╗ ████║██╔══██╗██╔════╝██╔════╝██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
// ██║     ██║   ██║██╔████╔██║███████║███████╗█████╗  ███████╗███████╗██║██║   ██║██╔██╗ ██║
// ██║     ██║   ██║██║╚██╔╝██║██╔══██║╚════██║██╔══╝  ╚════██║╚════██║██║██║   ██║██║╚██╗██║
// ███████╗╚██████╔╝██║ ╚═╝ ██║██║  ██║███████║███████╗███████║███████║██║╚██████╔╝██║ ╚████║
// ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: lumasession

//! The LumaSession words (decisions §11, §21, §30 "12d", §34 25.7): `Join LumaSession "name" |
//! <host>` (a name found through discovery, `/Merge`), `Leave LumaSession` (asks: leaving holds DMX
//! output), `Claim LumaSession [Station "X"]` (take the master role, or hand it on; asks), `Pause
//! LumaSession` (stop a decision's countdown) and the LumaSession object (List / Info). Every one
//! only asks the kernel's outbox (`session_link`: a link request, a session request); the network
//! side is `lumacore-lumasession`. Invite / Dismiss are verbs of the Station object, whose
//! handlers stay with it — this crate holds their rows.
//!
//! Owns:          the rows (`rows`), Join's parsing, Leave's ask, Claim / Pause, the LumaSession object.
//! Doesn't own:   the outbox and the session's state (the kernel's session_link), the stream (lumacore-lumasession).
//! Entry points:  register, ROWS, OBJECTS, HOOKS.
//! Decisions:     §11, §21 "6b" "6c" "6d", §30 "12d", §34 25.7.

mod claim;
mod join;
mod leave;
mod lumasession;
mod rows;

pub use rows::{HOOKS, OBJECTS, ROWS};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Adds the LumaSession rows to the catalog, after the kernel's.
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

/// A test binary's catalog: the kernel's and the LumaSession words (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
