// ██╗███╗   ██╗██████╗ ██╗   ██╗████████╗
// ██║████╗  ██║██╔══██╗██║   ██║╚══██╔══╝
// ██║██╔██╗ ██║██████╔╝██║   ██║   ██║
// ██║██║╚██╗██║██╔═══╝ ██║   ██║   ██║
// ██║██║ ╚████║██║     ╚██████╔╝   ██║
// ╚═╝╚═╝  ╚═══╝╚═╝      ╚═════╝    ╚═╝
// FILE: input

//! DMX input (decisions §11 "Step 4c hardening"): what an input line takes
//! in, and how the packets of every source become one layer per (line,
//! local universe) for the merge.
//!
//! - Several sources on one protocol universe: the E1.31 rule — the highest
//!   sACN priority wins, equal priority merges HTP per channel. Art-Net has
//!   no priority (every source counts as 100), so it is HTP.
//! - A line's `target` accepts that source IP only.
//! - A source unheard for 2.5 s is lost (E1.31 network data loss); a
//!   stream-terminated packet ends it at once. Its channels fall back to
//!   our output.
//! - Each channel of a layer carries the stamp (`next_stamp`) of its last
//!   change, so inside a band the latest change wins against our sources.
//! - E1.31 sequence numbers (decisions §29 "11b hardening"): a packet 0 to
//!   −20 behind the last one of its source and universe is late or a
//!   duplicate and is dropped; a bigger jump is a restarted source and is
//!   taken. Sync packets are checked the same way per sync address.
//! - E1.31 synchronization: a universe whose packets name a sync address is
//!   held until that address's sync packet arrives; without a sync packet
//!   for 2.5 s it falls back to taking data at once.
//! - Nothing here allocates per received packet: names are fixed buffers
//!   ([`SourceName`]), the receivers fill a reused `Vec`.
//!
//! Owns:          InputPlan, Packet, SourceKey, SourceName, InputLayer, InputMerger (E1.31 priority, then HTP, 2.5 s loss).
//! Doesn't own:   the receivers (sacn, artnet), the merge per channel in the tick (lumacore-render).
//! Entry points:  InputMerger, InputPlan, InputLayer, Packet, SourceKey, SourceInfo.
//! Decisions:     §11 "Step 4c hardening".

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::line::InputPriority;
use lumacore_base::next_stamp;
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// E1.31 network data loss timeout (also the sync fallback).
pub const SOURCE_TIMEOUT: Duration = Duration::from_millis(2500);

/// A source's name without a heap string (E1.31 source names are at most
/// 64 bytes; Art-Net sources are named by their IP).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SourceName {
    bytes: [u8; 64],
    len: u8,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl SourceName {
    /// From raw bytes (cut at the first NUL and at 64 bytes, at a char edge).
    pub fn from_bytes(raw: &[u8]) -> Self {
        let raw = &raw[..raw.iter().position(|&b| b == 0).unwrap_or(raw.len()).min(64)];
        let valid = match std::str::from_utf8(raw) {
            Ok(_) => raw.len(),
            Err(e) => e.valid_up_to(),
        };
        let mut bytes = [0u8; 64];
        bytes[..valid].copy_from_slice(&raw[..valid]);
        Self { bytes, len: valid as u8 }
    }

    /// An IPv4 address as its dotted text.
    pub fn from_ip(ip: Ipv4Addr) -> Self {
        use std::io::Write;
        let mut bytes = [0u8; 64];
        let mut cursor = std::io::Cursor::new(&mut bytes[..]);
        let _ = write!(cursor, "{ip}");
        let len = cursor.position() as u8;
        Self { bytes, len }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("")
    }
}

impl std::fmt::Debug for SourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl std::fmt::Display for SourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Who sent a packet: an sACN CID, or (Art-Net) the sender's IP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SourceKey {
    Cid(Uuid),
    Ip(Ipv4Addr),
}

/// One received packet: a universe of DMX, or an E1.31 sync packet.
#[derive(Debug, Clone, Copy)]
pub struct Packet {
    pub key: SourceKey,
    pub name: SourceName,
    pub ip: Ipv4Addr,
    /// sACN priority (Art-Net: 100).
    pub priority: u8,
    /// Protocol universe (sACN universe / Art-Net Port-Address); for a sync
    /// packet its sync address.
    pub universe: u16,
    pub data: [u8; 512],
    /// sACN stream-terminated: the source ends now.
    pub terminated: bool,
    /// E1.31 sequence number (Art-Net: None, not checked).
    pub sequence: Option<u8>,
    /// E1.31 synchronization address of a data packet (0 = none).
    pub sync: u16,
    /// An E1.31 sync packet (no data): release what waits for `universe`.
    pub is_sync: bool,
}

impl Packet {
    /// A DMX data packet without sequence or sync (Art-Net, tests).
    pub fn dmx(key: SourceKey, name: SourceName, ip: Ipv4Addr, priority: u8, universe: u16, data: [u8; 512]) -> Self {
        Self {
            key,
            name,
            ip,
            priority,
            universe,
            data,
            terminated: false,
            sequence: None,
            sync: 0,
            is_sync: false,
        }
    }
}

/// E1.31 6.7.2: a sequence number 0 to −20 behind the last is out of order.
pub fn out_of_order(last: u8, new: u8) -> bool {
    let diff = new.wrapping_sub(last) as i8;
    diff <= 0 && diff > -20
}

/// What one input line takes in (from `protocol_plan::desired`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPlan {
    pub line: u32,
    /// Accept only this source; None = any.
    pub source: Option<Ipv4Addr>,
    pub band: InputPriority,
    /// (local universe, protocol universe).
    pub routes: Vec<(u32, u16)>,
}

/// One input line's DMX for one local universe, ready to merge.
#[derive(Debug, Clone, PartialEq)]
pub struct InputLayer {
    pub line: u32,
    pub universe: u32,
    pub band: InputPriority,
    pub data: [u8; 512],
    /// When each channel last changed (`next_stamp`).
    pub stamps: Arc<[u64; 512]>,
}

/// A source as seen, for status (`Root.Runtime.Network.Sources`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInfo {
    pub key: SourceKey,
    pub name: String,
    pub ip: Ipv4Addr,
    pub priority: u8,
    pub universes: Vec<u16>,
}

struct Seen {
    name: SourceName,
    ip: Ipv4Addr,
    priority: u8,
    data: [u8; 512],
    last: Instant,
    sequence: Option<u8>,
    /// Data waiting for its sync packet (`sync` address).
    held: Option<[u8; 512]>,
    sync: u16,
}

/// The sync packets heard from one source on one sync address.
struct SyncSeen {
    sequence: u8,
    last: Instant,
}

/// Collects packets and makes the layers (one per driver).
#[derive(Default)]
pub struct InputMerger {
    /// By (protocol universe, source).
    sources: HashMap<(u16, SourceKey), Seen>,
    /// By (sync address, source).
    syncs: HashMap<(u16, SourceKey), SyncSeen>,
    /// The last layer of each (line, local universe), for change stamps.
    last: BTreeMap<(u32, u32), InputLayer>,
}

impl InputMerger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take in received packets (allocates only for a source or sync
    /// address heard for the first time).
    pub fn ingest<'a>(&mut self, packets: impl IntoIterator<Item = &'a Packet>, now: Instant) {
        for p in packets {
            if p.is_sync {
                self.sync(p, now);
            } else {
                self.data(p, now);
            }
        }
    }

    fn data(&mut self, p: &Packet, now: Instant) {
        if p.terminated {
            self.sources.remove(&(p.universe, p.key));
            return;
        }
        // Wait for the sync packet only while its address is alive.
        let synced = p.sync != 0
            && self
                .syncs
                .get(&(p.sync, p.key))
                .is_some_and(|s| now.duration_since(s.last) < SOURCE_TIMEOUT);
        match self.sources.get_mut(&(p.universe, p.key)) {
            Some(seen) => {
                if let (Some(last), Some(new)) = (seen.sequence, p.sequence)
                    && out_of_order(last, new)
                {
                    return;
                }
                seen.name = p.name;
                seen.ip = p.ip;
                seen.priority = p.priority;
                seen.last = now;
                seen.sequence = p.sequence;
                seen.sync = p.sync;
                if synced {
                    seen.held = Some(p.data);
                } else {
                    seen.data = p.data;
                    seen.held = None;
                }
            }
            None => {
                self.sources.insert(
                    (p.universe, p.key),
                    Seen {
                        name: p.name,
                        ip: p.ip,
                        priority: p.priority,
                        // A new source waiting for sync starts dark.
                        data: if synced { [0; 512] } else { p.data },
                        last: now,
                        sequence: p.sequence,
                        held: synced.then_some(p.data),
                        sync: p.sync,
                    },
                );
            }
        }
    }

    fn sync(&mut self, p: &Packet, now: Instant) {
        let address = p.universe;
        let sequence = p.sequence.unwrap_or(0);
        match self.syncs.get_mut(&(address, p.key)) {
            Some(s) => {
                if out_of_order(s.sequence, sequence) {
                    return;
                }
                s.sequence = sequence;
                s.last = now;
            }
            None => {
                self.syncs.insert((address, p.key), SyncSeen { sequence, last: now });
            }
        }
        for ((_, key), seen) in self.sources.iter_mut() {
            if *key == p.key
                && seen.sync == address
                && let Some(held) = seen.held.take()
            {
                seen.data = held;
            }
        }
    }

    /// Drop lost sources and make the layers for `plan`. Returns None when
    /// nothing changed since the last call.
    pub fn layers(&mut self, plan: &[InputPlan], now: Instant) -> Option<Vec<InputLayer>> {
        self.sources
            .retain(|_, s| now.duration_since(s.last) < SOURCE_TIMEOUT);
        self.syncs
            .retain(|_, s| now.duration_since(s.last) < SOURCE_TIMEOUT);
        // A sync address gone silent: what waited for it is taken (fallback).
        for seen in self.sources.values_mut() {
            if seen.held.is_some() && !self.syncs.keys().any(|(a, _)| *a == seen.sync) {
                seen.data = seen.held.take().unwrap_or(seen.data);
            }
        }
        let mut next: BTreeMap<(u32, u32), InputLayer> = BTreeMap::new();
        let mut stamp = None;
        for line in plan {
            for &(local, wire) in &line.routes {
                let Some(data) = self.merged(wire, line.source) else {
                    continue;
                };
                let key = (line.line, local);
                let layer = match self.last.get(&key) {
                    Some(prev) if prev.data == data && prev.band == line.band => prev.clone(),
                    prev => {
                        let now_stamp = *stamp.get_or_insert_with(next_stamp);
                        let mut stamps = prev.map_or([now_stamp; 512], |p| *p.stamps);
                        if let Some(p) = prev {
                            for (i, s) in stamps.iter_mut().enumerate() {
                                if p.data[i] != data[i] {
                                    *s = now_stamp;
                                }
                            }
                        }
                        InputLayer {
                            line: line.line,
                            universe: local,
                            band: line.band,
                            data,
                            stamps: Arc::new(stamps),
                        }
                    }
                };
                next.insert(key, layer);
            }
        }
        if next == self.last {
            return None;
        }
        self.last = next;
        Some(self.last.values().cloned().collect())
    }

    /// One protocol universe merged over its sources (E1.31: highest
    /// priority, then HTP). Allocates nothing.
    fn merged(&self, wire: u16, only: Option<Ipv4Addr>) -> Option<[u8; 512]> {
        let candidates = || {
            self.sources
                .iter()
                .filter(move |((u, _), s)| *u == wire && only.is_none_or(|ip| s.ip == ip))
                .map(|(_, s)| s)
        };
        let top = candidates().map(|s| s.priority).max()?;
        let mut out = [0u8; 512];
        for s in candidates().filter(|s| s.priority == top) {
            for (o, v) in out.iter_mut().zip(s.data.iter()) {
                *o = (*o).max(*v);
            }
        }
        Some(out)
    }

    /// The sources heard right now.
    pub fn sources(&self) -> Vec<SourceInfo> {
        let mut by_key: BTreeMap<SourceKey, SourceInfo> = BTreeMap::new();
        for ((wire, key), s) in &self.sources {
            let info = by_key.entry(*key).or_insert_with(|| SourceInfo {
                key: *key,
                name: s.name.to_string(),
                ip: s.ip,
                priority: s.priority,
                universes: Vec::new(),
            });
            info.priority = info.priority.max(s.priority);
            info.universes.push(*wire);
        }
        by_key
            .into_values()
            .map(|mut s| {
                s.universes = s.universes.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
                s
            })
            .collect()
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
