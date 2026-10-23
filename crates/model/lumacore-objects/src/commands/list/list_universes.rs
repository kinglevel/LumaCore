//! Universe, protocol line and network list functions.

use lumacore_tree::{LineOps, TreeNode};
use lumacore_protocols::{Direction, Kind, MergeMode, mode_label, wire_label};
use nu_ansi_term::Color;

use crate::AppState;
use crate::text::{LineRow, PropertySheet, UniverseRow, render_line_list};

/// Show detail/properties for a specific universe.
pub fn list_universe_detail(state: &AppState, id: u32) -> String {
    use crate::commands::info;
    info::info_universe(state, Some(&id.to_string()))
}

pub fn list_universes(state: &AppState) -> String {
    let rows: Vec<UniverseRow> = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .universes
        .iter()
        .map(|(&id, u)| {
            let name = u.meta.name().unwrap_or("<unnamed>").to_string();
            let uid_guid = u.meta.identity.guid;

            let fixture_count = state
                .root
                .show_data
                .devices
                .dmx
                .patch
                .fixtures
                .values()
                .filter(|f| {
                    f.primary_break()
                        .map(|b| b.universe_id == uid_guid)
                        .unwrap_or(false)
                })
                .count() as u32;

            let occupied = u.occupied_count();
            let pct = (occupied * 100) / 512;
            let bar_len = 20;
            let filled = (pct * bar_len) / 100;
            let usage_bar = format!(
                "[{}{}] {}/{} ({}%)",
                "\u{2588}".repeat(filled),
                "\u{2591}".repeat(bar_len - filled),
                occupied,
                512,
                pct
            );

            let on = LineOps::lines_on(&state.root.show_data, id);
            let routes: Vec<String> = on.iter().map(|(p, no, mode, w)| route_label(p, *no, *mode, *w)).collect();
            let merge = if on.iter().any(|(_, _, mode, _)| *mode == Direction::In)
                && u.merge_mode != MergeMode::Prio
            {
                format!(" (merge {})", u.merge_mode.label())
            } else {
                String::new()
            };
            let sacn_status = if routes.is_empty() {
                Color::Yellow.paint("unassigned").to_string()
            } else {
                Color::Green.paint(format!("{}{merge}", routes.join(", "))).to_string()
            };

            UniverseRow {
                id,
                name,
                fixtures: fixture_count,
                usage: usage_bar,
                sacn: sacn_status,
            }
        })
        .collect();

    crate::text::render_universe_status(rows)
}

/// `→ Sacn 2 10` / `← ArtNet 1 0.0.1` — a line carrying a universe, as shown.
pub fn route_label(protocol: &str, line: u32, mode: Direction, wire: u16) -> String {
    let arrow = match mode {
        Direction::Out => "→",
        Direction::In => "←",
    };
    format!("{arrow} {protocol} {line} {}", wire_label(protocol, wire))
}

/// A line's protocol settings as `Priority 150` (decisions §34 22.4).
fn settings_text(l: &lumacore_protocols::Line) -> String {
    l.declared().iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(" ")
}

fn state_cell(states: &[lumacore_tree::LineRuntime], protocol: &str, line: u32) -> String {
    match states.iter().find(|s| s.protocol == protocol && s.line == line) {
        Some(s) if s.active => Color::Green.paint(&s.state).to_string(),
        Some(s) if s.problem => Color::Red.paint(&s.state).to_string(),
        Some(s) => Color::Yellow.paint(&s.state).to_string(),
        None => String::new(),
    }
}

/// `List Sacn`: a protocol's lines with their universes, settings and live state.
pub fn list_lines(state: &AppState, protocol: &str) -> String {
    let show = &state.root.show_data;
    let states = crate::protocol_plan::line_states(state);
    let data = lumacore_protocols::protocol::find(protocol).is_some_and(|p| p.kind == Kind::Data);
    let rows: Vec<LineRow> = show
        .protocols
        .lines(protocol)
        .into_iter()
        .flatten()
        .map(|(&id, l)| LineRow {
            id,
            name: l.name().unwrap_or("").to_string(),
            network: show.protocols.network_name(l.network).to_string(),
            mode: l.mode.label().to_string(),
            universes: if data { "—".to_string() } else { l.run_label() },
            via: if data {
                format!("as {}", l.user.and_then(|u| show.user_name(u)).unwrap_or("—"))
            } else if l.mode == Direction::In {
                format!("{}{}", l.input_priority.label(), l.target.map_or(String::new(), |ip| format!(" from {ip}")))
            } else {
                mode_label(&l.protocol, l.target)
            },
            settings: settings_text(l),
            status: if l.enabled {
                state_cell(&states, protocol, id)
            } else {
                Color::Yellow.paint("disabled").to_string()
            },
        })
        .collect();
    render_line_list(protocol, rows)
}

/// `List Network`: the show's networks, this station's mapping of each, and
/// the interfaces this machine has.
pub fn list_networks(state: &AppState) -> String {
    let show = &state.root.show_data;
    let mapping = &state.root.station_settings.networks;
    let nics = lumacore_net::nics::list();
    let mut sheet = PropertySheet::new("Networks (show) → interface (this station)");
    if show.protocols.networks.is_empty() {
        sheet = sheet.line("  (none — Store Network \"Rig\")");
    }
    for (&id, n) in &show.protocols.networks {
        let name = n.name().unwrap_or("");
        let target = match mapping.get(&name.to_lowercase()) {
            None => Color::Red.paint("not mapped — nothing is sent").to_string(),
            Some(spec) => match lumacore_net::nics::resolve(spec, &nics) {
                Some(nic) => Color::Green
                    .paint(format!(
                        "{spec} → {} {} (broadcast {})",
                        nic.name, nic.ip, nic.broadcast
                    ))
                    .to_string(),
                None => Color::Red
                    .paint(format!("{spec} — no such interface here"))
                    .to_string(),
            },
        };
        let lines = show.protocols.all_lines().filter(|(_, _, l)| l.network == n.meta.identity.guid).count();
        sheet = sheet.line(&format!(
            "  {}  {:<12} {:<3} line(s)  {}",
            Color::Cyan.paint(format!("{id:>3}")),
            name,
            lines,
            target
        ));
    }
    sheet = sheet.line("").line("  Interfaces on this station:");
    for nic in &nics {
        sheet = sheet.line(&format!(
            "    {:<8} {:<15} broadcast {}",
            nic.name, nic.ip, nic.broadcast
        ));
    }
    sheet.build()
}
