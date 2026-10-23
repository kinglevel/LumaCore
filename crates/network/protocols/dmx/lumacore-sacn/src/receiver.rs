// ██████╗ ███████╗ ██████╗███████╗██╗██╗   ██╗███████╗██████╗
// ██╔══██╗██╔════╝██╔════╝██╔════╝██║██║   ██║██╔════╝██╔══██╗
// ██████╔╝█████╗  ██║     █████╗  ██║██║   ██║█████╗  ██████╔╝
// ██╔══██╗██╔══╝  ██║     ██╔══╝  ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗
// ██║  ██║███████╗╚██████╗███████╗██║ ╚████╔╝ ███████╗██║  ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝╚══════╝╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝
// FILE: receiver

//! sACN (E1.31) receiver on one network interface (decisions §11 "Step 4c
//! hardening"): joins the multicast groups of the input lines' universes on
//! that NIC and hears unicast too. Ignored: preview data, non-zero start
//! codes (e.g. per-address priority), our own CID, and — when the socket
//! can't be pinned to the NIC — sources outside the NIC's subnet. E1.31
//! sync packets are read too (the merger releases held data on them).
//!
//! Owns:          SacnReceiver (multicast groups per input universe on one NIC), parse_data_packet, parse_sync_packet.
//! Doesn't own:   the merge of sources (input), the sender (sacn::source).
//! Entry points:  SacnReceiver, parse_data_packet, parse_sync_packet.
//! Decisions:     §11 "Step 4c hardening".

use std::collections::BTreeSet;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

use uuid::Uuid;

use crate::source::{SACN_PORT, multicast_addr_for_universe};
use lumacore_protocols::input::{Packet, SourceKey, SourceName};
use lumacore_net::nics::{self, Nic};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

const OPTION_PREVIEW: u8 = 0x80;
const OPTION_TERMINATED: u8 = 0x40;

/// An E1.31 data packet as received.
#[derive(Debug, Clone, PartialEq)]
pub struct SacnData {
    pub cid: Uuid,
    pub name: SourceName,
    pub priority: u8,
    /// Synchronization address (0 = none).
    pub sync: u16,
    pub sequence: u8,
    pub preview: bool,
    pub terminated: bool,
    pub universe: u16,
    pub start_code: u8,
    pub slots: [u8; 512],
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Read an E1.31 data packet; None for anything else.
pub fn parse_data_packet(p: &[u8]) -> Option<SacnData> {
    if p.len() < 126
        || p[0..4] != [0x00, 0x10, 0x00, 0x00]
        || &p[4..16] != b"ASC-E1.17\0\0\0"
        || p[18..22] != [0, 0, 0, 4]
        || p[40..44] != [0, 0, 0, 2]
        || p[117] != 0x02
    {
        return None;
    }
    let cid = Uuid::from_slice(&p[22..38]).ok()?;
    let name = SourceName::from_bytes(&p[44..108]);
    let count = usize::from(u16::from_be_bytes([p[123], p[124]]));
    let slots_len = count.saturating_sub(1).min(512).min(p.len() - 126);
    let mut slots = [0u8; 512];
    slots[..slots_len].copy_from_slice(&p[126..126 + slots_len]);
    Some(SacnData {
        cid,
        name,
        // E1.31: 0–200; an out-of-spec sender never outranks every legal one.
        priority: p[108].min(200),
        sync: u16::from_be_bytes([p[109], p[110]]),
        sequence: p[111],
        preview: p[112] & OPTION_PREVIEW != 0,
        terminated: p[112] & OPTION_TERMINATED != 0,
        universe: u16::from_be_bytes([p[113], p[114]]),
        start_code: p[125],
        slots,
    })
}

/// An E1.31 synchronization packet: (CID, sequence, sync address).
pub fn parse_sync_packet(p: &[u8]) -> Option<(Uuid, u8, u16)> {
    if p.len() < 47
        || p[0..4] != [0x00, 0x10, 0x00, 0x00]
        || &p[4..16] != b"ASC-E1.17\0\0\0"
        || p[18..22] != [0, 0, 0, 8]
        || p[40..44] != [0, 0, 0, 1]
    {
        return None;
    }
    let cid = Uuid::from_slice(&p[22..38]).ok()?;
    Some((cid, p[44], u16::from_be_bytes([p[45], p[46]])))
}

/// Receives sACN on one NIC.
pub struct SacnReceiver {
    socket: UdpSocket,
    nic: Nic,
    pinned: bool,
    own_cid: Uuid,
    joined: BTreeSet<u16>,
}

impl SacnReceiver {
    pub fn new(nic: &Nic, own_cid: Uuid) -> std::io::Result<Self> {
        let (socket, pinned) = nics::listen(nic, SACN_PORT)?;
        Ok(Self {
            socket,
            nic: nic.clone(),
            pinned,
            own_cid,
            joined: BTreeSet::new(),
        })
    }

    /// Whether the shared port-5568 socket is pinned to the NIC (when not,
    /// unicast may go to another program on the port; decisions §11).
    pub fn pinned(&self) -> bool {
        self.pinned
    }

    /// Join the multicast groups of `universes` on this NIC (and leave the
    /// others). Returns the first join error.
    pub fn listen_to(&mut self, universes: &BTreeSet<u16>) -> Option<String> {
        let mut error = None;
        for u in self.joined.difference(universes).copied().collect::<Vec<_>>() {
            let _ = self
                .socket
                .leave_multicast_v4(&multicast_addr_for_universe(u), &self.nic.ip);
            self.joined.remove(&u);
        }
        for &u in universes.difference(&self.joined.clone()) {
            match self
                .socket
                .join_multicast_v4(&multicast_addr_for_universe(u), &self.nic.ip)
            {
                Ok(()) => {
                    self.joined.insert(u);
                }
                Err(e) => {
                    error.get_or_insert_with(|| format!("join sACN universe {u}: {e}"));
                }
            }
        }
        error
    }

    /// Everything received since the last call, appended to `out` (a
    /// reused buffer: nothing is allocated per packet).
    pub fn receive(&mut self, out: &mut Vec<Packet>) {
        let mut buf = [0u8; 700];
        while let Ok((n, from)) = self.socket.recv_from(&mut buf) {
            let SocketAddr::V4(from) = from else {
                continue;
            };
            let ip = *from.ip();
            if !self.pinned && !self.nic.on_subnet(ip) {
                continue;
            }
            if let Some((cid, sequence, address)) = parse_sync_packet(&buf[..n]) {
                if cid != self.own_cid {
                    let mut p = Packet::dmx(SourceKey::Cid(cid), SourceName::from_bytes(&[]), ip, 0, address, [0; 512]);
                    p.sequence = Some(sequence);
                    p.is_sync = true;
                    out.push(p);
                }
                continue;
            }
            let Some(d) = parse_data_packet(&buf[..n]) else {
                continue;
            };
            if d.cid == self.own_cid || d.preview || (d.start_code != 0 && !d.terminated) {
                continue;
            }
            let mut p = Packet::dmx(SourceKey::Cid(d.cid), d.name, ip, d.priority, d.universe, d.slots);
            p.terminated = d.terminated;
            p.sequence = Some(d.sequence);
            p.sync = d.sync;
            out.push(p);
        }
    }

    /// The address it hears on (for status).
    pub fn ip(&self) -> Ipv4Addr {
        self.nic.ip
    }
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
