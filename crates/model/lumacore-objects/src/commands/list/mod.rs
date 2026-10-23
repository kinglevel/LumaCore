//! List command handler: `List [target] [/r]` — console-style data object listing.
//!
//! - Bare `list`: show overview summary (fixtures, universes, groups, etc.)
//! - `list /r`: recursive tree view (shows everything)
//! - `list fixture`: list all patched fixtures
//! - `list fixture 1`: show details of fixture 1
//! - `list root.showdata`: show direct children of ShowData (dot-path tree browser)
//! - `list library [type]`: browse the library (GDTF files, presets, etc.)
//!
//! Note: For tree navigation, use `ls` instead. `list` is for data objects.

mod list_fixtures;
mod list_misc;
mod list_runtime;
mod list_tree;
mod list_universes;

use crate::tree_path::Node;
use crate::AppState;
use crate::errors::render_code;

use list_fixtures::{
    list_fixture_detail, list_fixture_type_detail, list_fixture_types, list_fixtures,
    list_gdtf_files,
};
use list_misc::{list_library_all, list_selection, list_watches};
use list_runtime::{list_clients, list_lumasession, list_shells, list_stations};
use list_tree::{list_context, list_dot_path, list_patch_children};
pub use list_tree::list_show_file;
pub use list_universes::route_label;
use list_universes::{list_lines, list_networks, list_universes};

/// `/r` (recursive) anywhere in the arguments.
pub fn recursive(args: &[&str]) -> bool {
    crate::commands::flags::has(args, &crate::commands::flags::RECURSIVE)
}

/// The arguments without the `/r` flag.
pub fn plain<'a>(args: &[&'a str]) -> Vec<&'a str> {
    args.iter()
        .filter(|a| !crate::commands::flags::is(a, &crate::commands::flags::RECURSIVE))
        .copied()
        .collect()
}

/// The numeric id in the first argument, if any.
fn id(args: &[&str]) -> Option<u32> {
    plain(args).first().and_then(|a| a.parse().ok())
}

/// Handle: List [target] [/r]
///
/// `List <object> …` is routed to the object's own List handler by dispatch
/// (objects publish themselves); this handles everything that isn't an
/// object: bare `list` (context), dot-paths (`list root.showdata`) and tree
/// nodes by name (`list patch`, `list playback`).
pub fn handle_list(state: &AppState, args: &[&str]) -> String {
    let recursive = recursive(args);
    let args = plain(args);
    let Some(target) = args.first().map(|s| s.to_lowercase()) else {
        return list_context(state, recursive);
    };
    if target.contains('.') {
        return list_dot_path(state, &target, recursive);
    }
    match tree_node(&target) {
        Some(Node::Patch) if !recursive => list_patch_children(state),
        Some(node) => list_dot_path(state, &node.path(), recursive),
        None => {
            let objects: Vec<&str> =
                crate::objects::supporting(crate::objects::Verb::List)
                    .map(|o| o.keyword)
                    .collect();
            render_code(format!(
                "Unknown list target '{target}'.\n\
                 Use: {}\n\
                 Or a tree node / dot-path: Root.ShowData, Root.Runtime, Patch, Playback",
                objects.join(", ")
            ), "E-CMD-001")
        }
    }
}

/// A tree node named `name` near the top of the tree (`root`, `patch`, `playback`,
/// `runtime`, …).
fn tree_node(name: &str) -> Option<Node> {
    if name.eq_ignore_ascii_case("root") {
        return Some(Node::Root);
    }
    // The names that lead to a node from anywhere (`patch`), then the top nodes' children.
    crate::tree_path::alias(name).or_else(|| {
        [Node::Root, Node::ShowData, Node::Devices, Node::Library]
            .into_iter()
            .find_map(|parent| parent.child(name))
    })
}

// ── Per-object lists (each object's List handler calls one of these) ────────

/// The listed fixtures narrowed to `nos` (a node's fixtures).
pub fn fixtures_of(state: &AppState, nos: &[u32]) -> String {
    list_fixtures::list_fixtures_of(state, nos)
}

pub fn fixtures(state: &AppState, args: &[&str]) -> String {
    match id(args) {
        Some(id) => list_fixture_detail(state, id),
        None => list_fixtures(state),
    }
}

pub fn fixture_types(state: &AppState, args: &[&str]) -> String {
    match id(args) {
        Some(id) => list_fixture_type_detail(state, id),
        None => list_fixture_types(state),
    }
}

pub fn universes(state: &AppState, args: &[&str]) -> String {
    match id(args) {
        Some(id) => list_universes::list_universe_detail(state, id),
        None => list_universes(state),
    }
}

/// `List Sacn` / `List ArtNet` / `List Osc`: one protocol's lines.
pub fn lines(state: &AppState, protocol: &str) -> String {
    list_lines(state, protocol)
}

pub fn networks(state: &AppState, _args: &[&str]) -> String {
    list_networks(state)
}

/// Library browser: `List Library [gdtf]`.
pub fn library(state: &AppState, args: &[&str]) -> String {
    let sub = plain(args)
        .first()
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    match sub.as_str() {
        "" => list_library_all(state),
        "gdtf" => list_gdtf_files(state),
        _ => render_code(format!("Unknown library type '{sub}'. Available: gdtf"), "E-CMD-008"),
    }
}

pub fn settings(state: &AppState, _args: &[&str]) -> String {
    super::handle_list_settings(state)
}

pub fn watches(state: &AppState, _args: &[&str]) -> String {
    list_watches(state)
}

pub fn claims(state: &AppState, _args: &[&str]) -> String {
    super::list_claims(state)
}

pub fn selection(state: &AppState, _args: &[&str]) -> String {
    list_selection(state)
}

pub fn undo(state: &AppState, args: &[&str]) -> String {
    super::oops::cmd_list_undo(state, args)
}

pub fn local_time(ms: u64) -> String {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_millis_opt(ms as i64)
        .single()
        .map(|t| t.format("%H:%M:%S%.3f").to_string())
        .unwrap_or_else(|| "?".to_string())
}

/// `List Journal [n]` — the last changes (newest last; default 20).
pub fn journal(state: &AppState, args: &[&str]) -> String {
    use crate::text::PropertySheet;
    let journal = &state.root.runtime.journal;
    let count = plain(args)
        .first()
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(20);
    let mut sheet = PropertySheet::new(&format!(
        "Journal ({} of {} kept, last {})",
        count.min(journal.len()),
        journal.len(),
        lumacore_tree::runtime::JOURNAL_CAPACITY
    ));
    if journal.is_empty() {
        return sheet.line("  No changes yet.").build();
    }
    for e in journal.entries().skip(journal.len().saturating_sub(count)) {
        let changed = if e.changed.is_empty() {
            "nothing changed".to_string()
        } else {
            e.changed.join(", ")
        };
        sheet = sheet.line(&format!(
            "  #{:<5} {}  {:<10} {}  → {} ({})",
            e.seq,
            local_time(e.time_ms),
            e.user,
            e.command,
            changed,
            e.result
        ));
    }
    sheet.build()
}

/// `List Health` / `Info Health [Render|Lines|Clock|Threads]` —
/// `Runtime.Health` (decisions §24 "Step 9 hardening"): the render engine
/// stage by stage, the output lines' rates, the session clock and every
/// long-lived thread.
pub fn health(state: &AppState, args: &[&str]) -> String {
    use crate::text::PropertySheet;
    let v = state.root.runtime.health.view();
    let only = plain(args).first().map(|a| a.to_ascii_lowercase());
    let wants = |part: &str| only.as_deref().is_none_or(|o| o == part);
    let mut sheet = PropertySheet::new("Health");
    if wants("render") {
        let r = &v.render;
        sheet = sheet
            .section("Render")
            .field("Rate", &format!("{} Hz ({} µs per frame)", r.rate_hz, r.period_us))
            .field("Frame", &format!("{} µs (max {} µs in the last second)", r.frame_us, r.frame_max_us))
            .field("Headroom", &format!("{} µs (min {} µs)", r.headroom_us, r.headroom_min_us))
            .field("Jitter", &format!("{} µs", r.jitter_us))
            .field(
                "Late",
                &format!(
                    "{} late · {} skipped · {} overload episode{}{}",
                    r.late,
                    r.skipped,
                    r.overload_episodes,
                    if r.overload_episodes == 1 { "" } else { "s" },
                    if r.overload { " · OVERLOAD now" } else { "" }
                ),
            )
            .field(
                "Panics",
                &format!(
                    "{} · restarts {}{}",
                    r.panics,
                    r.restarts,
                    if r.down { " · DOWN, the lines hold the last frame" } else { "" }
                ),
            )
            .field(
                "Plan",
                &format!(
                    "#{} · {} channels · {} universes · compile {} + {} µs ({} compiles)",
                    r.plan_generation,
                    r.channels,
                    r.universes,
                    r.compile_channel_us,
                    r.compile_value_us,
                    r.compiles
                ),
            )
            .field("Frames", &format!("{} (busy {})", r.frames, r.busy));
        for s in &r.stages {
            sheet = sheet.line(&format!(
                "  {:<11} at {:>5} µs  took {:>5} µs  (max {:>5} µs)",
                s.name, s.start_us, s.last_us, s.max_us
            ));
        }
    }
    if wants("lines") {
        sheet = sheet.section("Lines");
        if v.lines.is_empty() {
            sheet = sheet.line("  No output line is sending.");
        }
        for l in &v.lines {
            sheet = sheet.line(&format!(
                "  Output {:<3} {:<7} rate {} Hz (used {}, actual {})  send {} µs (max {})  packets {}  errors {}  \
                 frames sent twice {}  skipped {}",
                l.line,
                l.protocol,
                l.rate_hz,
                l.effective_hz,
                l.actual_hz,
                l.send_us,
                l.send_max_us,
                l.packets,
                l.errors,
                l.doubled,
                l.skipped
            ));
        }
    }
    if wants("clock") {
        let c = &v.clock;
        sheet = sheet.section("Clock").field(
            &format!("{} · {}", c.role, c.state),
            &format!(
                "offset {:+} µs · path delay {} µs · jitter {} µs · {:+.3} ppm · correction {:+.1} ppm · {} used / {} rejected · {} steps",
                c.offset_us,
                c.path_delay_us,
                c.jitter_us,
                c.frequency_ppb as f64 / 1000.0,
                c.correction_ppb as f64 / 1000.0,
                c.samples_used,
                c.samples_rejected,
                c.steps
            ),
        );
        for m in &c.members {
            sheet = sheet.line(&format!(
                "  {:<14} {:<9} offset {:+} µs · path delay {} µs · jitter {} µs · {:+.3} ppm",
                m.station,
                m.state,
                m.offset_us,
                m.path_delay_us,
                m.jitter_us,
                m.frequency_ppb as f64 / 1000.0
            ));
        }
        sheet = match (&c.ptp, &c.ptp_note) {
            (Some(p), _) => sheet.field(
                "PTP",
                &format!("{} · offset {:+} ns · path delay {} ns · GM {}", p.port_state, p.offset_ns, p.path_delay_ns, p.grandmaster),
            ),
            (None, Some(note)) => sheet.field("PTP", note),
            (None, None) => sheet,
        };
    }
    if wants("threads") {
        sheet = sheet.section("Threads");
        if v.threads.is_empty() {
            sheet = sheet.line("  No threads registered (this runtime is not a server).");
        }
        for t in v.threads {
            sheet = sheet.line(&format!(
                "  {:<22} {:<7} last {}  loops {:<8} loop {} µs (max {})  queue {}  refused {}  panics {}",
                t.name,
                if t.alive { "alive" } else { "stopped" },
                local_time(t.last_activity_ms),
                t.iterations,
                t.last_loop_us,
                t.max_loop_us,
                t.queue_depth,
                t.refused,
                t.panics
            ));
            // Real-time threads say how the OS schedules them (decisions §31).
            if !t.scheduling.is_empty() {
                sheet = sheet.line(&format!("  {:<22} scheduling: {}", "", t.scheduling));
            }
        }
    }
    sheet.build()
}

/// User profiles (the workspaces login users point at).
pub fn profiles(state: &AppState, _args: &[&str]) -> String {
    list_misc::list_users(state)
}

pub fn shells(state: &AppState, _args: &[&str]) -> String {
    list_shells(state)
}

pub fn lumasessions(state: &AppState, _args: &[&str]) -> String {
    list_lumasession(state)
}

pub fn interfaces(state: &AppState, _args: &[&str]) -> String {
    list_runtime::list_network(state)
}

pub fn stations(state: &AppState, _args: &[&str]) -> String {
    let mut out = list_stations(state);
    // The station table (decisions §30 "12d part 4"): each row's wiring.
    let table = &state.root.show_data.stations;
    if !table.is_empty() {
        let nets = crate::stations::network_names(state);
        let own = crate::stations::own_row(state);
        out.push_str("\n\nStation table (wiring, in the show):");
        for (no, r) in table {
            let wiring: Vec<String> = r
                .networks
                .iter()
                .map(|(g, nic)| format!("{} → {nic}", nets.get(g).map_or("?", |(_, n)| n.as_str())))
                .collect();
            out.push_str(&format!(
                "\n  {no:>3}  {:<16} {:<18} {}",
                r.meta.name().unwrap_or_default(),
                if own == Some(*no) { "this station" } else if r.station_id.is_empty() { "made in advance" } else { "" },
                if wiring.is_empty() { "—".to_string() } else { wiring.join(", ") }
            ));
        }
    }
    out
}

pub fn clients(state: &AppState, _args: &[&str]) -> String {
    list_clients(state)
}

/// A list that is a tree node: the node's children (`/r` for the subtree).
/// The list of a tree node (an object's List): a collection inside a datapool is the
/// acting user's selected datapool's (decisions §29).
pub fn tree(state: &AppState, node: Node, args: &[&str]) -> String {
    let path = match node.is_under(Node::DataPool) {
        true => node.path().replacen(".DataPool.", &format!(".{}.", state.selected_datapool()), 1),
        false => node.path(),
    };
    list_dot_path(state, &path, recursive(args))
}

/// Count .gdtf files in library/gdtf/ on disk.
pub fn count_library_gdtf() -> usize {
    let dir = std::path::Path::new("library/gdtf");
    if !dir.is_dir() {
        return 0;
    }
    lumacore_luma::drive::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| crate::commands::import::is_gdtf(&e.path()))
                .count()
        })
        .unwrap_or(0)
}
