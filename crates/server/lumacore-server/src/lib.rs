// ███████╗███████╗██████╗ ██╗   ██╗███████╗██████╗
// ██╔════╝██╔════╝██╔══██╗██║   ██║██╔════╝██╔══██╗
// ███████╗█████╗  ██████╔╝██║   ██║█████╗  ██████╔╝
// ╚════██║██╔══╝  ██╔══██╗╚██╗ ██╔╝██╔══╝  ██╔══██╗
// ███████║███████╗██║  ██║ ╚████╔╝ ███████╗██║  ██║
// ╚══════╝╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝
// FILE: server

//! The station's server — see the crate's README.md (decisions §23 "Step 8", §26, §34 24.2).

pub mod app;
pub mod builder;
pub mod context;
pub mod net;
pub mod state;

#[cfg(test)]
mod lumasession;

pub use builder::{Ended, SHUTDOWN_DEADLINE, Server, ServerBuilder, ServerConfig, restart_process, serve};
pub use context::ServerContext;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog (none of its own), after the crates it builds on (decisions
/// §34 21); the protocols it runs are the assembler's to add.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_views::register(r);
    lumacore_cmd_showfile::register(r);
    lumacore_cmd_devices::register(r);
    lumacore_cmd_protocols::register(r);
    lumacore_shell::register(r);
    lumacore_lumasession::register(r);
    lumacore_lines::register(r);
    #[cfg(feature = "http")]
    lumacore_http::register(r);
    r.enter(env!("CARGO_PKG_NAME"));
}

/// The server never installs the catalog itself (decisions §34 24.1: the station's binaries do,
/// before they start it); starting without one is a mistake of the caller.
pub fn require_catalog() -> Result<(), String> {
    if lumacore_registry::installed() {
        Ok(())
    } else {
        Err("the catalog isn't installed: the binary installs it before it starts the server (decisions §34 24.1)".into())
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// This crate's tests read the catalog, with the protocols the station builds with: it is
/// installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(|r| {
        crate::register(r);
        lumacore_artnet::register(r);
        lumacore_sacn::register(r);
        lumacore_osc::register(r);
    });
}
