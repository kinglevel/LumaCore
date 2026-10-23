// ██╗     ██╗███████╗████████╗███████╗███╗   ██╗███████╗██████╗
// ██║     ██║██╔════╝╚══██╔══╝██╔════╝████╗  ██║██╔════╝██╔══██╗
// ██║     ██║███████╗   ██║   █████╗  ██╔██╗ ██║█████╗  ██████╔╝
// ██║     ██║╚════██║   ██║   ██╔══╝  ██║╚██╗██║██╔══╝  ██╔══██╗
// ███████╗██║███████║   ██║   ███████╗██║ ╚████║███████╗██║  ██║
// ╚══════╝╚═╝╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝
// FILE: listener

//! OSC input lines (decisions §11): one UDP listener per enabled OSC input
//! line, bound to its network's NIC, and — like every input — only on the
//! station that sends (standalone or the LumaSession master;
//! `protocol_plan::silence`), so a broadcast message runs once. Commands run
//! as the line's user.
//!
//! OSC addresses:
//! - `/lumacore/cmd <string>` → the command line
//! - `/lumacore/go`, `/lumacore/back`, `/lumacore/stop` → `Go`, `Go-`, `Stop`
//!   as the line's user
//! - `/lumacore/fixture/<id>/<attr> <float>` → decoded
//!   (`InboundEvent::AttributeSet`) but not built yet (planned): the station
//!   says so once in the System Monitor
//!
//! Owns:          OscListener (one input line's UDP listener), the OSC packet decoder, what a message becomes as the line's user, open (the row's opener).
//! Doesn't own:   which lines listen (protocol_plan, lumacore-lines), running the command (the state thread).
//! Entry points:  OscListener::start, open, parse_osc_packet.
//! Decisions:     §11, §31 "OSC line users", §34 22.

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crossbeam_channel::Sender;
use lumacore_base::{InboundEvent, TransportAction};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A running OSC input line.
pub struct OscListener {
    pub bind: Ipv4Addr,
    pub port: u16,
    pub user: String,
    shutdown: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl OscListener {
    /// Bind `bind:port` (an error is the caller's to report) and listen.
    pub fn start(
        bind: Ipv4Addr,
        port: u16,
        user: &str,
        tx: Sender<InboundEvent>,
        health: Option<Arc<lumacore_tree::runtime::ThreadHealth>>,
    ) -> Result<Self, String> {
        let addr = SocketAddr::from((bind, port));
        let socket = UdpSocket::bind(addr).map_err(|e| format!("OSC {addr}: {e}"))?;
        socket
            .set_read_timeout(Some(std::time::Duration::from_millis(100)))
            .map_err(|e| e.to_string())?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let flag = shutdown.clone();
        let line_user = user.to_string();
        let thread = thread::Builder::new()
            .name(format!("osc-{port}"))
            .spawn(move || {
                let _alive = health.clone().map(lumacore_tree::runtime::ThreadGuard);
                osc_listener_loop(socket, &line_user, tx, flag, health);
            })
            .map_err(|e| format!("failed to spawn OSC thread: {e}"))?;
        Ok(Self {
            bind,
            port,
            user: user.to_string(),
            shutdown,
            thread: Some(thread),
        })
    }

    /// Ask the thread to end (within ~100 ms); returns it to join.
    pub fn stop(&mut self) -> Option<thread::JoinHandle<()>> {
        self.shutdown.store(true, Ordering::Relaxed);
        self.thread.take()
    }
}

impl lumacore_protocols::ControlListener for OscListener {
    fn stop(&mut self) -> Option<thread::JoinHandle<()>> {
        OscListener::stop(self)
    }
}

/// The row's opener: an OSC listener for one input line.
pub fn open(start: lumacore_protocols::ControlStart) -> Result<Box<dyn lumacore_protocols::ControlListener>, String> {
    let listener = OscListener::start(start.bind, start.port, &start.user, start.inbound, start.health)?;
    Ok(Box::new(listener))
}

/// What an OSC event becomes as the line's user.
fn as_user(event: InboundEvent, user: &str) -> InboundEvent {
    let command = match event {
        InboundEvent::Command(c) => c,
        InboundEvent::TransportControl(TransportAction::Go) => "Go".to_string(),
        InboundEvent::TransportControl(TransportAction::Back) => "Go-".to_string(),
        InboundEvent::TransportControl(TransportAction::Stop) => "Stop".to_string(),
        other => return other,
    };
    InboundEvent::UserCommand {
        user: user.to_string(),
        command,
    }
}

/// Main OSC listener loop.
///
/// The socket has a 100ms read timeout so the shutdown flag
/// is checked regularly without burning CPU.
fn osc_listener_loop(
    socket: UdpSocket,
    user: &str,
    tx: Sender<InboundEvent>,
    shutdown: Arc<AtomicBool>,
    health: Option<Arc<lumacore_tree::runtime::ThreadHealth>>,
) {
    let mut buf = [0u8; 4096];

    while !shutdown.load(Ordering::Relaxed) {
        let n = match socket.recv_from(&mut buf) {
            Ok((n, _addr)) => {
                if let Some(h) = &health {
                    h.tick(0);
                }
                n
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                if let Some(h) = &health {
                    h.heartbeat();
                }
                continue;
            }
            Err(e) => {
                log::warn!("osc engine: recv error: {e}");
                continue;
            }
        };

        if let Some(event) = parse_osc_packet(&buf[..n])
            && tx.try_send(as_user(event, user)).is_err()
        {
            if let Some(h) = &health {
                h.refused();
            }
            log::warn!("osc engine: inbound event channel full, dropping message");
        }
    }

    log::info!("osc engine: listener loop exited");
}

/// Parse a raw OSC packet into an InboundEvent.
///
/// OSC message format (simplified):
/// - Address pattern: null-terminated, padded to 4 bytes
/// - Type tag string: ",<types>\0", padded to 4 bytes
/// - Arguments: each padded to 4 bytes
///
/// We support a minimal subset sufficient for Companion integration.
pub fn parse_osc_packet(data: &[u8]) -> Option<InboundEvent> {
    // Extract the address string (null-terminated, 4-byte aligned)
    let addr_end = data.iter().position(|&b| b == 0)?;
    let address = std::str::from_utf8(&data[..addr_end]).ok()?;

    // Skip past the address (4-byte aligned)
    let addr_padded = (addr_end + 4) & !3;
    if addr_padded >= data.len() {
        // No type tag — handle address-only messages for transport
        return match address {
            "/lumacore/go" => Some(InboundEvent::TransportControl(TransportAction::Go)),
            "/lumacore/back" => Some(InboundEvent::TransportControl(TransportAction::Back)),
            "/lumacore/stop" => Some(InboundEvent::TransportControl(TransportAction::Stop)),
            _ => None,
        };
    }

    // Parse type tag string
    let type_tag_start = addr_padded;
    if data[type_tag_start] != b',' {
        // Transport commands without arguments
        return match address {
            "/lumacore/go" => Some(InboundEvent::TransportControl(TransportAction::Go)),
            "/lumacore/back" => Some(InboundEvent::TransportControl(TransportAction::Back)),
            "/lumacore/stop" => Some(InboundEvent::TransportControl(TransportAction::Stop)),
            _ => None,
        };
    }

    let type_tag_end = data[type_tag_start..].iter().position(|&b| b == 0)? + type_tag_start;
    let type_tags = std::str::from_utf8(&data[type_tag_start + 1..type_tag_end]).ok()?;
    let args_start = (type_tag_end + 4) & !3;

    // /lumacore/cmd <string> → Command
    if address == "/lumacore/cmd" && type_tags.starts_with('s') {
        let string_val = read_osc_string(data, args_start)?;
        return Some(InboundEvent::Command(string_val));
    }

    // /lumacore/fixture/<id>/<attr> <float> → AttributeSet
    if let Some(rest) = address.strip_prefix("/lumacore/fixture/") {
        let (id_str, attr) = rest.split_once('/')?;

        let fixture_id: u32 = id_str.parse().ok()?;

        if type_tags.starts_with('f') {
            let value = read_osc_float(data, args_start)?;
            return Some(InboundEvent::AttributeSet {
                fixture_id,
                attribute: attr.to_string(),
                value: value as f64,
            });
        }
    }

    // Transport controls
    match address {
        "/lumacore/go" => Some(InboundEvent::TransportControl(TransportAction::Go)),
        "/lumacore/back" => Some(InboundEvent::TransportControl(TransportAction::Back)),
        "/lumacore/stop" => Some(InboundEvent::TransportControl(TransportAction::Stop)),
        _ => {
            log::debug!("osc engine: unhandled address '{address}'");
            None
        }
    }
}

/// Read a null-terminated, 4-byte-aligned OSC string from a buffer.
fn read_osc_string(data: &[u8], offset: usize) -> Option<String> {
    if offset >= data.len() {
        return None;
    }
    let end = data[offset..].iter().position(|&b| b == 0)? + offset;
    std::str::from_utf8(&data[offset..end])
        .ok()
        .map(|s| s.to_string())
}

/// Read a big-endian f32 from the buffer at the given offset.
fn read_osc_float(data: &[u8], offset: usize) -> Option<f32> {
    if offset + 4 > data.len() {
        return None;
    }
    let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
    Some(f32::from_be_bytes(bytes))
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
