//! Universe commands: create, delete, and the runs of universes protocol lines carry
//! (decisions §11, §34 "34f7c hardened").
//!
//! Store Universe {id} ["name"]
//! Assign Universe 1 Thru 6 At Sacn 1 /Universe 1      (Output; typed in objects::patch)
//! Assign ArtNet 1 At Universe 2 /Universe 0.1.2       (Input; Art-Net Net.SubNet.Universe)
//! Unassign Universe 1 [At Sacn 1]

use lumacore_tree::events::TreeEvent;
use lumacore_tree::{LineOps};
use lumacore_fixtures::{Universe};
use lumacore_protocols::Direction;

use crate::AppState;
use crate::errors::render_code;
use crate::text::render_success;

/// A new universe `id` (`Store Universe 3 ["name"]`); unnamed it is "Universe 3".
pub fn create_universe(state: &mut AppState, id: u32, name: Option<String>) -> String {
    if id == 0 {
        return render_code("Universe numbers start at 1.", "E-CMD-008");
    }
    let name = name.unwrap_or_else(|| format!("Universe {id}"));

    // Check if already exists
    if state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .universes
        .contains_key(&id)
    {
        return render_code(format!("Universe {id} already exists."), "E-CMD-010");
    }

    // Insert directly with the user-specified ID (not from pool, since user chose the ID)
    let universe = Universe::new(Some(name.clone()));
    state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .universes
        .insert(id, universe);

    // Reserve this ID in the pool so future pool allocations skip it
    state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .universe_id_pool
        .reserve(id);

    state
        .root
        .emit(&TreeEvent::UniverseCreated { universe_id: id });

    render_success(&format!("Created Universe {id} '{name}'."))
}

/// The protocol universe a typed `/Universe` names on `protocol`'s wire (Art-Net `0.0.5`, sACN a
/// number); a plain number outside the range goes on to the range check (E-OUTPUT-005).
fn wire_start(protocol: &str, wire: Option<&str>) -> Result<Option<u32>, String> {
    let universes = lumacore_protocols::protocol::find(protocol).and_then(|p| p.universes);
    match (wire.map(str::trim), universes) {
        (None, _) => Ok(None),
        (Some(w), Some(u)) => (u.parse)(w)
            .map(u32::from)
            .or_else(|| w.parse::<u32>().ok())
            .map(Some)
            .ok_or_else(|| render_code(format!("/Universe {w}: a {protocol} universe ({}).", u.range_text()), "E-CMD-008")),
        (Some(w), None) => Err(render_code(format!("/Universe {w}: {protocol} carries commands, not universes."), "E-OUTPUT-006")),
    }
}

/// `Universe 5 Thru 8` as typed again (for the buttons of an ask).
fn run_text(ids: &[u32]) -> String {
    match (ids.iter().min(), ids.iter().max()) {
        (Some(a), Some(b)) if a == b => format!("Universe {a}"),
        (Some(a), Some(b)) => format!("Universe {a} Thru {b}"),
        _ => "Universe".to_string(),
    }
}

/// Give line `line` of `protocol` the run `ids` in `mode` (decisions §34 "34f7c hardened" 3):
/// `Assign Universe 5 Thru 8 At Sacn 1 [/Universe 101]` (Output), `Assign Sacn 2 At Universe 40`
/// (Input). A line that doesn't exist yet is made on your only network; a line that already
/// carries another run asks Replace / New line / Cancel. `again` is the line as typed, run
/// again by the ask's buttons.
pub fn assign_run(
    state: &mut AppState,
    protocol: &str,
    line: u32,
    ids: &[u32],
    mode: Direction,
    wire: Option<&str>,
) -> crate::DispatchResult {
    let reply = |s: String| crate::DispatchResult::Output(s.into());
    let start = match wire_start(protocol, wire) {
        Ok(s) => s,
        Err(e) => return reply(e),
    };
    let wire_opt = wire.map_or(String::new(), |w| format!(" /Universe {w}"));
    let again = match mode {
        Direction::Out => format!("Assign {} At {protocol} {line}{wire_opt}", run_text(ids)),
        Direction::In => format!("Assign {protocol} {line} At {}{wire_opt}", run_text(ids)),
    };
    let show = &state.root.show_data;
    if show.protocols.line(protocol, line).is_none() {
        // A new line: on the show's only network (or Default), else the operator picks one.
        let nets: Vec<String> = show.protocols.networks.values().filter_map(|n| n.meta.identity.name.clone()).collect();
        if nets.len() > 1 {
            let mut ask = crate::dialogs::Ask::new(
                format!("New {protocol} {line}"),
                format!("{protocol} {line} doesn't exist yet. Which network is it on?"),
            );
            for n in nets.iter().take(6) {
                ask = ask.button(n, lumacore_wire::ButtonKind::Plain, format!("Store {protocol} {line} /Network \"{n}\"; {again}"));
            }
            return crate::DispatchResult::Ask(Box::new(ask.cancel("Cancel")));
        }
        let network = nets.into_iter().next().unwrap_or_else(|| lumacore_protocols::DEFAULT_NETWORK.to_string());
        if LineOps::create_line(&mut state.root.show_data, protocol, mode, Some(line), None, &network).is_none() {
            return reply(render_code(format!("This build has no protocol {protocol}."), "E-CMD-007"));
        }
    } else if let Some(l) = show.protocols.line(protocol, line)
        && l.amount > 0
    {
        let mut wanted: Vec<u32> = ids.to_vec();
        wanted.sort_unstable();
        let same = l.mode == mode
            && wanted.first() == Some(&l.local_universe)
            && u32::try_from(wanted.len()).ok() == Some(l.amount);
        if !same {
            let next = show.protocols.node(protocol).and_then(|n| n.lines.keys().next_back()).map_or(1, |n| n + 1);
            let again_next = match mode {
                Direction::Out => format!("Assign {} At {protocol} {next}{wire_opt}", run_text(ids)),
                Direction::In => format!("Assign {protocol} {next} At {}{wire_opt}", run_text(ids)),
            };
            let ask = crate::dialogs::Ask::new(
                format!("{protocol} {line}"),
                format!("{protocol} {line} carries {} ({}). A line carries one run.", l.run_label(), l.mode.label()),
            )
            .button("Replace", lumacore_wire::ButtonKind::Primary, format!("Unassign {protocol} {line}; {again}"))
            .button(
                &format!("New line {protocol} {next}"),
                lumacore_wire::ButtonKind::Plain,
                format!("Copy {protocol} {line} At {next}; Unassign {protocol} {next}; {again_next}"),
            )
            .cancel("Cancel");
            return crate::DispatchResult::Ask(Box::new(ask));
        }
    }
    let mut events = Vec::new();
    let result = LineOps::set_run(&mut state.root.show_data, protocol, line, mode, ids, start, &mut events);
    for ev in events {
        state.root.emit(&ev);
    }
    match result {
        Ok(_) => {
            let show = &state.root.show_data;
            let l = show.protocols.line(protocol, line).expect("assigned");
            let network = show.protocols.network_name(l.network).to_string();
            let mut text = render_success(&format!("{protocol} {line}: {} ({})", l.run_label(), l.mode.label()));
            if !state.root.station_settings.networks.contains_key(&network.to_lowercase()) {
                text.push_str(&format!(
                    "\nNetwork \"{network}\" isn't mapped on this station: nothing is {} until \
                     Set Network \"{network}\" \"Interface\" <nic> (List Network shows the interfaces).",
                    match mode {
                        Direction::Out => "sent",
                        Direction::In => "received",
                    }
                ));
            }
            reply(text)
        }
        Err(e) => reply(crate::errors::render(&e)),
    }
}

/// Take universes `ids` off the lines that carry them: one line (`At Sacn 1`) or every line.
pub fn unassign_universes(state: &mut AppState, ids: &[u32], line: Option<(&str, u32)>) -> String {
    let targets: Vec<(String, u32)> = match line {
        Some((p, n)) => vec![(p.to_string(), n)],
        None => {
            let mut all: Vec<(String, u32)> = ids
                .iter()
                .flat_map(|&u| LineOps::lines_on(&state.root.show_data, u))
                .map(|(p, n, _, _)| (p, n))
                .collect();
            all.sort();
            all.dedup();
            all
        }
    };
    let mut removed = 0;
    for (protocol, no) in targets {
        match LineOps::take_off(&mut state.root.show_data, &protocol, no, ids) {
            Ok(n) => removed += n,
            Err(e) => return crate::errors::render(&e),
        }
    }
    render_success(&format!("Took {removed} universe{} off", if removed == 1 { "" } else { "s" }))
}

/// Handle: Delete Universe <n> [Thru <m>] (the one range rule, decisions §2)
///
/// Removes each universe from show data; one with patched fixtures stays
/// (named, with its fixtures).
pub fn handle_delete_universe(state: &mut AppState, args: &[&str]) -> String {
    let usage = "Usage: Delete Universe <n> [Thru <m>]";
    if args.is_empty() {
        return render_code(usage, "E-CMD-006");
    }
    let Some(ids) = crate::parser::number_list(args, &state.root.show_data.devices.dmx.patch.universes) else {
        return render_code(usage, "E-CMD-006");
    };
    if ids.is_empty() {
        return render_code(format!("Universe {} not found.", args.join(" ")), "E-UNIVERSE-001");
    }
    delete_universes(state, &ids)
}

/// Delete these universes; one in use is refused, the others go.
pub fn delete_universes(state: &mut AppState, ids: &[u32]) -> String {
    let mut lines = Vec::new();
    let mut deleted = Vec::new();
    let mut unpatched: Vec<String> = Vec::new();
    for &id in ids {
        let Some(guid) = state.root.show_data.universe_guid(id) else { continue };
        // The fixtures on it lose their address there and stay in the patch (decisions §11
        // "Issue 11 hardening" 5: the one Delete question named them); the lines stay on the slot.
        for f in state.root.show_data.devices.dmx.patch.fixtures.values_mut() {
            let before = f.mvr.breaks.len();
            f.mvr.breaks.retain(|b| b.universe_id != guid);
            if f.mvr.breaks.len() != before {
                unpatched.push(f.handle());
            }
        }
        state.root.show_data.devices.dmx.patch.universes.remove(&id);
        deleted.push(id.to_string());
    }
    if !unpatched.is_empty() {
        state.root.show_data.rebuild_occupancy();
        state.patch_generation += 1;
    }
    if !deleted.is_empty() {
        let s = if deleted.len() == 1 { "" } else { "s" };
        lines.push(render_success(&format!("Deleted Universe{s} {}.", deleted.join(", "))));
    }
    if !unpatched.is_empty() {
        lines.push(format!("Unpatched there (no address now): Fixture {}.", unpatched.join(", ")));
    }
    lines.join("\n")
}
