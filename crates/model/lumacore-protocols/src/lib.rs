// ██████╗ ██████╗  ██████╗ ████████╗ ██████╗  ██████╗ ██████╗ ██╗     ███████╗
// ██╔══██╗██╔══██╗██╔═══██╗╚══██╔══╝██╔═══██╗██╔════╝██╔═══██╗██║     ██╔════╝
// ██████╔╝██████╔╝██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║     ███████╗
// ██╔═══╝ ██╔══██╗██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║     ╚════██║
// ██║     ██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╗╚██████╔╝███████╗███████║
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝ ╚═════╝ ╚══════╝╚══════╝
// FILE: protocols

//! The protocol contract — see the crate's README.md (decisions §34 22, 25.14): what a protocol
//! is, the line data the show holds, the drivers' interface and the pure logic around them.

pub mod errors;
pub mod input;
pub mod line;
pub mod line_plan;
pub mod nic;
pub mod protocol;

pub use errors::DmxError;
pub use input::{InputLayer, InputMerger, InputPlan, Packet, SourceInfo, SourceKey, SourceName};
pub use line::{
    DEFAULT_LINE_RATE, DEFAULT_NETWORK, Direction, InputPriority, Line, LineSend, MergeMode, Network,
    mode_label, wire_label,
};
pub use line_plan::LinePlan;
pub use nic::Nic;
pub use protocol::{Capability, ControlListener, ControlStart, DmxDriver, DriverIdentity, Kind, Node, Open, ProtocolDef, Universes, no_settings};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog: its error codes and the check of every protocol row
/// (decisions §34 21, 22; it builds on no crate with rows). The protocols themselves add their
/// rows in their own crates.
pub fn register(r: &mut lumacore_registry::Registry) {
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(errors::ERRORS).check(protocol::check);
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
