// ███████╗██████╗  █████╗ ██╗    ██╗███╗   ██╗
// ██╔════╝██╔══██╗██╔══██╗██║    ██║████╗  ██║
// ███████╗██████╔╝███████║██║ █╗ ██║██╔██╗ ██║
// ╚════██║██╔═══╝ ██╔══██║██║███╗██║██║╚██╗██║
// ███████║██║     ██║  ██║╚███╔███╔╝██║ ╚████║
// ╚══════╝╚═╝     ╚═╝  ╚═╝ ╚══╝╚══╝ ╚═╝  ╚═══╝
// FILE: spawn

//! The network threads around the state thread: UDP command port, TCP shell
//! listener (one thread per client) and discovery. Each does its own I/O and
//! reaches the server state only through a [`StateHandle`]; each reports to
//! `Runtime.Health` and stops when `stop` is set.
//!
//! Owns:          the listener threads: the UDP command port (Time answered at the socket), the shell listener, discovery (beacons out of every NIC that serves LumaSession, the cache).
//! Doesn't own:   what a request does (handler), the shell's client thread (lumacore-shell), the sockets (lumacore-net).
//! Entry points:  spawn_command_port, spawn_shell_listener, spawn_discovery, spawn_drives_watcher.
//! Decisions:     §10, §23 "Step 8", §34 24.2.

use std::net::{TcpListener, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use lumacore_tree::runtime::{HealthBoard, ThreadGuard};

use lumacore_shell::handle_shell_client;
use lumacore_net::socket::bind_discovery_sender;
use super::sync::prune_discovery_cache;
use lumacore_sysmon::{sysmon_early, sysmon_note};
use lumacore_net::DiscoveryPeerState;
use lumacore_wire::{RemoteReply, SessionDiscoveryWire, bind_discovery_listener_at};
use super::SESSION_DISCOVERY_TTL_SECS;
use lumacore_tree::now_ms as unix_now_ms;
use crate::state::StateHandle;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn spawn_named(name: &str, f: impl FnOnce() + Send + 'static) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name(name.to_string())
        .spawn(f)
        .unwrap_or_else(|e| panic!("failed to spawn thread '{name}': {e}"))
}

/// Which interfaces serve LumaSession / Shell / HTTP (decisions §11),
/// updated by the state thread.
pub use lumacore_lumasession::Services;

/// How often the drives watcher reads the OS's mounts (decisions §38 6).
const DRIVES_EVERY: Duration = Duration::from_secs(2);

/// The drives watcher (decisions §38): every 2 s it probes the mounted volumes (never on the
/// state thread) and, when something changed, hands them to the state thread, which keeps the
/// drive table, publishes the drives in the `station` feed and writes a sysmon line per change.
pub fn spawn_drives_watcher(state: StateHandle, stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    spawn_named("drives", move || {
        let mut last: Option<Vec<lumacore_drives::probe::Volume>> = None;
        let mut local = (0u64, 0u64);
        while !stop.load(Ordering::Relaxed) {
            let volumes = lumacore_drives::probe::probe();
            let space = lumacore_drives::probe::space(std::path::Path::new("library"));
            if last.as_ref() != Some(&volumes) || space != local {
                last = Some(volumes.clone());
                local = space;
                let sent = state.send(move |ctx| {
                    let lines = lumacore_objects::drives::reconcile(&mut ctx.state, &volumes, space);
                    for line in lines {
                        sysmon_note("Drives", line);
                    }
                    ctx.publish_station_now = true;
                });
                if !sent {
                    last = None;
                }
            }
            thread::sleep(DRIVES_EVERY);
        }
    })
}

/// Requests waiting for the command port's worker. Beyond this the port answers "busy"
/// at once (decisions §31: machine senders get a bounded lane).
const UDP_QUEUE: usize = 64;

/// The UDP command port: one request, one reply. The receive thread only reads the socket:
/// session clock pings are answered there at once, everything else goes to the
/// `udp-requests` worker, which waits for the state thread — so a busy state thread never
/// delays a ping (decisions §24 "Session clock", §31).
pub fn spawn_udp_server(
    socket: UdpSocket,
    state: StateHandle,
    health: Arc<HealthBoard>,
    stop: Arc<AtomicBool>,
    services: Services,
) -> thread::JoinHandle<()> {
    spawn_named("udp", move || {
        let board = health.clone();
        let (queue, requests) = crossbeam_channel::bounded(UDP_QUEUE);
        let worker = match socket.try_clone() {
            Ok(reply_socket) => Some(spawn_udp_worker(reply_socket, requests, state, board.clone())),
            Err(err) => {
                log::error!("UDP command port: no worker ({err}); only clock pings are answered");
                None
            }
        };
        let health = health.register("Udp");
        let _alive = ThreadGuard(health.clone());
        while !stop.load(Ordering::Relaxed) {
            match lumacore_net::socket::recv_request(&socket) {
                Ok((request, addr)) => {
                    // A session clock ping is stamped here, at the socket, and
                    // answered at once (decisions §24 "Session clock").
                    let t2 = lumacore_base::clock::session_us();
                    let started = Instant::now();
                    // LumaSession requests only from interfaces that serve it.
                    if !services.load().lumasession_from(addr.ip()) {
                        continue;
                    }
                    if let lumacore_wire::RemoteRequest::Time { seq, t1, from, report } = request {
                        lumacore_clocksync::answer(&socket, addr, &board, seq, t1, t2, &from, report);
                    } else if worker.is_none() || queue.try_send((request, addr)).is_err() {
                        health.refused();
                        send_reply(&socket, addr, &error_reply(busy_message()));
                    }
                    health.set_queue_depth(queue.len());
                    health.tick(started.elapsed().as_micros() as u64);
                }
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut =>
                {
                    health.heartbeat();
                }
                Err(err) if err.kind() == std::io::ErrorKind::InvalidData => {
                    log::warn!("UDP command port: invalid packet: {err}");
                }
                Err(err) => {
                    log::error!("UDP command port error: {err}");
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        // Closing the queue ends the worker once it has answered what it holds.
        drop(queue);
        if let Some(worker) = worker {
            let _ = worker.join();
        }
    })
}

fn busy_message() -> String {
    lumacore_objects::errors::render_code(
        "The station is busy: too many requests are waiting on its command port. Try again.",
        "E-CMD-017",
    )
}

/// The command port's worker: runs each request on the state thread, in the order they
/// arrived, and sends the reply.
fn spawn_udp_worker(
    socket: UdpSocket,
    requests: crossbeam_channel::Receiver<(lumacore_wire::RemoteRequest, std::net::SocketAddr)>,
    state: StateHandle,
    board: Arc<HealthBoard>,
) -> thread::JoinHandle<()> {
    // A machine sender: the bounded lane (decisions §31).
    let state = state.machine();
    spawn_named("udp-requests", move || {
        let health = board.register("UdpRequests");
        let _alive = ThreadGuard(health.clone());
        loop {
            let (request, addr) = match requests.recv_timeout(Duration::from_millis(250)) {
                Ok(item) => item,
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    health.heartbeat();
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            };
            let started = Instant::now();
            // A command line (a client's, or another station's edit
            // of this one) waits like an HTTP one: at most 10 s, then
            // "busy" (E-CMD-017) and it never runs late (merge of
            // bugfixes, §30 busy coverage); station-to-station
            // requests keep waiting for the state thread.
            let interactive = matches!(request, lumacore_wire::RemoteRequest::Command { .. });
            let job = move |ctx: &mut crate::context::ServerContext| {
                lumacore_lumasession::link::collecting(ctx, |ctx| {
                    ctx.request_from = Some(addr);
                    let (reply, should_exit) = super::handler::handle_request(ctx, request);
                    ctx.request_from = None;
                    if should_exit {
                        ctx.shutdown_requested = true;
                    }
                    reply
                })
            };
            let reply = if interactive {
                state.call_within(crate::state::HTTP_WAIT, job)
            } else {
                state.call(job)
            };
            match reply {
                Ok((reply, waits)) if waits.is_empty() => {
                    send_reply(&socket, addr, &reply);
                }
                // A member waits for its master's answers on a
                // thread of its own, so the port keeps answering
                // (decisions §21 "6b").
                Ok((reply, waits)) => match socket.try_clone() {
                    Ok(socket) => {
                        spawn_named("udp-reply", move || {
                            let reply = lumacore_lumasession::link::with_replies(
                                reply,
                                waits,
                                lumacore_lumasession::link::UDP_WAIT,
                            );
                            send_reply(&socket, addr, &reply);
                        });
                    }
                    Err(err) => send_reply(&socket, addr, &error_reply(err.to_string())),
                },
                Err(err) => send_reply(&socket, addr, &error_reply(err.to_string())),
            }
            health.set_queue_depth(requests.len());
            health.tick(started.elapsed().as_micros() as u64);
        }
    })
}

/// A reply when the state thread could not answer (shutting down, or the
/// request panicked).
fn error_reply(message: String) -> RemoteReply {
    RemoteReply {
        ok: false,
        message,
        server_pid: std::process::id(),
        active_user: None,
        show_path: None,
        debug_mode: false,
        build_id: lumacore_objects::build_info().build_id,
        station_name: None,
        session_count: None,
        station_priority: None,
        lumasession: None,
        kind: lumacore_wire::StationKind::Server,
        client_protocol: lumacore_wire::CLIENT_PROTOCOL,
        session_protocol: lumacore_wire::SESSION_PROTOCOL,
        clear_screen: false,
    }
}

/// The TCP shell listener; every client gets its own thread.
pub fn spawn_shell_server(
    listener: TcpListener,
    state: StateHandle,
    health: Arc<HealthBoard>,
    stop: Arc<AtomicBool>,
    services: Services,
) -> thread::JoinHandle<()> {
    sysmon_early("Engine", "ShellServer thread spawned");
    spawn_named("shell-listener", move || {
        let health = health.register("Shell");
        let _alive = ThreadGuard(health.clone());
        while !stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, addr)) => {
                    health.tick(0);
                    // Only on interfaces that serve Shell (loopback always).
                    let local = stream.local_addr().map(|a| a.ip());
                    if !local.is_ok_and(|ip| services.load().shell_at(ip)) {
                        drop(stream);
                        continue;
                    }
                    let state = state.clone();
                    spawn_named("shell-client", move || {
                        handle_shell_client(stream, addr, state)
                    });
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    health.heartbeat();
                    thread::sleep(Duration::from_millis(50));
                }
                Err(err) => {
                    sysmon_early("Network", format!("[ERROR] Shell listener error: {err}"));
                    thread::sleep(Duration::from_millis(250));
                }
            }
        }
    })
}

/// Peers that dropped out of the discovery cache since they were announced:
/// log them and tell the users.
fn report_lost_peers(
    announced: &mut std::collections::HashMap<String, String>,
    current: &[String],
) {
    let stale: Vec<String> = announced
        .keys()
        .filter(|k| !current.contains(k))
        .cloned()
        .collect();
    for key in stale {
        sysmon_note(
            "Discovery",
            format!("Peer lost: {key} (stale after {SESSION_DISCOVERY_TTL_SECS}s)"),
        );
        if let Some(name) = announced.remove(&key) {
            lumacore_sysmon::feedback::station_lost(&name, &key);
        }
    }
}

/// Discovery: one thread listens for beacons, one sends ours every 750 ms.
pub fn spawn_session_discovery(
    discovery_port: u16,
    state: StateHandle,
    health: Arc<HealthBoard>,
    stop: Arc<AtomicBool>,
    services: Services,
) -> Vec<thread::JoinHandle<()>> {
    // A machine sender: the bounded lane; a refused update comes again with the next beacon.
    let state = state.machine();
    let listener = match bind_discovery_listener_at(discovery_port) {
        Ok(socket) => {
            sysmon_early(
                "Discovery",
                format!(
                    "Multicast listener bound on 0.0.0.0:{discovery_port} (groups {} / {})",
                    lumacore_wire::discovery_group(false),
                    lumacore_wire::discovery_group(true)
                ),
            );
            socket
        }
        Err(err) => {
            sysmon_early(
                "Engine",
                format!("[WARN] Discovery listener bind failed on {discovery_port}: {err}"),
            );
            return Vec::new();
        }
    };
    let sender = match bind_discovery_sender() {
        Ok(socket) => {
            sysmon_early("Discovery", "Beacon sender ready (750ms interval)");
            socket
        }
        Err(err) => {
            sysmon_early(
                "Engine",
                format!("[WARN] Discovery sender unavailable: {err}"),
            );
            return Vec::new();
        }
    };
    let cache = match state.call(|ctx| ctx.discovery_cache.clone()) {
        Ok(cache) => cache,
        Err(_) => return Vec::new(),
    };

    sysmon_early("Engine", "DiscoveryRecv thread spawned");
    let recv_state = state.clone();
    let recv_health = health.clone();
    let recv_stop = stop.clone();
    let recv_services = services.clone();
    let recv_thread = spawn_named("discovery-recv", move || {
        let health = recv_health.register("Discovery");
        let _alive = ThreadGuard(health.clone());
        // Station key → name, to log "found" once and "lost" with the name.
        let mut announced: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut buf = vec![0u8; 65535];
        // The discovery group is joined on every NIC that serves
        // LumaSession (checked every few seconds as NICs and settings change).
        let mut joined: std::collections::BTreeSet<std::net::Ipv4Addr> = Default::default();
        let mut joined_at: Option<Instant> = None;
        // Both groups (Default / Alternative) are joined; beacons from the
        // group this station doesn't use are ignored (§21 "6c").
        let groups = [lumacore_wire::discovery_group(false), lumacore_wire::discovery_group(true)];
        while !recv_stop.load(Ordering::Relaxed) {
            if joined_at.is_none_or(|t| t.elapsed() >= Duration::from_secs(3)) {
                joined_at = Some(Instant::now());
                let want: std::collections::BTreeSet<_> =
                    recv_services.load().lumasession.iter().map(|n| n.ip).collect();
                for ip in want.difference(&joined.clone()) {
                    for group in &groups {
                        let _ = listener.join_multicast_v4(group, ip);
                    }
                    joined.insert(*ip);
                }
                for ip in joined.clone().difference(&want) {
                    for group in &groups {
                        let _ = listener.leave_multicast_v4(group, ip);
                    }
                    joined.remove(ip);
                }
            }
            match listener.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    // Only from interfaces that serve LumaSession.
                    if !recv_services.load().lumasession_from(addr.ip()) {
                        continue;
                    }
                    let Ok(beacon) = serde_json::from_slice::<SessionDiscoveryWire>(&buf[..len])
                    else {
                        continue;
                    };
                    let mine = recv_services.load().group.to_string();
                    if !beacon.group.is_empty() && beacon.group != mine {
                        continue;
                    }
                    let started = Instant::now();
                    // One entry per station (it beacons on every NIC that
                    // serves LumaSession): by station id.
                    let key = if beacon.station_id.is_empty() {
                        format!("{}:{}", addr.ip(), beacon.cmd_port)
                    } else {
                        format!("{}:{}", beacon.station_id, beacon.cmd_port)
                    };
                    // A station shutting down cleanly: gone now, not when it
                    // goes stale (§30 "12b2 hardening").
                    if beacon.leaving {
                        let gone = key.clone();
                        let _ = recv_state.call(move |ctx| {
                            let removed = ctx
                                .discovery_cache
                                .lock()
                                .ok()
                                .is_some_and(|mut cache| cache.remove(&gone).is_some());
                            if removed {
                                ctx.refresh_runtime();
                            }
                        });
                        if let Some(name) = announced.remove(&key) {
                            sysmon_note("Discovery", format!("Peer left: {name} (shut down)"));
                            lumacore_sysmon::feedback::station_lost(&name, &key);
                        }
                        continue;
                    }
                    let peer = DiscoveryPeerState {
                        key: key.clone(),
                        station_name: beacon.station_name.clone(),
                        authority: beacon.authority.clone(),
                        endpoint: format!(
                            "{}:{}, {}:{}",
                            addr.ip(),
                            beacon.cmd_port,
                            addr.ip(),
                            beacon.shell_port
                        ),
                        cmd_port: beacon.cmd_port,
                        stream_port: beacon.stream_port,
                        session_count: beacon.session_count,
                        reachable: true,
                        last_seen_unix_ms: unix_now_ms(),
                        ip: addr.ip(),
                        lumasession: beacon.lumasession.clone(),
                        location: beacon.location.clone(),
                        priority: beacon.station_priority.clone(),
                        status: beacon.status.clone(),
                        version: beacon.version.clone(),
                        invite: beacon.invite,
                        station_id: beacon.station_id.clone(),
                        epoch: beacon.epoch,
                        applied: beacon.applied,
                        online_since_ms: beacon.online_since_ms,
                        show_status: beacon.show_status.clone(),
                    };
                    let (name, cmd_port) = (beacon.station_name.clone(), beacon.cmd_port);
                    // Our own beacon (multicast loops back) is not a peer.
                    let current = recv_state.call(move |ctx| {
                        if name == ctx.state.root.station_settings.station_name
                            && cmd_port == ctx.cmd_port
                        {
                            // Our own beacon: still a moment to drop stations
                            // that went quiet from the LumaSessions list.
                            let dropped = ctx.discovery_cache.lock().ok().is_some_and(|mut cache| {
                                let before = cache.len();
                                prune_discovery_cache(&mut cache);
                                cache.len() != before
                            });
                            if dropped {
                                ctx.refresh_runtime();
                            }
                            return None;
                        }
                        let keys = ctx.discovery_cache.lock().ok().map(|mut cache| {
                            // Keep the address a station was first heard on
                            // (prefer one that isn't loopback), so it doesn't
                            // flip between its NICs.
                            let mut peer = peer;
                            if let Some(old) = cache.get(&peer.key)
                                && (!old.ip.is_loopback() || peer.ip.is_loopback())
                            {
                                peer.ip = old.ip;
                                peer.endpoint = old.endpoint.clone();
                            }
                            cache.insert(peer.key.clone(), peer);
                            prune_discovery_cache(&mut cache);
                            cache.keys().cloned().collect::<Vec<_>>()
                        });
                        ctx.refresh_runtime();
                        keys
                    });
                    let Ok(Some(current)) = current else {
                        continue;
                    };
                    if announced
                        .insert(key.clone(), beacon.station_name.clone())
                        .is_none()
                    {
                        sysmon_note(
                            "Discovery",
                            format!(
                                "Peer found: {} ({}:{})",
                                beacon.station_name,
                                addr.ip(),
                                beacon.cmd_port
                            ),
                        );
                        lumacore_sysmon::feedback::station_joined(
                            &beacon.station_name,
                            &addr.ip().to_string(),
                        );
                    }
                    report_lost_peers(&mut announced, &current);
                    health.tick(started.elapsed().as_micros() as u64);
                }
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut =>
                {
                    health.heartbeat();
                    // Idle: peers can still go stale.
                    let current = cache.lock().ok().map(|mut cache| {
                        let before = cache.len();
                        prune_discovery_cache(&mut cache);
                        (cache.len() != before, cache.keys().cloned().collect::<Vec<_>>())
                    });
                    if let Some((dropped, current)) = current {
                        if dropped {
                            let _ = recv_state.call(|ctx| ctx.refresh_runtime());
                        }
                        report_lost_peers(&mut announced, &current);
                    }
                }
                Err(err) => {
                    log::debug!("session discovery listener error: {err}");
                    thread::sleep(Duration::from_millis(250));
                }
            }
        }
    });

    sysmon_early("Engine", "DiscoverySend thread spawned");
    let send_thread = spawn_named("discovery-send", move || {
        let health = health.register("DiscoverySend");
        let _alive = ThreadGuard(health.clone());
        // One beacon out of every NIC that serves LumaSession; none when
        // LumaSession is off.
        let send = |beacon: &SessionDiscoveryWire| {
            let access = services.load();
            let target = (access.group, discovery_port);
            if let Ok(bytes) = serde_json::to_vec(beacon) {
                for nic in access.lumasession.iter().filter(|_| access.online) {
                    let _ = socket2::SockRef::from(&sender).set_multicast_if_v4(&nic.ip);
                    let _ = sender.send_to(&bytes, target);
                }
            }
        };
        let mut last: Option<SessionDiscoveryWire> = None;
        while !stop.load(Ordering::Relaxed) {
            // The state thread is gone once the server shuts down.
            let Ok(beacon) = state.call(|ctx| ctx.local_discovery_beacon()) else {
                break;
            };
            send(&beacon);
            last = Some(beacon);
            health.tick(0);
            // Sleep in short steps so shutdown doesn't wait for a whole interval.
            for _ in 0..15 {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
        // Shutting down: a last beacon that says so, so listeners (the app's
        // servers list, other stations) drop us at once (§30 "12b2 hardening").
        if let Some(mut beacon) = last {
            beacon.leaving = true;
            beacon.sent_unix_ms = unix_now_ms();
            send(&beacon);
        }
    });

    vec![recv_thread, send_thread]
}

/// A reply on the command port; one too large for a packet is refused with its code (E-WIRE-002).
fn send_reply(socket: &std::net::UdpSocket, addr: std::net::SocketAddr, reply: &lumacore_wire::RemoteReply) {
    lumacore_net::socket::send_reply(socket, addr, reply, |bytes| {
        lumacore_objects::errors::render_code(
            format!("The reply ({bytes} bytes) is too large for the command port; run the line in a shell."),
            "E-WIRE-002",
        )
    });
}
