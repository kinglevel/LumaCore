//! /ws/shell — WebSocket bridge to the TCP shell port.
//!
//! Acts as a TCP proxy: xterm.js connects here over WebSocket, and this handler
//! connects to localhost:shell_port over TCP, relaying packets in both directions.
//! Both directions use the `lumacore_wire` shell packets (JSON + newline), the
//! same types the station and the CLI compile (decisions §23 "Step 8").
//!
//! WebSocket framing towards the browser:
//!   Binary — terminal frames: 4-byte little-endian viewport id (0 = the
//!            untagged frame), then ANSI written straight into that xterm.js
//!   Text   — JSON control messages: `{"type":"ui","event":{…}}`,
//!            `{"type":"status","status":{…}}` (command line bar status icons),
//!            `{"type":"suggestions","seq":…,…}` (decisions §30 "12b2")
//! From the browser (Text):
//!   `{"type":"key", key, ctrl, alt, shift, repeat}` — a DOM KeyboardEvent
//!   `{"type":"focus", target:"terminal"|"panel", kind?, window?}` — UI focus changed
//!   `{"type":"live_screen", no}` — the browser's screen (view commands default to it)
//!   `{"type":"scroll", window, row}` — a window scrolled: its first visible row,
//!   kept quietly on the window (decisions §30 "12e hardening")
//!   `{"type":"tap", object, no, hold?, action?}` — a UI object was tapped (view button,
//!   pool button: `action` = the pool's tap command template)
//!   `{"type":"gesture", id, line, phase: "move"|"end", window?, label?, from?, value?}` — a
//!   continuous control's line (decisions §30 "12f"): quiet, newest move wins, one entry at the end
//!   `{"type":"command", cmd, window?}` — a command from the UI (dialog OK), echoed like
//!   typed input; `window` = the window it belongs to (`Self`, decisions §30 "12c")
//!   `{"type":"key_hold" | "key_up", key, ctrl, alt, shift, held}` — a
//!   keyboard key held / released: its bound UI key's Hold / Release /
//!   ReleaseAfterHold
//!   `{"type":"key_action", key, gesture, meow, window?}` — a UI key gesture (press,
//!   release, hold, releaseafterhold, double), Meow held or latched; `window` = the
//!   window bar it sits in (`Self`)
//!   `{"type":"dialog_answer", id, button, inputs?, toggles?, selectors?}` — the answer
//!   to one of the station's dialogs (decisions §30 "12b")
//!   `{"type":"dialog_pause", id}` — a touch on a dialog's countdown
//!   `{"type":"suggest", seq, line?, cursor?, surface}` — suggestions for a command line
//!   (no line: the session's own); answered by `{"type":"suggestions", seq, …}`
//!   `{"type":"line_edit", start, end, text, expect?}` — take a suggestion: write the
//!   session's command line
//!   `{"type":"resize", cols, rows}`
//!   `{"type":"viewports", viewports:[{view, cols, rows, layers?}…]}` — terminal windows,
//!            focused first; each gets its own frame (stacked rendering)
//!   anything else — raw xterm data (paste), parsed as VT input

use std::net::SocketAddr;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, Uri};
use axum::response::IntoResponse;
use lumacore_wire::{
    RemoteShellCode, RemoteShellKey, ShellClientPacket, ShellServerPacket, UiFocus, ViewLayers,
    Viewport,
};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::HttpState;

/// The login travels in the hello: the token (cookie / `?token=`), or — with
/// no token — `Admin` without a password, which the server only accepts while
/// Admin has none. The real client address decides whether it is a remote login.
pub async fn ws_shell(
    ws: WebSocketUpgrade,
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    uri: Uri,
) -> impl IntoResponse {
    let shell_port = state.shell_port;
    let token = crate::auth::token_from_request(&headers, &uri);
    let device = crate::auth::device_from_request(&uri);
    ws.on_upgrade(move |socket| handle_shell(socket, shell_port, token, peer, device))
}

async fn handle_shell(
    socket: WebSocket,
    shell_port: u16,
    token: Option<String>,
    peer: SocketAddr,
    device: Option<String>,
) {
    if let Err(e) = run_proxy(socket, shell_port, token, peer, device).await {
        log::warn!("ws/shell: {e}");
    }
}

async fn run_proxy(
    mut ws: WebSocket,
    shell_port: u16,
    token: Option<String>,
    peer: SocketAddr,
    device: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tcp = TcpStream::connect(format!("127.0.0.1:{shell_port}")).await?;
    // Disable Nagle so small key packets are sent immediately.
    tcp.set_nodelay(true)?;
    let (tcp_read, mut tcp_write) = tcp.into_split();
    let mut lines = BufReader::new(tcp_read).lines();

    // Send Hello — use the default Admin profile (always present in every show)
    // Use xterm.js default dimensions (80×24) so the first frame is sane;
    // the client sends a resize immediately on ws.onopen with the real size.
    let hello = ShellClientPacket::Hello {
        user: "Admin".into(),
        width: 80,
        height: 24,
        password: None,
        token,
        client_addr: Some(peer.ip().to_string()),
        protocol: lumacore_wire::CLIENT_PROTOCOL,
        device,
    };
    tcp_write.write_all(&packet_line(&hello)).await?;

    loop {
        tokio::select! {
            // TCP server → WebSocket client
            line = lines.next_line() => {
                match line? {
                    None => break,
                    Some(line) => {
                        match serde_json::from_str::<ShellServerPacket>(&line) {
                            Ok(ShellServerPacket::Frame { ansi, view }) => {
                                let mut bytes = Vec::with_capacity(4 + ansi.len());
                                bytes.extend_from_slice(&view.unwrap_or(0).to_le_bytes());
                                bytes.extend_from_slice(ansi.as_bytes());
                                if ws.send(Message::Binary(bytes.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ShellServerPacket::Ui { event }) => {
                                let msg = json!({ "type": "ui", "event": event }).to_string();
                                if ws.send(Message::Text(msg.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ShellServerPacket::Status { status }) => {
                                let msg = json!({ "type": "status", "status": status }).to_string();
                                if ws.send(Message::Text(msg.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ShellServerPacket::Suggestions { suggestions }) => {
                                let mut msg = serde_json::to_value(&*suggestions).unwrap_or_default();
                                msg["type"] = json!("suggestions");
                                if ws.send(Message::Text(msg.to_string().into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ShellServerPacket::Closed { message, exit_code }) => {
                                // Tell the page why the shell ended before closing.
                                let msg = json!({ "type": "closed", "message": message, "exit_code": exit_code })
                                    .to_string();
                                let _ = ws.send(Message::Text(msg.into())).await;
                                break;
                            }
                            Ok(ShellServerPacket::Error { message }) => {
                                // Tell the page (e.g. a refused login) before closing.
                                log::warn!("ws/shell: server error: {message}");
                                let msg = json!({ "type": "error", "message": message }).to_string();
                                let _ = ws.send(Message::Text(msg.into())).await;
                                break;
                            }
                            Err(_) => {} // ignore what this build can't read
                        }
                    }
                }
            }
            // WebSocket client → TCP server
            msg = ws.recv() => {
                let text = match msg {
                    None | Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Text(t))) => t.to_string(),
                    Some(Ok(Message::Binary(b))) => match std::str::from_utf8(&b) {
                        Ok(s) => s.to_string(),
                        Err(_) => continue,
                    },
                    Some(Ok(_)) => continue,
                    Some(Err(_)) => break,
                };

                // xterm sends resize as JSON when we attach our own resize handler;
                // raw input is forwarded as key packets.
                if text.starts_with('{')
                    && let Ok(v) = serde_json::from_str::<Value>(&text) {
                        match browser_packet(&v) {
                            Browser::Packet(p) => {
                                tcp_write.write_all(&packet_line(&p)).await?;
                                continue;
                            }
                            Browser::Skip => continue, // unmappable key
                            Browser::Raw => {}
                        }
                    }

                for key in xterm_to_keys(&text) {
                    tcp_write.write_all(&packet_line(&key)).await?;
                }
            }
        }
    }

    Ok(())
}

/// Modifier bits follow crossterm 0.28 KeyModifiers: SHIFT=1, ALT=2, CONTROL=4.
fn modifier_bits(v: &Value) -> u8 {
    let flag = |name: &str| v[name].as_bool().unwrap_or(false);
    (flag("shift") as u8) | ((flag("alt") as u8) << 1) | ((flag("ctrl") as u8) << 2)
}

/// What a browser control message becomes.
enum Browser {
    Packet(ShellClientPacket),
    /// A key the shell has no code for (modifier-only …), or a malformed message.
    Skip,
    /// Not a control message: raw xterm input.
    Raw,
}

/// One packet as a line on the shell port.
fn packet_line(packet: &ShellClientPacket) -> Vec<u8> {
    let mut line = serde_json::to_vec(packet).unwrap_or_default();
    line.push(b'\n');
    line
}

fn key(code: RemoteShellCode, modifiers: u8, repeat: Option<bool>, base: Option<char>) -> ShellClientPacket {
    ShellClientPacket::Key {
        key: RemoteShellKey {
            code,
            modifiers,
            repeat,
            base,
        },
    }
}

/// A browser control message (`{"type": …}`) as a shell packet.
fn browser_packet(v: &Value) -> Browser {
    let packet = match v["type"].as_str() {
        Some("resize") => ShellClientPacket::Resize {
            width: v["cols"].as_u64().unwrap_or(220) as u16,
            height: v["rows"].as_u64().unwrap_or(50) as u16,
        },
        Some("viewports") => ShellClientPacket::Viewports {
            viewports: v["viewports"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .filter_map(|vp| {
                            Some(Viewport {
                                view: vp["view"].as_u64().filter(|n| *n > 0)? as u32,
                                width: vp["cols"].as_u64()?.min(u16::MAX as u64) as u16,
                                height: vp["rows"].as_u64()?.min(u16::MAX as u64) as u16,
                                layers: match vp["layers"].as_str() {
                                    Some("input") => ViewLayers::Input,
                                    Some("output") => ViewLayers::Output,
                                    _ => ViewLayers::Full,
                                },
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        },
        Some("key") => match dom_key(v) {
            Some(p) => p,
            None => return Browser::Skip,
        },
        // A keyboard key held / let go (the press came as `key`).
        Some(kind @ ("key_hold" | "key_up")) => match dom_key(v) {
            Some(ShellClientPacket::Key { key }) => ShellClientPacket::KeyGesture {
                key,
                gesture: match (kind, v["held"].as_bool().unwrap_or(false)) {
                    ("key_hold", _) => "hold",
                    (_, true) => "releaseafterhold",
                    _ => "release",
                }
                .to_string(),
            },
            _ => return Browser::Skip,
        },
        Some("focus") => ShellClientPacket::Focus {
            focus: match v["target"].as_str() {
                Some("panel") => UiFocus::Panel {
                    kind: v["kind"].as_str().unwrap_or("").to_string(),
                },
                _ => UiFocus::Terminal,
            },
            window: v["window"].as_u64().map(|n| n as u32),
        },
        Some("live_screen") => ShellClientPacket::Screen {
            no: v["no"].as_u64().map(|n| n as u32),
        },
        // A window scrolled: its first visible row (§30 "12e hardening").
        Some("scroll") => match (v["window"].as_u64(), v["row"].as_f64()) {
            (Some(window), Some(row)) if row.is_finite() => ShellClientPacket::Scroll {
                window: window.min(u32::MAX as u64) as u32,
                row,
            },
            _ => return Browser::Skip,
        },
        Some("command") => match v["cmd"].as_str() {
            Some(cmd) => ShellClientPacket::Command {
                cmd: cmd.to_string(),
                // A window bar / menu / settings line names its window:
                // `Self` (decisions §30 "12c").
                window: v["window"].as_u64().map(|n| n as u32),
            },
            None => return Browser::Skip,
        },
        Some("gesture") => match (v["id"].as_str(), v["line"].as_str(), v["phase"].as_str()) {
            (Some(id), Some(line), Some(phase @ ("move" | "end"))) => ShellClientPacket::Gesture {
                id: id.to_string(),
                line: line.to_string(),
                phase: if phase == "end" { lumacore_wire::GesturePhase::End } else { lumacore_wire::GesturePhase::Move },
                window: v["window"].as_u64().map(|n| n as u32),
                label: v["label"].as_str().map(str::to_string),
                from: v["from"].as_str().map(str::to_string),
                value: v["value"].as_str().map(str::to_string),
            },
            _ => return Browser::Skip,
        },
        Some("key_action") => match (v["key"].as_str(), v["gesture"].as_str()) {
            (Some(key), Some(gesture)) => ShellClientPacket::KeyAction {
                key: key.to_string(),
                gesture: gesture.to_string(),
                meow: v["meow"].as_bool().unwrap_or(false),
                window: v["window"].as_u64().map(|n| n as u32),
            },
            _ => return Browser::Skip,
        },
        Some("dialog_answer") => {
            let mut answer: lumacore_wire::DialogAnswer = match serde_json::from_value(v.clone()) {
                Ok(a) => a,
                Err(_) => return Browser::Skip,
            };
            answer.id = v["id"].as_str().unwrap_or_default().to_string();
            ShellClientPacket::DialogAnswer { answer }
        }
        // Suggestions for the command line (decisions §30 "12b2"): no
        // `line` = the session's own line where it is now (the bottom bar).
        Some("suggest") => ShellClientPacket::Suggest {
            seq: v["seq"].as_u64().unwrap_or(0),
            line: v["line"].as_str().map(str::to_string),
            cursor: v["cursor"].as_u64().map(|c| c.min(u32::MAX as u64) as u32),
            surface: serde_json::from_value(v["surface"].clone()).unwrap_or_default(),
        },
        // Taking a suggestion writes the session's command line.
        Some("line_edit") => match (v["start"].as_u64(), v["end"].as_u64(), v["text"].as_str()) {
            (Some(start), Some(end), Some(text)) => ShellClientPacket::LineEdit {
                start: start.min(u32::MAX as u64) as u32,
                end: end.min(u32::MAX as u64) as u32,
                text: text.to_string(),
                expect: v["expect"].as_str().map(str::to_string),
            },
            _ => return Browser::Skip,
        },
        Some("dialog_pause") => match v["id"].as_str() {
            Some(id) => ShellClientPacket::DialogPause { id: id.to_string() },
            None => return Browser::Skip,
        },
        Some("tap") => match (v["object"].as_str(), v["no"].as_u64()) {
            (Some(object), Some(no)) => ShellClientPacket::Tap {
                object: object.to_string(),
                no: no as u32,
                hold: v["hold"].as_bool().unwrap_or(false),
                action: v["action"].as_str().map(str::to_string),
                window: v["window"].as_u64().map(|n| n as u32),
            },
            _ => return Browser::Skip,
        },
        _ => return Browser::Raw,
    };
    Browser::Packet(packet)
}

/// Convert a browser KeyboardEvent (`key` = DOM key name) into a key packet.
/// None for keys the shell has no code for (modifier-only etc.). `base` is the
/// letter or digit where the key sits, sent with Alt or Ctrl held (decisions
/// 0295): a Mac's Option+E is a dead key — it is that letter.
fn dom_key(v: &Value) -> Option<ShellClientPacket> {
    let base = v["base"].as_str().and_then(|b| {
        let mut c = b.chars();
        c.next().filter(|_| c.next().is_none())
    });
    let name = match v["key"].as_str().unwrap_or("") {
        "Dead" if base.is_some() => base.map(String::from).unwrap_or_default(),
        n => n.to_string(),
    };
    let name = name.as_str();
    let mut mods = modifier_bits(v);
    let code = match name {
        "Enter" => RemoteShellCode::Enter,
        "Tab" => RemoteShellCode::Tab,
        "Backspace" => RemoteShellCode::Backspace,
        "Delete" => RemoteShellCode::Delete,
        "Pause" => RemoteShellCode::Pause,
        "Escape" => RemoteShellCode::Esc,
        "Home" => RemoteShellCode::Home,
        "End" => RemoteShellCode::End,
        "PageUp" => RemoteShellCode::PageUp,
        "PageDown" => RemoteShellCode::PageDown,
        "ArrowUp" => RemoteShellCode::Up,
        "ArrowDown" => RemoteShellCode::Down,
        "ArrowLeft" => RemoteShellCode::Left,
        "ArrowRight" => RemoteShellCode::Right,
        "Insert" => RemoteShellCode::Insert,
        "PrintScreen" => RemoteShellCode::PrintScreen,
        _ if name.len() > 1 && name.starts_with('F') && name[1..].parse::<u8>().is_ok() => {
            RemoteShellCode::F(name[1..].parse::<u8>().unwrap())
        }
        _ => {
            let mut chars = name.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => {
                    // Shift is already folded into the character ("A", "!"),
                    // except for letters where crossterm also reports SHIFT.
                    if !ch.is_alphabetic() {
                        mods &= !1;
                    }
                    RemoteShellCode::Char(ch)
                }
                _ => return None,
            }
        }
    };
    Some(key(code, mods, v["repeat"].as_bool(), base))
}

/// Convert an xterm.js `onData` string into one or more key packet JSON values.
fn xterm_to_keys(input: &str) -> Vec<ShellClientPacket> {
    let mut out = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        let (code, modifiers, consumed) = parse_next_key(remaining);
        out.push(key(code, modifiers, None, None));
        remaining = &remaining[consumed..];
    }

    out
}

/// Parse the next key sequence from the front of `s`.
/// Returns (code_value, modifiers, bytes_consumed).
///
/// Modifier bits follow crossterm 0.28 KeyModifiers: SHIFT=1, ALT=2, CONTROL=4.
/// VT modifier parameter `m` maps to crossterm bits as `m - 1` (VT base is 1).
fn parse_next_key(s: &str) -> (RemoteShellCode, u8, usize) {
    let bytes = s.as_bytes();

    // SS3 sequences: \x1bO[P-S]  (F1-F4 without modifier)
    if bytes.len() >= 3 && bytes[0] == 0x1b && bytes[1] == b'O' {
        let code = match bytes[2] {
            b'P' => Some(RemoteShellCode::F(1)),
            b'Q' => Some(RemoteShellCode::F(2)),
            b'R' => Some(RemoteShellCode::F(3)),
            b'S' => Some(RemoteShellCode::F(4)),
            _ => None,
        };
        if let Some(c) = code {
            return (c, 0, 3);
        }
    }

    // CSI sequences: \x1b[ <params> <terminator>
    // Params are digits and semicolons; terminator is the first non-digit/non-semicolon.
    if bytes.len() >= 3 && bytes[0] == 0x1b && bytes[1] == b'[' {
        let mut end = 2;
        while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b';') {
            end += 1;
        }
        if end < bytes.len() {
            let terminator = bytes[end];
            let inner = &s[2..end];
            let consumed = end + 1;

            // Parse numeric params (split on ';')
            let params: Vec<u16> = inner
                .split(';')
                .filter(|p| !p.is_empty())
                .map(|p| p.parse().unwrap_or(1))
                .collect();

            let p1 = params.first().copied().unwrap_or(0);
            // VT modifier: p2 >= 1; crossterm bits = p2 - 1
            let p2 = params.get(1).copied().unwrap_or(1);
            let mods: u8 = if p2 >= 1 { (p2 - 1) as u8 } else { 0 };

            let code_opt: Option<RemoteShellCode> = match terminator {
                b'A' => Some(RemoteShellCode::Up),
                b'B' => Some(RemoteShellCode::Down),
                b'C' => Some(RemoteShellCode::Right),
                b'D' => Some(RemoteShellCode::Left),
                b'H' => Some(RemoteShellCode::Home),
                b'F' => Some(RemoteShellCode::End),
                // F1-F4 with modifier: \x1b[1;mP etc.
                b'P' if p1 == 0 || p1 == 1 => Some(RemoteShellCode::F(1)),
                b'Q' if p1 == 0 || p1 == 1 => Some(RemoteShellCode::F(2)),
                b'R' if p1 == 0 || p1 == 1 => Some(RemoteShellCode::F(3)),
                b'S' if p1 == 0 || p1 == 1 => Some(RemoteShellCode::F(4)),
                b'~' => match p1 {
                    1 | 7 => Some(RemoteShellCode::Home),
                    2 => Some(RemoteShellCode::Insert),
                    3 => Some(RemoteShellCode::Delete),
                    4 | 8 => Some(RemoteShellCode::End),
                    5 => Some(RemoteShellCode::PageUp),
                    6 => Some(RemoteShellCode::PageDown),
                    11 => Some(RemoteShellCode::F(1)),
                    12 => Some(RemoteShellCode::F(2)),
                    13 => Some(RemoteShellCode::F(3)),
                    14 => Some(RemoteShellCode::F(4)),
                    15 => Some(RemoteShellCode::F(5)),
                    17 => Some(RemoteShellCode::F(6)),
                    18 => Some(RemoteShellCode::F(7)),
                    19 => Some(RemoteShellCode::F(8)),
                    20 => Some(RemoteShellCode::F(9)),
                    21 => Some(RemoteShellCode::F(10)),
                    23 => Some(RemoteShellCode::F(11)),
                    24 => Some(RemoteShellCode::F(12)),
                    _ => None,
                },
                _ => None,
            };

            if let Some(code) = code_opt {
                return (code, mods, consumed);
            }
        }
    }

    // Single-byte cases
    let first = bytes[0];
    match first {
        0x0d | 0x0a => (RemoteShellCode::Enter, 0, 1),
        0x7f => (RemoteShellCode::Backspace, 0, 1),
        0x09 => (RemoteShellCode::Tab, 0, 1),
        // Bare ESC or unrecognised escape sequence — consume only the ESC byte
        0x1b => (RemoteShellCode::Esc, 0, 1),
        // Ctrl+A..Ctrl+Z (0x01..0x1a): crossterm CONTROL = 4
        b @ 0x01..=0x1a => {
            let ch = char::from(b + 0x60);
            (RemoteShellCode::Char(ch), 4u8, 1)
        }
        0x1c..=0x1f => (RemoteShellCode::Esc, 0, 1),
        _ => {
            let ch = s.chars().next().unwrap_or('\0');
            let len = ch.len_utf8();
            (RemoteShellCode::Char(ch), 0u8, len)
        }
    }
}

#[cfg(test)]
mod key_gesture_tests {
    use super::*;

    fn gesture(v: Value) -> Option<String> {
        match browser_packet(&v) {
            Browser::Packet(ShellClientPacket::KeyGesture { gesture, .. }) => Some(gesture),
            _ => None,
        }
    }

    #[test]
    fn keyboard_hold_and_release_become_key_gestures() {
        use serde_json::json;
        assert_eq!(gesture(json!({"type": "key_hold", "key": "F5"})).as_deref(), Some("hold"));
        assert_eq!(gesture(json!({"type": "key_up", "key": "F5"})).as_deref(), Some("release"));
        assert_eq!(
            gesture(json!({"type": "key_up", "key": "F5", "held": true})).as_deref(),
            Some("releaseafterhold")
        );
        assert_eq!(gesture(json!({"type": "key_up", "key": "Shift"})), None);
    }
}
