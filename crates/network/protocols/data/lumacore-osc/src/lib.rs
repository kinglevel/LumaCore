//  ██████╗ ███████╗ ██████╗
// ██╔═══██╗██╔════╝██╔════╝
// ██║   ██║███████╗██║
// ██║   ██║╚════██║██║
// ╚██████╔╝███████║╚██████╗
//  ╚═════╝ ╚══════╝ ╚═════╝
// FILE: osc

//! OSC input — see the crate's README.md (decisions §11, §34 22): a Data protocol, whose
//! messages become command lines run as the line's user. It declares itself in [`register`].

pub mod listener;

pub use listener::{OscListener, parse_osc_packet};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The UDP port a new OSC input line listens on.
pub const OSC_PORT: u16 = 8000;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// OSC as a protocol (decisions §34 22.2): input lines only, Data kind.
pub static OSC: lumacore_protocols::ProtocolDef = lumacore_protocols::ProtocolDef {
    name: "Osc",
    kind: lumacore_protocols::Kind::Data,
    outputs: false,
    inputs: true,
    port: OSC_PORT,
    capabilities: &[],
    open: lumacore_protocols::Open::Control(listener::open),
    untargeted: "Any source",
    universes: None,
    settings,
    protocol_settings: lumacore_protocols::no_settings,
};

/// OSC's own setting (decisions §34 22.4): the UDP port an input line listens on.
fn settings() -> Vec<lumacore_registry::setting::SettingDef> {
    use lumacore_registry::setting::{SettingDef, SettingKind};
    vec![
        SettingDef::new("Port", "Port", SettingKind::Int { min: 1, max: 65535, named: Vec::new() }, serde_json::json!(OSC_PORT))
            .hint("the UDP port the line listens on"),
    ]
}

/// This crate's rows in the catalog: OSC, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_protocols::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(std::slice::from_ref(&OSC));
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
