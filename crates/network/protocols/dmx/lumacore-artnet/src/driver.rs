// ██████╗ ██████╗ ██╗██╗   ██╗███████╗██████╗
// ██╔══██╗██╔══██╗██║██║   ██║██╔════╝██╔══██╗
// ██║  ██║██████╔╝██║██║   ██║█████╗  ██████╔╝
// ██║  ██║██╔══██╗██║╚██╗ ██╔╝██╔══╝  ██╔══██╗
// ██████╔╝██║  ██║██║ ╚████╔╝ ███████╗██║  ██║
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝
// FILE: driver

//! Art-Net as a [`DmxDriver`] (decisions §34 22): one [`ArtNetSource`] on the driver's NIC, sending
//! a line's universes to the targets worked out from the nodes, answering ArtPoll for the input
//! lines and reporting the nodes it hears.
//!
//! Owns:          ArtNetDriver, open (the row's opener).
//! Doesn't own:   the packets and the socket (source), when a line sends (lumacore-lines).
//! Entry points:  open.
//! Decisions:     §11, §34 22.

use std::collections::BTreeSet;
use std::net::Ipv4Addr;

use lumacore_protocols::{DmxDriver, DmxError, DriverIdentity, LinePlan, Node, Packet};

use crate::source::ArtNetSource;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Art-Net on one NIC.
pub struct ArtNetDriver {
    source: ArtNetSource,
    /// The station's name (ArtPollReply).
    name: String,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The row's opener: bind Art-Net on the driver's NIC.
pub fn open(identity: &DriverIdentity) -> Result<Box<dyn DmxDriver>, DmxError> {
    Ok(Box::new(ArtNetDriver {
        source: ArtNetSource::new(&identity.nic)?,
        name: identity.name.clone(),
    }))
}

impl DmxDriver for ArtNetDriver {
    fn targets(&mut self, wire: u16, unicast: Option<Ipv4Addr>) -> Vec<Ipv4Addr> {
        self.source.targets(wire, unicast)
    }

    fn prepare(&mut self, streams: &[(Ipv4Addr, u16)]) {
        self.source.prepare(streams.iter().copied());
    }

    // tidy: render-begin (decisions §24: the per-frame path)
    fn send(&mut self, wire: u16, data: &[u8; 512], _line: &LinePlan, targets: &[Ipv4Addr]) -> std::io::Result<u64> {
        self.source.send(wire, data, targets)
    }

    fn sync(&mut self, _line: &LinePlan) -> std::io::Result<()> {
        self.source.sync()
    }
    // tidy: render-end

    fn receive(&mut self, wires: &BTreeSet<u16>, packets: &mut Vec<Packet>) -> Option<String> {
        let ports: Vec<u16> = wires.iter().copied().collect();
        self.source.poll(&ports, &self.name, packets);
        (!ports.is_empty() && !self.source.listening)
            .then(|| "Art-Net input: port 6454 is taken on this machine".to_string())
    }

    fn pinned(&self) -> Option<bool> {
        Some(self.source.pinned())
    }

    fn nodes(&self) -> Vec<Node> {
        self.source
            .nodes()
            .into_iter()
            .map(|n| Node {
                ip: n.ip,
                name: n.short_name,
                outputs: n.outputs,
                refresh_hz: n.refresh_hz,
            })
            .collect()
    }
}
