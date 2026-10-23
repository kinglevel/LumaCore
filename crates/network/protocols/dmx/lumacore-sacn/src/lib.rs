// ███████╗ █████╗  ██████╗███╗   ██╗
// ██╔════╝██╔══██╗██╔════╝████╗  ██║
// ███████╗███████║██║     ██╔██╗ ██║
// ╚════██║██╔══██║██║     ██║╚██╗██║
// ███████║██║  ██║╚██████╗██║ ╚████║
// ╚══════╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═══╝
// FILE: sacn

//! sACN (E1.31) — see the crate's README.md (decisions §11, §34 22): a hand-encoded sender bound
//! to one network interface ([`SacnSource`]) and a receiver for input lines ([`SacnReceiver`]).
//! It declares itself as a protocol in [`register`].

pub mod driver;
pub mod receiver;
pub mod source;

pub use receiver::{SacnData, SacnReceiver, parse_data_packet};
pub use source::{Destination, SACN_PORT, SacnSource, multicast_addr_for_universe};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// sACN as a protocol (decisions §34 22.1): the name lines use and E1.31 sync.
pub static SACN: lumacore_protocols::ProtocolDef = lumacore_protocols::ProtocolDef {
    name: "Sacn",
    kind: lumacore_protocols::Kind::Dmx,
    outputs: true,
    inputs: true,
    port: SACN_PORT,
    capabilities: &[lumacore_protocols::Capability::Sync],
    open: lumacore_protocols::Open::Dmx(driver::open),
    untargeted: "Multicast",
    universes: Some(lumacore_protocols::Universes {
        property: "SacnUniverse",
        first: 1,
        last: 63999,
        label: |u| u.to_string(),
        parse: |s| s.parse().ok(),
    }),
    settings,
    protocol_settings: lumacore_protocols::no_settings,
};

/// sACN's own setting (decisions §34 22.4): the E1.31 priority receivers merge by.
fn settings() -> Vec<lumacore_registry::setting::SettingDef> {
    use lumacore_registry::setting::{SettingDef, SettingKind};
    vec![
        SettingDef::new("Priority", "Priority", SettingKind::Int { min: 0, max: 200, named: Vec::new() }, serde_json::json!(100))
            .hint("E1.31 priority: a receiver takes the highest (0–200)"),
    ]
    .into_iter()
    // What a line sends with: output lines only.
    .map(|mut d| {
        d.section = "Output".to_string();
        d
    })
    .collect()
}

/// This crate's rows in the catalog: sACN, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_protocols::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(std::slice::from_ref(&SACN));
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
