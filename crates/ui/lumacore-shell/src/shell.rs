// ███████╗██╗  ██╗███████╗██╗     ██╗
// ██╔════╝██║  ██║██╔════╝██║     ██║
// ███████╗███████║█████╗  ██║     ██║
// ╚════██║██╔══██║██╔══╝  ██║     ██║
// ███████║██║  ██║███████╗███████╗███████╗
// ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝
// FILE: shell

//! The TCP shell port (decisions §23 "Step 8"): one thread per client. It reads the client's
//! packets, runs every key, line, gesture, tap and answer as a job on the state thread (in typed
//! order, never dropped while busy, 30 s at most each), and sends back the frames and status the
//! terminal rendered for that client.
//!
//! Owns:          the shell port's client thread: hello, login and attach, the monitor client, reading packets and queuing keys in order, rendering and sending frames, the waits (SHELL_WAIT, SUGGEST_WAIT), the packet framing.
//! Doesn't own:   the listener that accepts clients (the server's spawn), the terminals (lumacore-terminal), the state thread (lumacore-lanes).
//! Entry points:  handle_shell_client, read_shell_packets, send_shell_packet, SHELL_WAIT, SUGGEST_WAIT.
//! Decisions:     §23 "Step 8", §30 "12b", R1-67, Q40, §34 24.2.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use lumacore_lanes::{CallError, StateHandle};
use lumacore_sysmon::sysmon_note;
use lumacore_wire::{ShellClientPacket, ShellServerPacket};

use crate::host::ShellHost;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How long a shell's line, key or render waits for the state thread before it answers "busy"
/// (R1-67, Q40).
pub const SHELL_WAIT: Duration = Duration::from_secs(30);
/// Suggestions wait at most this long; a busy station gives none (merge of bugfixes, decisions §30).
pub const SUGGEST_WAIT: Duration = Duration::from_secs(1);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// What a shell client does after a state call failed: the server is going
/// away (end the session), or one job panicked (logged; keep going).
fn call_failed<P: std::fmt::Display>(addr: SocketAddr, err: CallError<P>) -> bool {
    match err {
        CallError::Stopped => false,
        err @ (CallError::Panicked(_) | CallError::Reentrant | CallError::Busy { .. } | CallError::Full) => {
            log::error!("shell client {addr}: {err}");
            true
        }
    }
}

/// Answers one `Suggest` for the client's line (best effort: a busy station
/// gives none, the next keystroke asks again). False when the client is gone.
#[allow(clippy::too_many_arguments)]
fn answer_suggest<C: ShellHost>(
    state: &StateHandle<C>,
    writer: &mut TcpStream,
    addr: SocketAddr,
    client_id: uuid::Uuid,
    seq: u64,
    line: Option<String>,
    cursor: Option<u32>,
    surface: lumacore_wire::Surface,
) -> bool {
    let reply = state.call_within(SUGGEST_WAIT, move |ctx| {
        let (sessions, state) = ctx.split();
        sessions.suggest_for_client(client_id, state, seq, line, cursor, surface)
    });
    match reply {
        Err(CallError::Busy { .. }) => true,
        Ok(Ok(suggestions)) => {
            let _ = send_shell_packet(writer, &ShellServerPacket::Suggestions { suggestions: Box::new(suggestions) });
            true
        }
        Ok(Err(_)) => true,
        Err(err) => call_failed(addr, err),
    }
}

/// UI events back to the client.
fn send_ui(writer: &mut TcpStream, events: Vec<lumacore_wire::UiEvent>) {
    for event in events {
        let _ = send_shell_packet(writer, &ShellServerPacket::Ui { event });
    }
}

/// A rendered pass for one client: what the state thread decided, sent by the
/// client's own thread.
struct RenderPass {
    /// `Login` moved the client: (new session, new user, token).
    switched: Option<(uuid::Uuid, String, Option<String>)>,
    frames: Vec<(Option<u32>, lumacore_terminal::repl::RenderedShellFrame)>,
    status: lumacore_wire::Status,
    /// The session ended: (message, exit code). Already detached.
    close: Option<(String, i32)>,
    /// The client's user was deleted (decisions §30 "12c2"): it is logged
    /// out (a `LoggedOut` UI event, then `Closed`).
    logged_out: bool,
}

pub fn handle_shell_client<C: ShellHost>(stream: TcpStream, addr: SocketAddr, state: StateHandle<C>) {
    let mut reader = match stream.try_clone() {
        Ok(clone) => clone,
        Err(err) => {
            log::warn!("shell client {addr} clone failed: {err}");
            return;
        }
    };
    let mut writer = stream;
    let _ = reader.set_nonblocking(true);
    let _ = writer.set_nodelay(true);
    let _ = writer.set_read_timeout(Some(Duration::from_millis(100)));
    let mut read_buf = Vec::new();

    enum HelloResult {
        Shell {
            user: String,
            width: u16,
            height: u16,
            password: Option<String>,
            token: Option<String>,
            remote: bool,
            from: String,
            web: bool,
            device: Option<String>,
        },
        Monitor,
    }

    let hello_deadline = std::time::Instant::now() + Duration::from_secs(2);
    // Packets that arrived in the same read as the hello (a fast client sends
    // its viewports / screen right after it): handled after login.
    let mut early: Vec<ShellClientPacket> = Vec::new();
    let hello = loop {
        match read_shell_packets(&mut reader, &mut read_buf) {
            Ok(packets) => {
                let mut packets = packets.into_iter();
                if let Some(packet) = packets.next() {
                    early.extend(packets);
                    match packet {
                        ShellClientPacket::Hello {
                            user,
                            width,
                            height,
                            password,
                            token,
                            client_addr,
                            protocol,
                            device,
                        } => {
                            if protocol != lumacore_wire::CLIENT_PROTOCOL {
                                refuse_protocol(&mut writer, protocol);
                                return;
                            }
                            // The HTTP bridge (on this machine) passes the real
                            // client address; anyone else's claim is ignored and
                            // the TCP peer counts (decisions §19).
                            let bridged = bridged_client(addr.ip(), client_addr.as_deref());
                            let ip = bridged.unwrap_or(addr.ip());
                            let remote = !ip.is_loopback();
                            // For sysmon: who is really connecting, and through what.
                            let from = if bridged.is_some() {
                                format!("{ip} (web)")
                            } else {
                                addr.to_string()
                            };
                            let web = bridged.is_some();
                            break HelloResult::Shell {
                                user,
                                width,
                                height,
                                password,
                                token,
                                remote,
                                from,
                                web,
                                device,
                            };
                        }
                        ShellClientPacket::MonitorHello { protocol } => {
                            if protocol != lumacore_wire::CLIENT_PROTOCOL {
                                refuse_protocol(&mut writer, protocol);
                                return;
                            }
                            break HelloResult::Monitor;
                        }
                        _ => {
                            let _ = send_shell_packet(
                                &mut writer,
                                &ShellServerPacket::Error {
                                    message: "expected shell hello".to_string(),
                                },
                            );
                            return;
                        }
                    }
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut => {}
            Err(err) => {
                let _ = send_shell_packet(
                    &mut writer,
                    &ShellServerPacket::Error {
                        message: err.to_string(),
                    },
                );
                return;
            }
        }

        if std::time::Instant::now() >= hello_deadline {
            let _ = send_shell_packet(
                &mut writer,
                &ShellServerPacket::Error {
                    message: "timed out waiting for shell hello".to_string(),
                },
            );
            return;
        }
        thread::sleep(Duration::from_millis(10));
    };

    // ── Monitor client: stream sysmon messages ──────────────
    // History, then every new line (progress ticks left out) — one
    // subscription to the sysmon ring, like the web System Monitor.
    if matches!(hello, HelloResult::Monitor) {
        let (history, rx) = lumacore_sysmon::subscribe();
        for line in &history {
            if send_shell_packet(
                &mut writer,
                &ShellServerPacket::Frame {
                    ansi: format!("{}\r\n", line.text()),
                    view: None,
                },
            )
            .is_err()
            {
                return;
            }
        }
        sysmon_note("Session", format!("monitor client connected from {addr}"));
        loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(line) if line.transient => {}
                Ok(line) => {
                    if send_shell_packet(
                        &mut writer,
                        &ShellServerPacket::Frame {
                            ansi: format!("{}\r\n", line.text()),
                            view: None,
                        },
                    )
                    .is_err()
                    {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if client disconnected
                    match read_shell_packets(&mut reader, &mut read_buf) {
                        Ok(packets) => {
                            if packets
                                .iter()
                                .any(|p| matches!(p, ShellClientPacket::Disconnect))
                            {
                                break;
                            }
                        }
                        Err(err)
                            if err.kind() == std::io::ErrorKind::WouldBlock
                                || err.kind() == std::io::ErrorKind::TimedOut => {}
                        Err(_) => break,
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        log::info!("monitor client {addr} disconnected");
        return;
    }

    let HelloResult::Shell {
        user,
        width,
        height,
        password,
        token,
        remote,
        from,
        web,
        device,
    } = hello
    else {
        unreachable!()
    };

    // ── Log in and attach ────────────────────────────────────────────────
    let login = {
        let from = from.clone();
        state.call(move |ctx| {
            // Log in: a token from POST /api/login, or user + password.
            let login = match token.as_deref() {
                Some(t) => lumacore_objects::passwords::verify_token(ctx.state(), t)
                    .ok_or_else(|| "login expired — log in again".to_string())
                    .and_then(|name| {
                        lumacore_objects::passwords::may_log_in_from(ctx.state(), &name, remote)
                            .map(|()| name)
                    }),
                None => lumacore_objects::passwords::authenticate(
                    ctx.state(),
                    &user,
                    password.as_deref(),
                    remote,
                ),
            };
            let user = match login {
                Ok(name) => name,
                Err(message) => {
                    ctx.sysmon(
                        "Session",
                        format!("shell login refused from {from}: {message}"),
                    );
                    return Err(message);
                }
            };
            let attach = ctx.attach_shell_client(user.clone(), addr, width, height);
            if web {
                ctx.sessions().mark_ui_client(attach.client_id);
            }
            if remote {
                ctx.sessions().mark_remote_client(attach.client_id);
            }
            // Every login asks for the Meow License before it is finished (decisions 0290): a
            // user + password hello, or a token whose login hasn't consented yet. A reload with a
            // finished login's token is no new login.
            if token.as_deref().is_none_or(|t| lumacore_objects::consent::token_waits(ctx.state(), t)) {
                let holder = match &device {
                    Some(d) => lumacore_objects::dialogs::Audience::Device(d.clone()),
                    None => lumacore_objects::dialogs::Audience::Client(attach.client_id),
                };
                lumacore_objects::consent::begin(ctx.state(), holder, &user, token.as_deref());
            }
            if let Some(device) = device {
                lumacore_objects::devices::known(ctx.state(), &device);
                ctx.sessions().set_client_device(attach.client_id, device);
            }
            ctx.sysmon(
                "Session",
                format!(
                    "shell login accepted for '{user}' from {from} -> {} ({})",
                    attach.session_id,
                    if attach.reused_existing_session {
                        "reattach"
                    } else {
                        "new session"
                    }
                ),
            );
            if let Some(summary) = ctx.sessions().session_summary(attach.session_id) {
                ctx.sysmon("Session", summary);
            }
            ctx.refresh_runtime();
            Ok((
                user,
                attach.client_id,
                attach.session_id,
                attach.reused_existing_session,
            ))
        })
    };
    let (mut user, client_id, mut session_id, reused_existing_session) = match login {
        Ok(Ok(attached)) => attached,
        Ok(Err(message)) => {
            let _ = send_shell_packet(&mut writer, &ShellServerPacket::Error { message });
            return;
        }
        Err(err) => {
            let _ = send_shell_packet(
                &mut writer,
                &ShellServerPacket::Error {
                    message: err.to_string(),
                },
            );
            return;
        }
    };

    // Detach this client (state thread) when the connection ends badly.
    let detach = |state: &StateHandle<C>| {
        state.send(move |ctx| {
            let _ = ctx.sessions().detach_client(client_id);
            ctx.refresh_runtime();
        });
    };

    // "LD connected / disconnected" for everyone (per user and address; see
    // feedback::ClientPresence). Replaced when this client logs in as another user.
    let mut _presence = lumacore_sysmon::feedback::ClientPresence::join(&user, &from);

    // Last frame sent per viewport (None = the untagged client-size frame).
    let mut last_frames: std::collections::HashMap<Option<u32>, String> =
        std::collections::HashMap::new();
    let mut last_status: Option<lumacore_wire::Status> = None;
    let mut needs_render = true; // send the initial frame immediately
    // Keys not run yet, in typed order (never dropped while busy).
    let mut queued_keys: std::collections::VecDeque<ShellClientPacket> = std::collections::VecDeque::new();
    let mut told_busy = false;
    // A suggestion asked while keys were queued: the newest, answered after them.
    let mut pending_suggest: Option<(u64, Option<String>, Option<u32>, lumacore_wire::Surface)> = None;
    let mut last_render = Instant::now();

    loop {
        let mut did_work = false;

        // ── 1. Read and handle inbound packets first ──────────────────────
        // Keys are processed before render so a key press triggers a render
        // on the same iteration, not the next one.
        let read = if early.is_empty() {
            read_shell_packets(&mut reader, &mut read_buf)
        } else {
            Ok(std::mem::take(&mut early))
        };
        match read {
            Ok(packets) => {
                for packet in packets {
                    did_work = true;
                    match packet {
                        // Keys wait in order and are never dropped while the
                        // station is busy (R1 review 18): a lost digit would
                        // change the line (`Delete Fixture 12` → `… 2`).
                        packet @ ShellClientPacket::Key { .. } => queued_keys.push_back(packet),
                        ShellClientPacket::Resize { width, height } => {
                            let resized = state.call_within(SHELL_WAIT, move |ctx| {
                                let _ = ctx.sessions().resize_client(client_id, width, height);
                                ctx.refresh_runtime();
                                if let Some(summary) = ctx.sessions().session_summary(session_id) {
                                    ctx.sysmon(
                                        "Session",
                                        format!("shell resize applied ({summary})"),
                                    );
                                }
                            });
                            if let Err(err) = resized
                                && !call_failed(addr, err)
                            {
                                return;
                            }
                            needs_render = true;
                        }
                        ShellClientPacket::Viewports { viewports } => {
                            state.send(move |ctx| {
                                let _ = ctx.set_shell_viewports(client_id, viewports);
                            });
                            last_frames.clear(); // new sizes: resend everything
                            last_status = None;
                            needs_render = true;
                        }
                        ShellClientPacket::Focus { focus, window } => {
                            state.send(move |ctx| {
                                let _ = ctx.set_shell_focus(client_id, focus, window);
                            });
                        }
                        ShellClientPacket::Screen { no } => {
                            state.send(move |ctx| {
                                let _ = ctx.set_shell_live_screen(client_id, no);
                            });
                        }
                        ShellClientPacket::Scroll { window, row } => {
                            state.send(move |ctx| {
                                let _ = ctx.set_shell_scroll(client_id, window, row);
                            });
                        }
                        // Lines, taps and dialog answers wait in the key queue
                        // too: in order, at most 30 s at a time, never dropped.
                        packet @ (ShellClientPacket::Command { .. }
                        | ShellClientPacket::Tap { .. }
                        | ShellClientPacket::DialogAnswer { .. }) => queued_keys.push_back(packet),
                        // A continuous control's line (decisions §30 "12f"): while the
                        // station is busy only its newest move waits — an older move of
                        // the same gesture still queued is dropped; an end never is.
                        // The dropped move hands on what only a gesture's first line carries.
                        ShellClientPacket::Gesture { id, line, phase, window, mut label, mut from, value } => {
                            queued_keys.retain(|q| match q {
                                ShellClientPacket::Gesture { id: other, phase: lumacore_wire::GesturePhase::Move, label: l, from: f, .. }
                                    if *other == id =>
                                {
                                    label = label.take().or_else(|| l.clone());
                                    from = from.take().or_else(|| f.clone());
                                    false
                                }
                                _ => true,
                            });
                            queued_keys.push_back(ShellClientPacket::Gesture { id, line, phase, window, label, from, value });
                        }
                        // Gestures and key actions wait in the same queue, so
                        // nothing overtakes a key typed before it.
                        packet @ (ShellClientPacket::KeyGesture { .. } | ShellClientPacket::KeyAction { .. }) => {
                            queued_keys.push_back(packet)
                        }
                        ShellClientPacket::DialogPause { id } => {
                            state.send(move |ctx| {
                                let (sessions, state) = ctx.split();
                                let _ = sessions.pause_dialog_for_client(client_id, state, &id);
                                ctx.drain_sysmon_pending();
                            });
                            needs_render = true;
                        }
                        // Suggestions are best effort (merge of bugfixes, §30
                        // busy coverage): behind queued keys the newest one
                        // waits until they ran (answered for the line they
                        // leave — a taken suggestion edits the line and the
                        // list asks again in the same breath), and a busy
                        // station gives none rather than holding the shell
                        // (the next keystroke asks again).
                        ShellClientPacket::Suggest { seq, line, cursor, surface } if !queued_keys.is_empty() => {
                            pending_suggest = Some((seq, line, cursor, surface));
                        }
                        ShellClientPacket::Suggest { seq, line, cursor, surface } => {
                            if !answer_suggest(&state, &mut writer, addr, client_id, seq, line, cursor, surface) {
                                return;
                            }
                        }
                        // A suggestion taken edits the line: in the key queue,
                        // so it never overtakes the keys typed before it.
                        packet @ ShellClientPacket::LineEdit { .. } => {
                            queued_keys.push_back(packet);
                            needs_render = true;
                        }
                        ShellClientPacket::Disconnect => {
                            state.send(move |ctx| {
                                ctx.sysmon(
                                    "Session",
                                    format!(
                                        "shell client {client_id} disconnected from session {session_id} (reused_existing_session={reused_existing_session})"
                                    ),
                                );
                                let _ = ctx.sessions().detach_client(client_id);
                                ctx.refresh_runtime();
                            });
                            let _ = send_shell_packet(
                                &mut writer,
                                &ShellServerPacket::Closed {
                                    message: "shell disconnected".to_string(),
                                    exit_code: 0,
                                },
                            );
                            return;
                        }
                        ShellClientPacket::Hello { .. } | ShellClientPacket::MonitorHello { .. } => {}
                    }
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut => {}
            Err(err) => {
                let _ = send_shell_packet(
                    &mut writer,
                    &ShellServerPacket::Error {
                        message: err.to_string(),
                    },
                );
                detach(&state);
                break;
            }
        }

        // ── 1b. Run the queued keys in order ─────────────────────────────
        while let Some(packet) = queued_keys.front().cloned() {
            did_work = true;
            // A key can run a line: at most 30 s, then "busy" (R1-67, Q40).
            let handled = state.call_within(SHELL_WAIT, move |ctx| {
                let result = match packet {
                    ShellClientPacket::Key { key } => ctx.handle_shell_key(client_id, key).inspect_err(|_| {
                        let _ = ctx.sessions().detach_client(client_id);
                    }),
                    // A gesture or action that fails changes nothing (no reply).
                    ShellClientPacket::KeyGesture { key, gesture } => {
                        Ok(ctx.shell_key_gesture(client_id, key, &gesture).unwrap_or_default())
                    }
                    ShellClientPacket::KeyAction { key, gesture, meow, window } => {
                        Ok(ctx.shell_key_action(client_id, &key, &gesture, meow, window).unwrap_or_default())
                    }
                    ShellClientPacket::Command { cmd, window } => {
                        Ok(ctx.shell_ui_command(client_id, &cmd, window).unwrap_or_default())
                    }
                    ShellClientPacket::Gesture { id, line, phase, window, label, from, value } => {
                        let (sessions, state) = ctx.split();
                        Ok(sessions
                            .gesture_for_client(client_id, state, lumacore_terminal::Gesture { id, line, phase, window, label, from, value })
                            .unwrap_or_default())
                    }
                    ShellClientPacket::Tap { object, no, hold, action, window } => {
                        Ok(ctx.shell_tap(client_id, &object, no, hold, action.as_deref(), window).unwrap_or_default())
                    }
                    ShellClientPacket::DialogAnswer { answer } => {
                        let (sessions, state) = ctx.split();
                        Ok(sessions.answer_dialog_for_client(client_id, state, &answer).unwrap_or_default())
                    }
                    ShellClientPacket::LineEdit { start, end, text, expect } => {
                        let (sessions, state) = ctx.split();
                        let _ = sessions.edit_line_for_client(client_id, state, start, end, &text, expect.as_deref());
                        Ok(Vec::new())
                    }
                    _ => Ok(Vec::new()),
                };
                // Sysmon messages queued by command dispatch.
                ctx.drain_sysmon_pending();
                result
            });
            match handled {
                // The key didn't run: it stays first in the queue, and the
                // keys typed meanwhile wait behind it.
                Err(CallError::Busy { running: false }) => {
                    if !told_busy {
                        told_busy = true;
                        send_ui(
                            &mut writer,
                            vec![lumacore_wire::UiEvent::Notice {
                                text: format!(
                                    "The station is busy: your keys ({}) wait and run in order. [E-CMD-017]",
                                    queued_keys.len()
                                ),
                                level: "warn".into(),
                            }],
                        );
                    }
                    break;
                }
                Err(err @ CallError::Busy { running: true }) => {
                    queued_keys.pop_front();
                    send_ui(
                        &mut writer,
                        vec![lumacore_wire::UiEvent::Notice { text: err.to_string(), level: "warn".into() }],
                    );
                }
                Ok(Ok(events)) => {
                    queued_keys.pop_front();
                    told_busy = false;
                    send_ui(&mut writer, events);
                }
                Ok(Err(message)) => {
                    let _ = send_shell_packet(&mut writer, &ShellServerPacket::Error { message });
                    return;
                }
                Err(err) => {
                    queued_keys.pop_front();
                    if !call_failed(addr, err) {
                        return;
                    }
                }
            }
            needs_render = true;
        }
        // A suggestion asked behind the keys: answered once they ran.
        if queued_keys.is_empty()
            && let Some((seq, line, cursor, surface)) = pending_suggest.take()
            && !answer_suggest(&state, &mut writer, addr, client_id, seq, line, cursor, surface)
        {
            return;
        }

        // ── 2. Render: only after key/resize activity or every ~50ms ─────
        // Skipping idle renders avoids the expensive ratatui TestBackend
        // allocation + full buffer serialization on every loop iteration.
        if needs_render || last_render.elapsed() >= Duration::from_millis(50) {
            needs_render = false;
            last_render = Instant::now();

            // A busy station: this frame is skipped and drawn again on the
            // next turn (bounded like every interactive call, R1-67).
            let pass = state.call_within(SHELL_WAIT, move |ctx| render_pass(ctx, client_id, session_id));
            let pass = match pass {
                Ok(Ok(pass)) => pass,
                Ok(Err(message)) => {
                    let _ = send_shell_packet(&mut writer, &ShellServerPacket::Error { message });
                    break;
                }
                Err(CallError::Busy { .. }) => {
                    needs_render = true;
                    continue;
                }
                Err(err) => {
                    if call_failed(addr, err) {
                        continue;
                    }
                    break;
                }
            };

            if pass.logged_out {
                let event = lumacore_wire::UiEvent::LoggedOut { disconnect: false };
                let _ = send_shell_packet(&mut writer, &ShellServerPacket::Ui { event });
                if let Some((message, exit_code)) = pass.close {
                    let _ = send_shell_packet(&mut writer, &ShellServerPacket::Closed { message, exit_code });
                }
                break;
            }

            if let Some((new_session, new_user, token)) = pass.switched {
                session_id = new_session;
                user = new_user.clone();
                _presence = lumacore_sysmon::feedback::ClientPresence::join(&user, &from);
                last_frames.clear();
                last_status = None;
                let event = lumacore_wire::UiEvent::UserChanged {
                    user: new_user,
                    token,
                };
                let _ = send_shell_packet(&mut writer, &ShellServerPacket::Ui { event });
            }

            let mut send_failed = false;
            for (view, frame) in pass.frames {
                if last_frames.get(&view) == Some(&frame.ansi) {
                    continue;
                }
                let packet = ShellServerPacket::Frame {
                    ansi: frame.ansi,
                    view,
                };
                if send_shell_packet(&mut writer, &packet).is_err() {
                    send_failed = true;
                    break;
                }
                if let ShellServerPacket::Frame { ansi, view } = packet {
                    last_frames.insert(view, ansi);
                }
                did_work = true;
            }
            if !send_failed && last_status.as_ref() != Some(&pass.status) {
                let packet = ShellServerPacket::Status {
                    status: Box::new(pass.status.clone()),
                };
                send_failed = send_shell_packet(&mut writer, &packet).is_err();
                last_status = Some(pass.status);
            }
            if let Some((message, exit_code)) = pass.close {
                let _ = send_shell_packet(
                    &mut writer,
                    &ShellServerPacket::Closed { message, exit_code },
                );
                break;
            }
            if send_failed {
                detach(&state);
                break;
            }
        }

        // ── 3. Yield when idle to prevent busy-spin ────────────────────────
        if !did_work {
            thread::sleep(Duration::from_millis(1));
        }
    }
}

/// On the state thread: apply a pending `Login` switch, render the client's
/// frames, and end the session if it detached or finished.
fn render_pass<C: ShellHost>(
    ctx: &mut C,
    client_id: uuid::Uuid,
    session_id: uuid::Uuid,
) -> Result<RenderPass, String> {
    // `Login` from this client: move it to the new user's session and let its
    // UI rejoin as that user.
    let mut switched = None;
    if let Some(new_user) = ctx.sessions().take_user_switch(client_id) {
        let (sessions, state) = ctx.split();
        let moved = sessions.move_client_to_user(client_id, &new_user, state);
        if let Ok(new_session) = moved {
            ctx.sysmon(
                "Session",
                format!("shell client {client_id} logged in as {new_user}"),
            );
            let token = lumacore_objects::passwords::issue_token(ctx.state(), &new_user);
            switched = Some((new_session, new_user, token));
        }
    }
    let session_id = switched.as_ref().map_or(session_id, |(s, _, _)| *s);

    // A user that is gone from the show (deleted, decisions §30 "12c2") is
    // logged out everywhere: on this station and, by replication, on every
    // member.
    if let Some(user) = ctx.sessions().client_user(client_id).map(str::to_string)
        && !user.eq_ignore_ascii_case(lumacore_objects::INTERNAL_ROOT_USER)
        && ctx.state().root.show_data.user(&user).is_none()
    {
        ctx.sysmon("Session", format!("shell client {client_id} logged out: user {user} no longer exists"));
        let _ = ctx.sessions().detach_client(client_id);
        ctx.refresh_runtime();
        return Ok(RenderPass {
            switched: None,
            frames: Vec::new(),
            status: lumacore_wire::Status::default(),
            close: Some((format!("User {user} was deleted: logged out"), 0)),
            logged_out: true,
        });
    }

    let render = match ctx.render_shell_for_client(client_id) {
        Ok(render) => render,
        Err(err) => {
            let _ = ctx.sessions().detach_client(client_id);
            ctx.refresh_runtime();
            return Err(err);
        }
    };
    let close = if render.detached {
        ctx.sysmon(
            "Session",
            format!("shell client {client_id} detached from session {session_id}"),
        );
        Some(("shell session detached".to_string(), 0))
    } else if !render.running {
        ctx.sysmon(
            "Session",
            format!(
                "shell session {session_id} finished with exit code {}",
                render.exit_code
            ),
        );
        // exit_code 0 means shutdown was requested from the shell
        if render.exit_code == 0 {
            ctx.request_shutdown();
            ctx.sysmon("Shutdown", "shutdown requested from shell session");
        }
        Some(("shell session closed".to_string(), render.exit_code))
    } else {
        None
    };
    if close.is_some() {
        let _ = ctx.sessions().detach_client(client_id);
        ctx.refresh_runtime();
    }
    Ok(RenderPass {
        switched,
        frames: render.frames,
        status: render.status,
        close,
        logged_out: false,
    })
}

pub fn read_shell_packets(
    reader: &mut TcpStream,
    read_buf: &mut Vec<u8>,
) -> std::io::Result<Vec<ShellClientPacket>> {
    let mut chunk = [0u8; 8192];
    let mut packets = Vec::new();

    loop {
        match reader.read(&mut chunk) {
            Ok(0) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "shell client disconnected",
                ));
            }
            Ok(n) => take_packets(read_buf, &chunk[..n], &mut packets)?,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                if packets.is_empty() {
                    return Err(err);
                }
                return Ok(packets);
            }
            Err(err) => return Err(err),
        }

        if !packets.is_empty() {
            return Ok(packets);
        }
    }
}

/// Add `chunk` to `read_buf` and take every whole line out of it as a
/// packet. What is left in `read_buf` never holds a newline, so only the new
/// bytes are searched (R1-19: a long line arriving in many chunks was
/// rescanned from its start after each one, O(n²)).
fn take_packets(
    read_buf: &mut Vec<u8>,
    chunk: &[u8],
    packets: &mut Vec<ShellClientPacket>,
) -> std::io::Result<()> {
    let mut from = read_buf.len();
    read_buf.extend_from_slice(chunk);
    while let Some(at) = read_buf[from..].iter().position(|b| *b == b'\n') {
        let pos = from + at;
        let line = String::from_utf8_lossy(&read_buf[..pos]).to_string();
        read_buf.drain(..=pos);
        from = 0;
        if line.trim().is_empty() {
            continue;
        }
        let packet = serde_json::from_str(line.trim_end())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        packets.push(packet);
    }
    Ok(())
}

pub fn send_shell_packet(
    writer: &mut TcpStream,
    packet: &ShellServerPacket,
) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(packet)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    writer.write_all(&bytes)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// A client speaking another client protocol: tell it why (E-WIRE-001) and
/// close (decisions §23 "Step 8").
fn refuse_protocol(writer: &mut TcpStream, protocol: u32) {
    let message = lumacore_wire::client_mismatch(protocol, lumacore_wire::CLIENT_PROTOCOL);
    log::warn!("shell: {message}");
    let _ = send_shell_packet(writer, &ShellServerPacket::Error { message });
}

/// The client address a hello reports, believed only from a peer on this
/// machine (the HTTP shell bridge); anyone else's claim is ignored
/// (decisions §19).
fn bridged_client(peer: std::net::IpAddr, claimed: Option<&str>) -> Option<std::net::IpAddr> {
    claimed
        .filter(|_| peer.is_loopback())
        .and_then(|a| a.parse().ok())
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
