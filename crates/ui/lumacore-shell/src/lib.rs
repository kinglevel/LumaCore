// ███████╗██╗  ██╗███████╗██╗     ██╗
// ██╔════╝██║  ██║██╔════╝██║     ██║
// ███████╗███████║█████╗  ██║     ██║
// ╚════██║██╔══██║██╔══╝  ██║     ██║
// ███████║██║  ██║███████╗███████╗███████╗
// ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝
// FILE: shell

//! The TCP shell port — see the crate's README.md (decisions §23 "Step 8", §34 24.2).

pub mod host;
pub mod shell;

pub use host::ShellHost;
pub use shell::{SHELL_WAIT, SUGGEST_WAIT, handle_shell_client, read_shell_packets, send_shell_packet};

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
    lumacore_terminal::register(r);
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
