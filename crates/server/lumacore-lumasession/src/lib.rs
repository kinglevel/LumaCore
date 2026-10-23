// ██╗     ██╗   ██╗███╗   ███╗ █████╗ ███████╗███████╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
// ██║     ██║   ██║████╗ ████║██╔══██╗██╔════╝██╔════╝██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
// ██║     ██║   ██║██╔████╔██║███████║███████╗█████╗  ███████╗███████╗██║██║   ██║██╔██╗ ██║
// ██║     ██║   ██║██║╚██╔╝██║██╔══██║╚════██║██╔══╝  ╚════██║╚════██║██║██║   ██║██║╚██╗██║
// ███████╗╚██████╔╝██║ ╚═╝ ██║██║  ██║███████║███████╗███████║███████║██║╚██████╔╝██║ ╚████║
// ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: lumasession

//! LumaSession, the station ⇄ station side — see the crate's README.md (decisions §10, §21,
//! §34 19.3, 24.2).

pub mod access;
pub mod election;
pub mod host;
pub mod link;
pub mod replication;
pub mod session;
pub mod stream;

pub use access::{ServiceAccess, Services};
pub use host::SessionHost;
pub use session::{Deferred, LumaSession, handle_dismiss, handle_invite, run_deferred, station_user};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog (none of its own), after the crates it builds on
/// (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_cmd_lumasession::register(r);
    r.enter(env!("CARGO_PKG_NAME"));
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
    lumacore_registry::install_with(crate::register);
}
