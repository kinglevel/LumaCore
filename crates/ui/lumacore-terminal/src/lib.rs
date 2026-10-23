// ████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗ █████╗ ██╗
// ╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██╔══██╗██║
//    ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║███████║██║
//    ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██╔══██║██║
//    ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║  ██║███████╗
//    ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝
// FILE: terminal

//! The terminals — see the crate's README.md (decisions §34 18).

/// The objects crate, by the name the station's `crate::core::…` paths use.
pub use lumacore_objects as core;

pub mod repl;
pub mod session_manager;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A continuous control's line as the shell receives it (decisions §30 "12f": the gesture
/// channel; `lumacore_wire::ShellClientPacket::Gesture`).
pub struct Gesture {
    pub id: String,
    pub line: String,
    pub phase: lumacore_wire::GesturePhase,
    pub window: Option<u32>,
    pub label: Option<String>,
    pub from: Option<String>,
    pub value: Option<String>,
}

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog (none of its own yet), after the crates it builds on
/// (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_suggest::register(r);
    lumacore_cmd_treeshell::register(r);
    lumacore_cmd_keys::register(r);
    lumacore_cmd_tags::register(r);
    lumacore_cmd_station::register(r);
    lumacore_cmd_devices::register(r);
    lumacore_cmd_protocols::register(r);
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
