// ███████╗ █████╗  ██████╗███╗   ██╗    ███████╗ ██████╗ ██╗   ██╗██████╗  ██████╗███████╗
// ██╔════╝██╔══██╗██╔════╝████╗  ██║    ██╔════╝██╔═══██╗██║   ██║██╔══██╗██╔════╝██╔════╝
// ███████╗███████║██║     ██╔██╗ ██║    ███████╗██║   ██║██║   ██║██████╔╝██║     █████╗
// ╚════██║██╔══██║██║     ██║╚██╗██║    ╚════██║██║   ██║██║   ██║██╔══██╗██║     ██╔══╝
// ███████║██║  ██║╚██████╗██║ ╚████║    ███████║╚██████╔╝╚██████╔╝██║  ██║╚██████╗███████╗
// ╚══════╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═══╝    ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚══════╝
// FILE: sacn source

//! sACN (ANSI E1.31-2018) sender, bound to one network interface.
//!
//! Hand-encoded so the socket is ours: it binds to the mapped NIC's address
//! and sets the multicast interface to it (`IP_MULTICAST_IF`), so on a
//! machine with several NICs each network's output leaves on its own card
//! (decisions §11). Sends data packets, universe discovery every 10 s and
//! stream-terminated packets when a universe stops or the sender is dropped.
//!
//! Sending reuses one preallocated packet (decisions §24: the per-frame send
//! allocates nothing); a line with `Sync` puts the sync universe in its data
//! packets and sends an E1.31 synchronization packet after them.
//!
//! Owns:          SacnSource (E1.31 data, sync and discovery packets, bound to one NIC), Destination, the multicast address of a universe.
//! Doesn't own:   when a line sends (lumacore_lines::lines), the receiver (receiver).
//! Entry points:  SacnSource, Destination, multicast_addr_for_universe, SACN_PORT.
//! Decisions:     §11, §24 "Step 9 hardening".

use std::collections::{BTreeSet, HashMap};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};
use uuid::Uuid;

use lumacore_protocols::DmxError;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The sACN port.
pub const SACN_PORT: u16 = 5568;
/// Universe discovery goes to this universe's multicast group.
const DISCOVERY_UNIVERSE: u16 = 64214;
const DISCOVERY_INTERVAL: Duration = Duration::from_secs(10);
const ACN_ID: [u8; 12] = *b"ASC-E1.17\0\0\0";
const DATA_LEN: usize = 638;
const SYNC_LEN: usize = 49;
const OPTION_TERMINATED: u8 = 0x40;
/// The universe sync packets go to (E1.31 §6.3; one per sender).
pub const SYNC_UNIVERSE: u16 = 63999;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Compute the E1.31 multicast address for a universe number.
pub fn multicast_addr_for_universe(universe: u16) -> Ipv4Addr {
    Ipv4Addr::new(239, 255, (universe >> 8) as u8, (universe & 0xFF) as u8)
}

fn flags_len(len: usize) -> [u8; 2] {
    (0x7000 | len as u16).to_be_bytes()
}

fn root_layer(out: &mut Vec<u8>, total: usize, vector: u32, cid: &Uuid) {
    out.extend_from_slice(&0x0010u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&ACN_ID);
    out.extend_from_slice(&flags_len(total - 16));
    out.extend_from_slice(&vector.to_be_bytes());
    out.extend_from_slice(cid.as_bytes());
}

fn source_name(out: &mut Vec<u8>, name: &str) {
    let mut field = [0u8; 64];
    let bytes = name.as_bytes();
    let n = bytes.len().min(63);
    field[..n].copy_from_slice(&bytes[..n]);
    out.extend_from_slice(&field);
}

/// An E1.31 data packet (start code 0, 512 slots).
pub fn data_packet(
    cid: &Uuid,
    name: &str,
    universe: u16,
    priority: u8,
    sequence: u8,
    options: u8,
    data: &[u8; 512],
) -> Vec<u8> {
    let mut p = Vec::with_capacity(DATA_LEN);
    root_layer(&mut p, DATA_LEN, 0x0000_0004, cid);
    // Framing layer
    p.extend_from_slice(&flags_len(DATA_LEN - 38));
    p.extend_from_slice(&0x0000_0002u32.to_be_bytes());
    source_name(&mut p, name);
    p.push(priority.min(200));
    p.extend_from_slice(&0u16.to_be_bytes()); // sync address
    p.push(sequence);
    p.push(options);
    p.extend_from_slice(&universe.to_be_bytes());
    // DMP layer
    p.extend_from_slice(&flags_len(DATA_LEN - 115));
    p.push(0x02);
    p.push(0xA1);
    p.extend_from_slice(&0u16.to_be_bytes());
    p.extend_from_slice(&1u16.to_be_bytes());
    p.extend_from_slice(&513u16.to_be_bytes());
    p.push(0); // start code
    p.extend_from_slice(data);
    p
}

/// A universe discovery packet (one page, up to 512 universes).
pub fn discovery_packet(cid: &Uuid, name: &str, universes: &[u16]) -> Vec<u8> {
    let universes = &universes[..universes.len().min(512)];
    let total = 120 + 2 * universes.len();
    let mut p = Vec::with_capacity(total);
    root_layer(&mut p, total, 0x0000_0008, cid);
    p.extend_from_slice(&flags_len(total - 38));
    p.extend_from_slice(&0x0000_0002u32.to_be_bytes());
    source_name(&mut p, name);
    p.extend_from_slice(&[0; 4]); // reserved
    p.extend_from_slice(&flags_len(total - 112));
    p.extend_from_slice(&0x0000_0001u32.to_be_bytes());
    p.push(0); // page
    p.push(0); // last page
    for u in universes {
        p.extend_from_slice(&u.to_be_bytes());
    }
    p
}

/// An E1.31 universe synchronization packet.
pub fn sync_packet(cid: &Uuid, sequence: u8, sync_universe: u16) -> [u8; SYNC_LEN] {
    let mut p = Vec::with_capacity(SYNC_LEN);
    root_layer(&mut p, SYNC_LEN, 0x0000_0008, cid);
    p.extend_from_slice(&flags_len(SYNC_LEN - 38));
    p.extend_from_slice(&0x0000_0001u32.to_be_bytes());
    p.push(sequence);
    p.extend_from_slice(&sync_universe.to_be_bytes());
    p.extend_from_slice(&[0, 0]); // reserved
    let mut out = [0u8; SYNC_LEN];
    out.copy_from_slice(&p);
    out
}

/// Where a universe goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Destination {
    Multicast,
    Unicast(Ipv4Addr),
}

/// An sACN sender on one interface.
pub struct SacnSource {
    socket: UdpSocket,
    cid: Uuid,
    name: String,
    sequences: HashMap<u16, u8>,
    /// Streams sent and not yet terminated.
    live: BTreeSet<(Destination, u16)>,
    last_discovery: Option<Instant>,
    /// The data packet, rewritten in place for every send.
    packet: [u8; DATA_LEN],
    /// The sync packet (its sequence is rewritten).
    sync: [u8; SYNC_LEN],
    sync_sequence: u8,
}

impl SacnSource {
    /// Bind to `bind` (any free local port) and send multicast out of it.
    pub fn new(bind: Ipv4Addr, cid: Uuid, name: &str) -> Result<Self, DmxError> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        socket.bind(&SocketAddr::V4(SocketAddrV4::new(bind, 0)).into())?;
        socket.set_multicast_ttl_v4(20)?;
        socket.set_multicast_loop_v4(true)?;
        if !bind.is_unspecified() {
            socket.set_multicast_if_v4(&bind)?;
        }
        let mut packet = [0u8; DATA_LEN];
        packet.copy_from_slice(&data_packet(&cid, name, 1, 100, 0, 0, &[0; 512]));
        Ok(Self {
            socket: socket.into(),
            cid,
            name: name.to_string(),
            sequences: HashMap::new(),
            live: BTreeSet::new(),
            last_discovery: None,
            packet,
            sync: sync_packet(&cid, 0, SYNC_UNIVERSE),
            sync_sequence: 0,
        })
    }

    fn target(dest: Destination, universe: u16) -> SocketAddr {
        let ip = match dest {
            Destination::Multicast => multicast_addr_for_universe(universe),
            Destination::Unicast(ip) => ip,
        };
        SocketAddr::V4(SocketAddrV4::new(ip, SACN_PORT))
    }

    fn next_sequence(&mut self, universe: u16) -> u8 {
        let s = self.sequences.entry(universe).or_insert(0);
        *s = s.wrapping_add(1);
        *s
    }

    /// Get ready to send `streams`: their sequence numbers exist (so the
    /// send allocates nothing) and streams no longer planned are ended.
    pub fn prepare(&mut self, streams: &BTreeSet<(Destination, u16)>) {
        let gone: Vec<(Destination, u16)> = self.live.difference(streams).copied().collect();
        for (dest, universe) in gone {
            self.terminate(dest, universe);
        }
        for &(dest, universe) in streams {
            self.sequences.entry(universe).or_insert(0);
            self.live.insert((dest, universe));
        }
    }

// tidy: render-begin (decisions §24: the per-frame path)
    /// Send one universe (in place: no allocation for a prepared stream).
    /// With `sync`, receivers hold it until the sync packet.
    pub fn send(
        &mut self,
        universe: u16,
        data: &[u8; 512],
        priority: u8,
        dest: Destination,
        sync: bool,
    ) -> std::io::Result<()> {
        let seq = self.next_sequence(universe);
        let p = &mut self.packet;
        p[108] = priority.min(200);
        p[109..111].copy_from_slice(&(if sync { SYNC_UNIVERSE } else { 0 }).to_be_bytes());
        p[111] = seq;
        p[112] = 0;
        p[113..115].copy_from_slice(&universe.to_be_bytes());
        p[126..638].copy_from_slice(data);
        if !self.live.contains(&(dest, universe)) {
            self.live.insert((dest, universe));
        }
        self.socket.send_to(&self.packet, Self::target(dest, universe))?;
        Ok(())
    }

    /// Send the universe synchronization packet.
    pub fn sync(&mut self, dest: Destination) -> std::io::Result<()> {
        self.sync_sequence = self.sync_sequence.wrapping_add(1);
        self.sync[44] = self.sync_sequence;
        self.socket
            .send_to(&self.sync, Self::target(dest, SYNC_UNIVERSE))?;
        Ok(())
    }

// tidy: render-end
    /// Universe discovery when it is due (every 10 s).
    pub fn discovery(&mut self) {
        if self
            .last_discovery
            .is_some_and(|t| t.elapsed() < DISCOVERY_INTERVAL)
        {
            return;
        }
        self.last_discovery = Some(Instant::now());
        let universes: Vec<u16> = self
            .live
            .iter()
            .filter(|(d, _)| *d == Destination::Multicast)
            .map(|(_, u)| *u)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !universes.is_empty() {
            let p = discovery_packet(&self.cid, &self.name, &universes);
            let _ = self
                .socket
                .send_to(&p, Self::target(Destination::Multicast, DISCOVERY_UNIVERSE));
        }
    }

    fn terminate(&mut self, dest: Destination, universe: u16) {
        for _ in 0..3 {
            let seq = self.next_sequence(universe);
            let p = data_packet(
                &self.cid,
                &self.name,
                universe,
                0,
                seq,
                OPTION_TERMINATED,
                &[0; 512],
            );
            let _ = self.socket.send_to(&p, Self::target(dest, universe));
        }
        self.live.remove(&(dest, universe));
    }
}

impl Drop for SacnSource {
    /// Stream-terminated packets for everything still live.
    fn drop(&mut self) {
        self.prepare(&BTreeSet::new());
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
