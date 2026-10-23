// ███████╗██╗   ██╗███╗   ██╗ ██████╗
// ██╔════╝╚██╗ ██╔╝████╗  ██║██╔════╝
// ███████╗ ╚████╔╝ ██╔██╗ ██║██║
// ╚════██║  ╚██╔╝  ██║╚██╗██║██║
// ███████║   ██║   ██║ ╚████║╚██████╗
// ╚══════╝   ╚═╝   ╚═╝  ╚═══╝ ╚═════╝
// FILE: sync

//!
//! Owns:          Root.Runtime's network view (listeners, discovered stations, connections, users' clients), pruning the discovery cache.
//! Doesn't own:   the discovery threads (spawn), the terminals (lumacore-terminal).
//! Entry points:  sync_session_network_runtime, prune_discovery_cache.
//! Decisions:     §23 "Step 8", §34 24.2.

use std::collections::BTreeMap;

use lumacore_tree::{
    DiscoveryPeerRuntime, NetworkConnectionRuntime, NetworkInterfaceRuntime,
    NetworkListenerRuntime, SessionClientRuntime, SessionStationRuntime,
};

use lumacore_terminal::session_manager::SessionRegistry;

use lumacore_net::DiscoveryPeerState;
use super::SESSION_DISCOVERY_TTL_SECS;
use lumacore_tree::now_ms as unix_now_ms;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

pub(crate) fn sync_session_network_runtime(
    state: &mut lumacore_objects::AppState,
    sessions: &SessionRegistry,
    (cmd_port, shell_port, stream_port): (u16, u16, u16),
    http_port: Option<u16>,
    discovered_peers: Vec<DiscoveryPeerState>,
) {
    let station_name = state.root.station_settings.station_name.clone();
    let shell_endpoint = format!("0.0.0.0:{shell_port}");
    let cmd_endpoint = format!("0.0.0.0:{cmd_port}");
    let revision = state.show_generation;

    state.root.runtime.network_engines.insert(
        "SessionNetwork".to_string(),
        lumacore_tree::EngineRuntime {
            running: true,
            revision,
            last_error: None,
        },
    );
    state.root.runtime.session_engines.insert(
        "SessionAuthority".to_string(),
        lumacore_tree::EngineRuntime {
            running: true,
            revision,
            last_error: None,
        },
    );

    state.root.runtime.luma_sessions.listeners.clear();
    state.root.runtime.luma_sessions.listeners.insert(
        "Cmdline".to_string(),
        NetworkListenerRuntime {
            name: "Cmdline".to_string(),
            transport: "UDP".to_string(),
            bind: "0.0.0.0".to_string(),
            port: cmd_port,
            active: true,
        },
    );
    if let Some(http_port) = http_port {
        state.root.runtime.luma_sessions.listeners.insert(
            "HTTP".to_string(),
            NetworkListenerRuntime {
                name: "HTTP".to_string(),
                transport: "TCP".to_string(),
                bind: "0.0.0.0".to_string(),
                port: http_port,
                active: true,
            },
        );
    }
    state.root.runtime.luma_sessions.listeners.insert(
        "Shell".to_string(),
        NetworkListenerRuntime {
            name: "Shell".to_string(),
            transport: "TCP".to_string(),
            bind: "0.0.0.0".to_string(),
            port: shell_port,
            active: true,
        },
    );

    state.root.runtime.luma_sessions.discovery.clear();
    state.root.runtime.luma_sessions.discovery.insert(
        station_name.clone(),
        DiscoveryPeerRuntime {
            name: station_name.clone(),
            role: "Authority".to_string(),
            endpoint: format!("{cmd_endpoint}, {shell_endpoint}"),
            reachable: true,
            session_count: state.root.runtime.users.len() as u32,
            last_seen_unix_ms: Some(unix_now_ms()),
            lumasession: state.root.station_settings.lumasession.clone(),
            location: state.root.station_settings.location.clone(),
            priority: state.root.station_settings.station_priority.to_string(),
            status: lumacore_objects::session_link::status_word(state).to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            invite: state.root.station_settings.invite,
            ip: "this station".to_string(),
            cmd_port,
            stream_port,
            station_id: state.local_settings.station_id.clone(),
            show_status: state.show_status.to_string(),
            epoch: state.session_epoch,
        },
    );

    // This station's interfaces with what each serves (Menu → Network).
    state.root.runtime.network.interfaces.clear();
    for nic in lumacore_net::nics::list() {
        let services = state.root.station_settings.services_of(&nic.name);
        state.root.runtime.network.interfaces.insert(
            format!("{} {}", nic.name, nic.ip),
            NetworkInterfaceRuntime {
                name: nic.name.clone(),
                address: format!("{}/{}", nic.ip, u32::from(nic.netmask).count_ones()),
                active: true,
                lumasession: services.lumasession,
                shell: services.shell || nic.ip.is_loopback(),
                http: services.http || nic.ip.is_loopback(),
            },
        );
    }

    let snapshot = sessions.snapshot();

    state.root.runtime.luma_sessions.connections.clear();
    for client in &snapshot.clients {
        state.root.runtime.luma_sessions.connections.insert(
            client.id.to_string(),
            NetworkConnectionRuntime {
                id: client.id.to_string(),
                transport: "TCP Shell".to_string(),
                peer: client.remote_addr.to_string(),
                session_id: Some(client.session_id),
                client_id: Some(client.id),
                state: "attached".to_string(),
            },
        );
    }

    let joined_sessions: Vec<_> = state
        .root
        .runtime
        .joined_sessions
        .values()
        .cloned()
        .collect();

    for joined in &joined_sessions {
        let key = joined.target.clone();
        state.root.runtime.luma_sessions.discovery.insert(
            key.clone(),
            DiscoveryPeerRuntime {
                name: key.clone(),
                role: "RemoteAuthority".to_string(),
                endpoint: joined.target.clone(),
                reachable: joined.connected,
                session_count: 1,
                last_seen_unix_ms: Some(unix_now_ms()),
                status: "Master".to_string(),
                ip: joined.authority.clone(),
                cmd_port: joined.cmd_port,
                ..Default::default()
            },
        );
        state.root.runtime.luma_sessions.connections.insert(
            format!("join:{key}"),
            NetworkConnectionRuntime {
                id: format!("join:{key}"),
                transport: "LumaSession stream".to_string(),
                peer: joined.target.clone(),
                session_id: None,
                client_id: None,
                state: if joined.connected {
                    "joined".to_string()
                } else {
                    "disconnected".to_string()
                },
            },
        );
    }

    for peer in discovered_peers {
        state.root.runtime.luma_sessions.discovery.insert(
            peer.key.clone(),
            DiscoveryPeerRuntime {
                name: peer.station_name.clone(),
                role: if peer.authority == peer.station_name {
                    "DiscoveredAuthority".to_string()
                } else {
                    "DiscoveredClient".to_string()
                },
                endpoint: peer.endpoint.clone(),
                reachable: peer.reachable,
                session_count: peer.session_count,
                last_seen_unix_ms: Some(peer.last_seen_unix_ms),
                lumasession: peer.lumasession.clone(),
                location: peer.location.clone(),
                priority: peer.priority.clone(),
                status: peer.status.clone(),
                version: peer.version.clone(),
                invite: peer.invite,
                ip: peer.ip.to_string(),
                cmd_port: peer.cmd_port,
                stream_port: peer.stream_port,
                station_id: peer.station_id.clone(),
                show_status: peer.show_status.clone(),
                epoch: peer.epoch,
            },
        );
    }

    state.root.runtime.luma_sessions.health.state = "running".to_string();
    state.root.runtime.luma_sessions.health.last_event = Some(format!(
        "authority={} sessions={} clients={}",
        station_name,
        snapshot.sessions.len(),
        snapshot.clients.len()
    ));

    let mut clients_by_session = std::collections::HashMap::new();
    for client in snapshot.clients {
        clients_by_session
            .entry(client.session_id)
            .or_insert_with(Vec::new)
            .push(client);
    }

    let live_users: std::collections::HashSet<String> =
        snapshot.sessions.iter().map(|s| s.user.clone()).collect();

    for session in snapshot.sessions {
        let runtime_session = state.root.runtime.user(&session.user);
        runtime_session.meta.guid = session.id;
        runtime_session.user = session.user.clone();
        runtime_session.authority = station_name.clone();
        runtime_session.attached = !session.attached_client_ids.is_empty();
        runtime_session.stations.clear();
        runtime_session.stations.insert(
            station_name.clone(),
            SessionStationRuntime {
                name: station_name.clone(),
                role: "Authority".to_string(),
                endpoint: shell_endpoint.clone(),
                connected: true,
                ..Default::default()
            },
        );
        runtime_session.clients.clear();
        for client in clients_by_session.remove(&session.id).unwrap_or_default() {
            runtime_session.clients.insert(
                client.id.to_string(),
                SessionClientRuntime {
                    id: client.id,
                    kind: "Shell".to_string(),
                    remote_addr: client.remote_addr.to_string(),
                    width: client.width,
                    height: client.height,
                    connected: true,
                },
            );
        }
    }

    for (user, session) in state.root.runtime.users.iter_mut() {
        if live_users.contains(user) {
            continue;
        }
        session.attached = false;
        session.authority = station_name.clone();
        session.stations.clear();
        session.clients.clear();
    }

    for joined in joined_sessions {
        let session = state.root.runtime.user(&joined.user);
        session.user = joined.user.clone();
        session.authority = joined.target.clone();
        session.attached = joined.connected;
        session.stations.insert(
            joined.target.clone(),
            SessionStationRuntime {
                name: joined.target.clone(),
                role: "Authority".to_string(),
                endpoint: joined.target.clone(),
                connected: joined.connected,
                ..Default::default()
            },
        );
        session.stations.insert(
            station_name.clone(),
            SessionStationRuntime {
                name: station_name.clone(),
                role: "Client".to_string(),
                endpoint: shell_endpoint.clone(),
                connected: true,
                ..Default::default()
            },
        );
    }
}

pub(crate) fn prune_discovery_cache(cache: &mut BTreeMap<String, DiscoveryPeerState>) {
    let cutoff = unix_now_ms().saturating_sub(SESSION_DISCOVERY_TTL_SECS * 1000);
    cache.retain(|_, peer| peer.last_seen_unix_ms >= cutoff);
}
