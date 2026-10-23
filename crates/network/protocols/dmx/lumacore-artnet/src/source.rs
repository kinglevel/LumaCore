// ███████╗ ██████╗ ██╗   ██╗██████╗  ██████╗███████╗
// ██╔════╝██╔═══██╗██║   ██║██╔══██╗██╔════╝██╔════╝
// ███████╗██║   ██║██║   ██║██████╔╝██║     █████╗
// ╚════██║██║   ██║██║   ██║██╔══██╗██║     ██╔══╝
// ███████║╚██████╔╝╚██████╔╝██║  ██║╚██████╗███████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚══════╝
// FILE: source

//! Art-Net on one network interface (decisions §11 "Step 4b/4c hardening"):
//!
//! - Output: ArtDmx (OpCode 0x5000) to the nodes that report a Port-Address
//!   in their ArtPollReply, else to the network's directed broadcast.
//!   ArtPoll goes out every 3 s.
//! - Input: ArtDmx from the network becomes `Packet`s; while input lines
//!   exist, ArtPoll from a controller is answered with an ArtPollReply
//!   listing their Port-Addresses, so guest desks in ArtPoll mode find us.
//!   Our own packets (any of this machine's addresses) are ignored.
//! - A node's ArtPollReply `RefreshRate` caps the rate of lines sending to
//!   it; a line with `Sync` sends ArtSync after its universes (decisions §24
//!   "Step 9 hardening"). ArtDmx is rewritten in one preallocated packet.
//!
//! Owns:          ArtNetSource (send ArtDmx / ArtSync, ArtPoll, hear ArtDmx and ArtPollReply), ArtNetNode, the packet builders and parsers.
//! Doesn't own:   when a line sends (lumacore_lines::lines), the input merge (lumacore_protocols::input).
//! Entry points:  ArtNetSource, ArtNetNode, artdmx_packet, parse_artdmx, parse_poll_reply, poll_reply_packets.
//! Decisions:     §11, §24 "Step 9 hardening".

use std::collections::{BTreeMap, HashMap};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};

use crate::ARTNET_PORT;
use lumacore_protocols::DmxError;
use lumacore_protocols::input::{Packet, SourceKey, SourceName};
use lumacore_net::nics::{self, Nic};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Art-Net protocol ID (ASCII "Art-Net\0").
const ARTNET_ID: [u8; 8] = *b"Art-Net\0";
const OP_POLL: u16 = 0x2000;
const OP_POLL_REPLY: u16 = 0x2100;
const OP_DMX: u16 = 0x5000;
const OP_SYNC: u16 = 0x5200;
const PROTOCOL_VERSION: [u8; 2] = [0x00, 0x0E];
/// How often nodes are polled, and when an unheard node is dropped.
pub const POLL_INTERVAL: Duration = Duration::from_secs(3);
const NODE_TIMEOUT: Duration = Duration::from_secs(10);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// An ArtDmx packet for a 15-bit Port-Address.
pub fn artdmx_packet(port_address: u16, sequence: u8, data: &[u8; 512]) -> Vec<u8> {
    let mut p = Vec::with_capacity(18 + 512);
    p.extend_from_slice(&ARTNET_ID);
    p.extend_from_slice(&OP_DMX.to_le_bytes());
    p.extend_from_slice(&PROTOCOL_VERSION);
    p.push(sequence);
    p.push(0); // physical port
    p.push((port_address & 0xFF) as u8); // SubUni: SubNet << 4 | Universe
    p.push(((port_address >> 8) & 0x7F) as u8); // Net
    p.extend_from_slice(&512u16.to_be_bytes());
    p.extend_from_slice(data);
    p
}

/// An ArtSync: nodes output what they buffered.
pub fn artsync_packet() -> [u8; 14] {
    let mut p = [0u8; 14];
    p[..8].copy_from_slice(&ARTNET_ID);
    p[8..10].copy_from_slice(&OP_SYNC.to_le_bytes());
    p[10..12].copy_from_slice(&PROTOCOL_VERSION);
    p
}

/// An ArtPoll asking nodes to reply (and to reply on change).
pub fn artpoll_packet() -> Vec<u8> {
    let mut p = Vec::with_capacity(14);
    p.extend_from_slice(&ARTNET_ID);
    p.extend_from_slice(&OP_POLL.to_le_bytes());
    p.extend_from_slice(&PROTOCOL_VERSION);
    p.push(0x02); // Flags: send ArtPollReply on change
    p.push(0x00); // DiagPriority
    p
}

/// An ArtDmx packet as received: (Port-Address, 512 slots).
pub fn parse_artdmx(p: &[u8]) -> Option<(u16, [u8; 512])> {
    if p.len() < 18 || p[..8] != ARTNET_ID || u16::from_le_bytes([p[8], p[9]]) != OP_DMX {
        return None;
    }
    let port_address = u16::from(p[14]) | (u16::from(p[15] & 0x7F) << 8);
    let len = usize::from(u16::from_be_bytes([p[16], p[17]]))
        .min(512)
        .min(p.len() - 18);
    let mut data = [0u8; 512];
    data[..len].copy_from_slice(&p[18..18 + len]);
    Some((port_address, data))
}

fn is_poll(p: &[u8]) -> bool {
    p.len() >= 12 && p[..8] == ARTNET_ID && u16::from_le_bytes([p[8], p[9]]) == OP_POLL
}

/// ArtPollReplies announcing `inputs` as ports that take DMX from the
/// network: up to four per reply, one reply (BindIndex 1, 2, …) per
/// Net/SubNet group.
pub fn poll_reply_packets(ip: Ipv4Addr, name: &str, inputs: &[u16]) -> Vec<Vec<u8>> {
    let mut groups: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
    for &pa in inputs {
        groups.entry(pa >> 4).or_default().push(pa);
    }
    let chunks: Vec<(u16, Vec<u16>)> = groups
        .into_iter()
        .flat_map(|(group, pas)| {
            pas.chunks(4)
                .map(|c| (group, c.to_vec()))
                .collect::<Vec<_>>()
        })
        .collect();
    chunks
        .into_iter()
        .enumerate()
        .map(|(i, (group, ports))| {
            let mut p = vec![0u8; 239];
            p[..8].copy_from_slice(&ARTNET_ID);
            p[8..10].copy_from_slice(&OP_POLL_REPLY.to_le_bytes());
            p[10..14].copy_from_slice(&ip.octets());
            p[14..16].copy_from_slice(&ARTNET_PORT.to_le_bytes());
            p[18] = ((group >> 4) & 0x7F) as u8; // Net
            p[19] = (group & 0x0F) as u8; // SubNet
            p[20..22].copy_from_slice(&0x00FFu16.to_be_bytes()); // OEM unknown
            let short = name.as_bytes();
            let n = short.len().min(17);
            p[26..26 + n].copy_from_slice(&short[..n]);
            let n = short.len().min(63);
            p[44..44 + n].copy_from_slice(&short[..n]);
            p[173] = ports.len() as u8;
            for (k, pa) in ports.iter().enumerate() {
                p[174 + k] = 0x80; // outputs DMX taken from the network
                p[182 + k] = 0x80; // data is being output
                p[190 + k] = (pa & 0x0F) as u8;
            }
            p[200] = 0x00; // StNode
            p[207..211].copy_from_slice(&ip.octets());
            p[211] = (i + 1) as u8; // BindIndex
            p[212] = 0x08; // 15-bit Port-Address
            p
        })
        .collect()
}

/// Next ArtDmx sequence number: 1..=255, wrapping back to 1 (0 = disabled).
fn next_sequence(sequence: u8) -> u8 {
    if sequence == u8::MAX { 1 } else { sequence + 1 }
}

/// A node that answered ArtPoll.
#[derive(Debug, Clone, PartialEq)]
pub struct ArtNetNode {
    pub ip: Ipv4Addr,
    pub short_name: String,
    /// The Port-Addresses its output ports take.
    pub outputs: Vec<u16>,
    /// The most frames per second it takes (ArtPollReply RefreshRate;
    /// 0–44 reported = DMX512's 44). None in older replies.
    pub refresh_hz: Option<u16>,
    /// Which of the node's binds sent this reply (Art-Net 4 BindIndex,
    /// byte 211; 1 = the root device, also when the reply is older). A node
    /// with more than four ports answers once per bind (R1-53).
    pub bind_index: u8,
}

/// Read an ArtPollReply (Art-Net 4 layout).
pub fn parse_poll_reply(p: &[u8]) -> Option<ArtNetNode> {
    if p.len() < 194 || p[..8] != ARTNET_ID || u16::from_le_bytes([p[8], p[9]]) != OP_POLL_REPLY {
        return None;
    }
    let ip = Ipv4Addr::new(p[10], p[11], p[12], p[13]);
    let net = u16::from(p[18] & 0x7F);
    let sub = u16::from(p[19] & 0x0F);
    let short_name = String::from_utf8_lossy(&p[26..44])
        .trim_end_matches('\0')
        .to_string();
    let ports = usize::from(p[173]).min(4);
    let outputs = (0..ports)
        .filter(|&i| p[174 + i] & 0x80 != 0)
        .map(|i| (net << 8) | (sub << 4) | u16::from(p[190 + i] & 0x0F))
        .collect();
    // RefreshRateHi/Lo (Art-Net 4) after Status3, DefaultRespUID, User.
    let refresh_hz = (p.len() >= 228).then(|| {
        let hz = u16::from_be_bytes([p[226], p[227]]);
        if hz <= 44 { 44 } else { hz }
    });
    let bind_index = p.get(211).copied().filter(|&b| b != 0).unwrap_or(1);
    Some(ArtNetNode {
        ip,
        short_name,
        outputs,
        refresh_hz,
        bind_index,
    })
}

/// Art-Net on one interface: sends and receives on one socket.
pub struct ArtNetSource {
    socket: UdpSocket,
    nic: Nic,
    /// Pinned to the NIC (else packets are filtered by its subnet).
    pinned: bool,
    /// This machine's addresses: our own packets are ignored.
    own: Vec<Ipv4Addr>,
    sequences: HashMap<(Ipv4Addr, u16), u8>,
    /// Every bind of every node, by (IP, BindIndex).
    nodes: HashMap<(Ipv4Addr, u8), (ArtNetNode, Instant)>,
    last_poll: Option<Instant>,
    /// Whether packets can be heard (port 6454 could be bound).
    pub listening: bool,
    /// The ArtDmx packet, rewritten in place for every send.
    packet: [u8; 530],
}

impl ArtNetSource {
    /// Bind port 6454 on `nic` (shared with other programs). If it can't be
    /// bound the source still sends, to the broadcast only, and hears
    /// nothing.
    pub fn new(nic: &Nic) -> Result<Self, DmxError> {
        let (socket, pinned, listening) = match nics::listen(nic, ARTNET_PORT) {
            Ok((s, pinned)) => (s, pinned, true),
            Err(_) => {
                let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
                s.set_broadcast(true)?;
                s.bind(&SocketAddr::V4(SocketAddrV4::new(nic.ip, 0)).into())?;
                s.set_nonblocking(true)?;
                (s.into(), true, false)
            }
        };
        let mut own: Vec<Ipv4Addr> = nics::list().into_iter().map(|n| n.ip).collect();
        own.push(nic.ip);
        Ok(Self {
            socket,
            nic: nic.clone(),
            pinned,
            own,
            sequences: HashMap::new(),
            nodes: HashMap::new(),
            last_poll: None,
            listening,
            packet: artdmx_packet(0, 0, &[0; 512])
                .try_into()
                .expect("ArtDmx is 530 bytes"),
        })
    }

    /// Poll when due; read replies, DMX and polls (answered when `inputs`
    /// isn't empty); drop silent nodes. Appends the DMX received to `packets`
    /// (a reused buffer: nothing is allocated per DMX packet).
    pub fn poll(&mut self, inputs: &[u16], name: &str, packets: &mut Vec<Packet>) {
        if self.last_poll.is_none_or(|t| t.elapsed() >= POLL_INTERVAL) {
            self.last_poll = Some(Instant::now());
            let _ = self.socket.send_to(
                &artpoll_packet(),
                SocketAddrV4::new(self.nic.broadcast, ARTNET_PORT),
            );
        }
        let mut buf = [0u8; 600];
        while let Ok((n, from)) = self.socket.recv_from(&mut buf) {
            let SocketAddr::V4(from) = from else {
                continue;
            };
            let ip = *from.ip();
            if self.own.contains(&ip) || (!self.pinned && !self.nic.on_subnet(ip)) {
                continue;
            }
            let p = &buf[..n];
            if let Some(node) = parse_poll_reply(p) {
                self.note_node(node);
            } else if let Some((universe, data)) = parse_artdmx(p) {
                packets.push(Packet::dmx(SourceKey::Ip(ip), SourceName::from_ip(ip), ip, 100, universe, data));
            } else if is_poll(p) && !inputs.is_empty() {
                for reply in poll_reply_packets(self.nic.ip, name, inputs) {
                    let _ = self.socket.send_to(&reply, from);
                }
            }
        }
        self.nodes
            .retain(|_, (_, seen)| seen.elapsed() < NODE_TIMEOUT);
    }

    /// Whether the shared port-6454 socket is pinned to the NIC (when not,
    /// unicast may go to another program on the port; decisions §11).
    pub fn pinned(&self) -> bool {
        self.pinned
    }

    /// Keep a node's reply (one entry per bind).
    fn note_node(&mut self, node: ArtNetNode) {
        self.nodes
            .insert((node.ip, node.bind_index), (node, Instant::now()));
    }

    /// The nodes currently answering, one per IP: the ports of every bind
    /// together, named by the lowest bind.
    pub fn nodes(&self) -> Vec<ArtNetNode> {
        let mut binds: Vec<&ArtNetNode> = self.nodes.values().map(|(n, _)| n).collect();
        binds.sort_by_key(|n| (n.ip, n.bind_index));
        let mut nodes: Vec<ArtNetNode> = Vec::new();
        for b in binds {
            match nodes.last_mut() {
                Some(n) if n.ip == b.ip => {
                    n.outputs.extend(&b.outputs);
                    n.refresh_hz = match (n.refresh_hz, b.refresh_hz) {
                        (Some(a), Some(c)) => Some(a.min(c)),
                        (a, c) => a.or(c),
                    };
                }
                _ => nodes.push(b.clone()),
            }
        }
        for n in &mut nodes {
            n.outputs.sort_unstable();
            n.outputs.dedup();
        }
        nodes
    }

    /// Where a Port-Address goes: `unicast`, else every node that outputs
    /// it, else the directed broadcast. Worked out when the plan or the
    /// nodes change, never while sending.
    pub fn targets(&self, port_address: u16, unicast: Option<Ipv4Addr>) -> Vec<Ipv4Addr> {
        if let Some(ip) = unicast {
            return vec![ip];
        }
        let mut nodes: Vec<Ipv4Addr> = self
            .nodes
            .values()
            .filter(|(n, _)| n.outputs.contains(&port_address))
            .map(|(n, _)| n.ip)
            .collect();
        // Two binds of one node can output the same Port-Address.
        nodes.sort();
        nodes.dedup();
        if nodes.is_empty() {
            vec![self.nic.broadcast]
        } else {
            nodes
        }
    }

    /// Make sure every (target, Port-Address) has a sequence number, so the
    /// send allocates nothing.
    pub fn prepare(&mut self, streams: impl Iterator<Item = (Ipv4Addr, u16)>) {
        for key in streams {
            self.sequences.entry(key).or_insert(0);
        }
    }

// tidy: render-begin (decisions §24: the per-frame path)
    /// Send one Port-Address to `targets` (in place). Returns the packets
    /// sent. Every target is tried: one that fails (unreachable, no route)
    /// never starves the rest; the first error is returned after (R1-53).
    pub fn send(
        &mut self,
        port_address: u16,
        data: &[u8; 512],
        targets: &[Ipv4Addr],
    ) -> std::io::Result<u64> {
        self.packet[14] = (port_address & 0xFF) as u8;
        self.packet[15] = ((port_address >> 8) & 0x7F) as u8;
        self.packet[18..].copy_from_slice(data);
        let mut sent = 0;
        let mut first_error = None;
        for &ip in targets {
            let seq = match self.sequences.get_mut(&(ip, port_address)) {
                Some(s) => s,
                None => self.sequences.entry((ip, port_address)).or_insert(0),
            };
            *seq = next_sequence(*seq);
            self.packet[12] = *seq;
            match self
                .socket
                .send_to(&self.packet, SocketAddrV4::new(ip, ARTNET_PORT))
            {
                Ok(_) => sent += 1,
                Err(e) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
        }
        match first_error {
            Some(e) => Err(e),
            None => Ok(sent),
        }
    }

    /// ArtSync to the directed broadcast.
    pub fn sync(&mut self) -> std::io::Result<()> {
        self.socket.send_to(
            &artsync_packet(),
            SocketAddrV4::new(self.nic.broadcast, ARTNET_PORT),
        )?;
        Ok(())
    }
// tidy: render-end
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
