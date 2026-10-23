//  █████╗ ██████╗ ████████╗███╗   ██╗███████╗████████╗
// ██╔══██╗██╔══██╗╚══██╔══╝████╗  ██║██╔════╝╚══██╔══╝
// ███████║██████╔╝   ██║   ██╔██╗ ██║█████╗     ██║
// ██╔══██║██╔══██╗   ██║   ██║╚██╗██║██╔══╝     ██║
// ██║  ██║██║  ██║   ██║   ██║ ╚████║███████╗   ██║
// ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═══╝╚══════╝   ╚═╝
// FILE: artnet

//! Art-Net 4 — see the crate's README.md (decisions §11, §34 22): ArtDmx out to the nodes found by
//! ArtPoll, else to the network's directed broadcast; ArtDmx in, with ArtPollReply for input
//! lines. It declares itself as a protocol in [`register`].

pub mod driver;
pub mod source;

pub use source::{ArtNetNode, ArtNetSource, parse_artdmx, parse_poll_reply, poll_reply_packets};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Art-Net UDP port (fixed by protocol specification).
pub const ARTNET_PORT: u16 = 6454;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Art-Net as a protocol (decisions §34 22.1): the name lines use, sync, node discovery and the
/// one shared port 6454.
pub static ARTNET: lumacore_protocols::ProtocolDef = lumacore_protocols::ProtocolDef {
    name: "ArtNet",
    kind: lumacore_protocols::Kind::Dmx,
    outputs: true,
    inputs: true,
    port: ARTNET_PORT,
    capabilities: &[
        lumacore_protocols::Capability::Sync,
        lumacore_protocols::Capability::Discovery,
        lumacore_protocols::Capability::SharedPort,
    ],
    open: lumacore_protocols::Open::Dmx(driver::open),
    untargeted: "ArtPoll",
    universes: Some(lumacore_protocols::Universes {
        property: "PortAddress",
        first: 0,
        last: 32767,
        label: port_address_label,
        parse: port_address,
    }),
    settings: lumacore_protocols::no_settings,
    protocol_settings: lumacore_protocols::no_settings,
};

/// An Art-Net universe as typed: `5` or `0.0.5` (Net.SubNet.Universe).
fn port_address(s: &str) -> Option<u16> {
    // tidy: not-keywords (Art-Net address fields)
    if !s.contains('.') {
        return s.parse().ok();
    }
    let parts: Vec<u16> = s.split('.').map(|p| p.parse().ok()).collect::<Option<_>>()?;
    match parts.as_slice() {
        [net, sub, uni] => port_address_of(*net, *sub, *uni),
        _ => None,
    }
}

/// A Port-Address (15 bit) as `net.subnet.universe`.
pub fn port_address_label(port_address: u16) -> String {
    format!("{}.{}.{}", (port_address >> 8) & 0x7F, (port_address >> 4) & 0x0F, port_address & 0x0F)
}

/// `net.subnet.universe` parts → Port-Address.
pub fn port_address_of(net: u16, subnet: u16, universe: u16) -> Option<u16> {
    (net <= 0x7F && subnet <= 0x0F && universe <= 0x0F).then_some((net << 8) | (subnet << 4) | universe)
}

/// This crate's rows in the catalog: Art-Net, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_protocols::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(std::slice::from_ref(&ARTNET));
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

#[cfg(test)]
#[test]
fn port_addresses_round_trip() {
    let pa = port_address_of(1, 2, 3).unwrap();
    assert_eq!(port_address_label(pa), "1.2.3");
    assert!(port_address_of(0, 16, 0).is_none());
    assert_eq!(port_address("0.0.5"), Some(5));
    assert_eq!(port_address("12"), Some(12));
}
