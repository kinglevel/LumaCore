//! Runtime-related list functions: network, sessions, stations, engines, playbacks.

use nu_ansi_term::{Color, Style};

use crate::AppState;

/// `Root.Runtime.Network` / `List Interface`: this station's interfaces and
/// what each serves (Menu → Network → Interfaces).
pub fn list_network(state: &AppState) -> String {
    let network = &state.root.runtime.network;
    let yn = |b: bool, w: usize| {
        if b {
            Color::Green.paint(format!("{:<w$}", "Yes"))
        } else {
            Color::DarkGray.paint(format!("{:<w$}", "No"))
        }
    };
    let mut lines = vec![
        Style::new().bold().paint("Interfaces").to_string(),
        format!(
            "  {:<8} {:<20} {:<12} {:<6} {:<6}",
            "Name", "Address", "LumaSession", "Shell", "HTTP"
        ),
    ];
    for nic in network.interfaces.values() {
        lines.push(format!(
            "  {} {:<20} {} {} {}",
            Color::Cyan.paint(format!("{:<8}", nic.name)),
            nic.address,
            yn(nic.lumasession, 12),
            yn(nic.shell, 6),
            yn(nic.http, 6)
        ));
    }
    if network.interfaces.is_empty() {
        lines.push(Style::new().dimmed().paint("  (no interfaces)").to_string());
    }
    if !state.root.station_settings.online {
        lines.push(
            Color::Red
                .paint("LumaSession is off: only localhost is served (Set Station \"Online\" On)")
                .to_string(),
        );
    }
    lines.join("\n")
}

/// `List LumaSession`: this station and every station discovery sees (the
/// LumaSessions grid), then this master's members and its listeners.
pub fn list_lumasession(state: &AppState) -> String {
    let ls = &state.root.runtime.luma_sessions;
    let mut lines = vec![
        Style::new().bold().paint("LumaSessions").to_string(),
        format!(
            "  {:<16} {:<16} {:<14} {:<12} {:<11} {:<8} {:<7} {:<8} {}",
            "Station", "IP", "LumaSession", "Location", "Status", "Priority", "Invite", "Version", "Show"
        ),
    ];
    let mut peers: Vec<_> = ls.discovery.values().filter(|p| p.cmd_port != 0).collect();
    peers.sort_by(|a, b| (a.ip != "this station", &a.name).cmp(&(b.ip != "this station", &b.name)));
    for p in peers {
        let name = if p.ip == "this station" {
            Color::Yellow.paint(format!("{:<16}", format!("{} ◀", p.name))).to_string()
        } else {
            Color::Cyan.paint(format!("{:<16}", p.name)).to_string()
        };
        lines.push(format!(
            "  {} {:<16} {:<14} {:<12} {:<11} {:<8} {:<7} {:<8} {}",
            name,
            p.ip,
            p.lumasession,
            p.location,
            p.status,
            p.priority,
            if p.invite { "Yes" } else { "No" },
            p.version,
            p.show_status
        ));
    }
    if !ls.members.is_empty() {
        lines.push(Style::new().bold().paint("Members").to_string());
        for m in ls.members.values() {
            lines.push(format!("  {} {}:{}", Color::Green.paint(&m.station), m.ip, m.cmd_port));
        }
    }
    let listeners: Vec<String> = ls
        .listeners
        .values()
        .map(|l| format!("{} {} {}", l.name, l.transport, l.port))
        .collect();
    lines.push(format!("Listeners: {}", listeners.join(", ")));
    lines.join("\n")
}

/// `List Shell`: logged-in users and their shell clients.
pub fn list_shells(state: &AppState) -> String {
    let mut sessions: Vec<_> = state.root.runtime.users.values().collect();
    sessions.sort_by(|a, b| a.user.cmp(&b.user));
    if sessions.is_empty() {
        return Style::new()
            .dimmed()
            .paint("No runtime sessions.")
            .to_string();
    }

    let mut lines = vec![Style::new().bold().paint("Users").to_string()];
    for session in sessions {
        lines.push(format!(
            "  {}  modes={} clients={} stations={} watches={} {} authority={}",
            Color::Cyan.paint(&session.user),
            {
                let on: Vec<&str> = session.modes.on().map(|m| m.name()).collect();
                if on.is_empty() {
                    "-".to_string()
                } else {
                    on.join(",")
                }
            },
            session.clients.len(),
            session.stations.len(),
            session.watches.len(),
            if session.attached { "attached" } else { "idle" },
            session.authority
        ));
    }
    lines.join("\n")
}

/// `List Station`: this station, its LumaSession's members, and every
/// station the show knows (In Session / Lost / Known; §21 "6c").
pub fn list_stations(state: &AppState) -> String {
    let station = &state.root.station_settings;
    let mut lines = vec![
        Style::new().bold().paint("Stations").to_string(),
        format!(
            "  {} ◀  {}  priority {}",
            Color::Cyan.paint(&station.station_name),
            crate::session_link::status_word(state),
            station.station_priority
        ),
    ];
    if let Some(link) = &state.link {
        lines.push(format!("  {}  Master  {}", Color::Cyan.paint(&link.master), link.target));
    }
    for m in state.root.runtime.luma_sessions.members.values() {
        if m.station == station.station_name {
            continue;
        }
        lines.push(format!(
            "  {}  Member{}  {}{}",
            Color::Cyan.paint(&m.station),
            if m.connected { "" } else { " (lost)" },
            m.ip,
            if m.users.is_empty() {
                String::new()
            } else {
                format!("  users: {}", m.users.join(", "))
            }
        ));
    }
    let known = crate::session_link::known_lines(state);
    if !known.is_empty() {
        lines.push(String::new());
        lines.push(Style::new().bold().paint("Known stations (the show's list; Delete Station forgets one)").to_string());
        lines.extend(known);
    }
    lines.join("\n")
}

pub fn list_clients(state: &AppState) -> String {
    let mut clients: Vec<_> = state
        .root
        .runtime
        .users
        .values()
        .flat_map(|session| session.clients.values())
        .collect();
    clients.sort_by_key(|client| client.id);
    let mut discovered: Vec<_> = state
        .root
        .runtime
        .luma_sessions
        .discovery
        .values()
        .filter(|peer| peer.role.contains("Authority"))
        .collect();
    discovered.sort_by(|a, b| a.name.cmp(&b.name));

    if clients.is_empty() && discovered.is_empty() {
        return Style::new()
            .dimmed()
            .paint("No attached or discovered clients.")
            .to_string();
    }

    let mut lines = vec![Style::new().bold().paint("Clients").to_string()];
    for client in clients {
        lines.push(format!(
            "  {}  {}  {}  {}x{}",
            Color::Cyan.paint(client.id.to_string()),
            client.kind,
            client.remote_addr,
            client.width,
            client.height
        ));
    }
    for peer in discovered {
        lines.push(format!(
            "  {}  {}  {}",
            Color::Yellow.paint(&peer.name),
            "DiscoveredSession",
            peer.endpoint
        ));
    }
    lines.join("\n")
}

pub fn list_network_engines(state: &AppState) -> String {
    list_engine_family("NetworkEngines", &state.root.runtime.network_engines)
}

pub fn list_session_engines(state: &AppState) -> String {
    list_engine_family("SessionEngines", &state.root.runtime.session_engines)
}

pub fn list_rendering_engines(state: &AppState) -> String {
    list_engine_family("RenderingEngines", &state.root.runtime.rendering_engines)
}

pub fn list_global_variables(state: &AppState) -> String {
    if state.root.runtime.global_variables.is_empty() {
        return Style::new()
            .dimmed()
            .paint("No global variables.")
            .to_string();
    }
    let mut lines = vec![Style::new().bold().paint("GlobalVariables").to_string()];
    for (name, value) in &state.root.runtime.global_variables {
        lines.push(format!("  {} = {:?}", Color::Cyan.paint(name), value));
    }
    lines.join("\n")
}

fn list_engine_family(
    title: &str,
    engines: &std::collections::HashMap<String, lumacore_tree::EngineRuntime>,
) -> String {
    if engines.is_empty() {
        return Style::new()
            .dimmed()
            .paint(format!("No {title}."))
            .to_string();
    }

    let mut entries: Vec<_> = engines.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut lines = vec![Style::new().bold().paint(title).to_string()];
    for (name, engine) in entries {
        let state = if engine.running { "running" } else { "idle" };
        lines.push(format!(
            "  {}  revision={}  {}",
            Color::Cyan.paint(name),
            engine.revision,
            state
        ));
    }
    lines.join("\n")
}
