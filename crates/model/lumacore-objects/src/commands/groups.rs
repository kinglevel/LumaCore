//! Groups (decisions §22): DataPool groups of the user's selected datapool —
//! their members in selection order with grid spots, the selection rule,
//! the grid cursor, Store options, group masters and the Group object.
//!
//! - `Group 3` / `Group 3 + Group 4 - Fixture 2` / `Group 1 Thru 3` — select
//!   (a plain select adds until a value was set, then replaces; the usual console convention)
//! - `+ …` / `- …` — always add / take out
//! - `Group 3 At 50` — select, then set the values
//! - `Master Group 3 At 50 Fade 2` — the group master (mode on the group,
//!   level in `ShowData.Playbacks`)
//! - `Store Group 3 ["name"] [/Merge | /Overwrite | /Remove] [/GridMerge AppendX|Off]`
//! - `Delete Group 3 [Thru 5] [/Force]`, `Copy Group 1 At 5 [/Merge | /Overwrite]`,
//!   `Update Group 3`, `Move Group 3 At 7`, `Label`, `Edit`
//! - `Set Group 3 "Mode"|"PreserveGrid"|"MoveGridCursor"|"Name" <value>` (a master's level is
//!   `Master Group 3 At 50`), a member's grid spot `Set Group 3 Fixture 12 "X" 4`
//! - `Grid 2/0` — move the grid cursor

pub mod context;

use crate::commands::flags as F;
use nu_ansi_term::Color;
use tabled::settings::Style as TableStyle;
use tabled::{Table, Tabled};
use uuid::Uuid;

use lumacore_tree::datapool::{Datapool, Group};
use lumacore_tree::programmer::SelectMode;
use lumacore_tree::{GridCell, GridCursorMove, GroupMasterMode, MasterLevel, TreeNode};

use super::control::error_reply;
use crate::AppState;
use lumacore_wire::ButtonKind;
use crate::DispatchResult;
use crate::errors::render_code;
use crate::objects::{ObjectDef, Verb, out};
use crate::pools::{CellBar, PoolAction, PoolCell, PoolContext, PoolDef};
use crate::text::{PropertySheet, render_success};

// ── Lookup ───────────────────────────────────────────────────────────────────

fn ensure_datapool(state: &mut AppState, dp_id: u32) {
    state
        .root
        .show_data
        .datapools
        .entry(dp_id)
        .or_insert_with(|| Datapool::new(Some("Default".to_string())));
}

/// The groups of the user's selected datapool, created when missing.
fn groups_mut(state: &mut AppState) -> &mut std::collections::BTreeMap<u32, Group> {
    let dp_id = state.selected_datapool();
    ensure_datapool(state, dp_id);
    &mut state
        .root
        .show_data
        .datapools
        .get_mut(&dp_id)
        .expect("ensure_datapool guarantees existence")
        .groups
}

/// A group number: 1–9999 for a new one; a group an older show already has
/// above the cap is named as it is (only new numbers are capped).
fn parse_id(state: &AppState, raw: &str) -> Result<u32, String> {
    raw.parse::<u32>()
        .ok()
        .filter(|&n| n >= 1 && (n <= crate::pools::MAX_NO || state.user_groups().contains_key(&n)))
        .ok_or_else(|| format!("Invalid group number '{raw}': groups are 1–{}.", crate::pools::MAX_NO))
}

/// Group numbers named by `args` (`3`, `3 Thru 5`, `Thru`, `1 Thru 9 - 4`):
/// the one range rule (decisions §2, `parser::number_list`) over the user's
/// groups — missing ones are skipped, none at all is an error. Groups above
/// the cap that an older show has are named like any other (only new numbers
/// are capped).
/// Errors come coded (E-CMD-006 / E-GROUP-003).
fn parse_ids(state: &AppState, args: &[&str]) -> Result<Vec<u32>, String> {
    if args.is_empty() {
        return Err(render_code("Which group?", "E-CMD-006"));
    }
    let groups = state.user_groups();
    let Some(ids) = crate::parser::number_list(args, groups) else {
        return Err(render_code(format!("'{}' isn't a group number or range.", args.join(" ")), "E-CMD-006"));
    };
    if ids.is_empty() {
        return Err(render_code(format!("Group {} not found.", args.join(" ")), "E-GROUP-003"));
    }
    Ok(ids)
}

fn not_found(id: u32) -> String {
    render_code(format!("Group {id} not found."), "E-GROUP-003")
}

/// `Store Group … /GridMerge AppendX|Off` (decisions §31: declared on the Group object).
const GRID_MERGE: F::FlagDef =
    F::FlagDef::new("GridMerge", "Where merged members go in the grid").value("AppendX|Off").choices(&["AppendX", "Off"]);

/// The plain arguments: without flags and /GridMerge's value.
fn plain<'a>(args: &[&'a str]) -> Vec<&'a str> {
    F::positional(args, &[&GRID_MERGE])
}

// ── Selecting ────────────────────────────────────────────────────────────────

/// Select group `id` of the user's datapool by `how`, with its grid spots.
/// Members whose fixture was deleted are skipped (their links stay).
pub fn select_group(
    state: &mut AppState,
    id: u32,
    how: SelectMode,
) -> Result<usize, String> {
    let group = state
        .user_groups()
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found(id))?;
    let live: Vec<_> = group
        .members
        .iter()
        .filter(|m| {
            state
                .root
                .show_data
                .fixture_id_for_guid(m.fixture)
                .is_some()
        })
        .copied()
        .collect();
    let ids: Vec<u32> = live
        .iter()
        .filter_map(|m| state.root.show_data.fixture_id_for_guid(m.fixture))
        .collect();
    crate::programmers::ensure_unclaimed(state, &ids)?;
    let fixtures: Vec<Uuid> = live.iter().map(|m| m.fixture).collect();
    let cells: Vec<GridCell> = live.iter().map(|m| m.cell).collect();
    state.select_members(
        &fixtures,
        &cells,
        how,
        group.preserve_grid,
        group.move_grid_cursor,
    )
}

enum Term<'a> {
    Groups(Vec<u32>),
    GroupByName(String),
    Fixtures(Vec<&'a str>),
}

/// One term of a selection line: `Group 3`, `Group 1 Thru 3`, `Group "Wash"`,
/// `Fixture 2 Thru 5` or (with `default_group`) bare numbers.
fn term<'a>(state: &AppState, tokens: &[&'a str], default_group: bool) -> Result<Term<'a>, String> {
    let (is_group, rest) = match tokens.first() {
        Some(w) if w.eq_ignore_ascii_case("group") => (true, &tokens[1..]),
        Some(w) if w.eq_ignore_ascii_case("fixture") => (false, &tokens[1..]),
        _ => (default_group, tokens),
    };
    if !is_group {
        return Ok(Term::Fixtures(rest.to_vec()));
    }
    match rest.first() {
        Some(w) if w.starts_with('"') => Ok(Term::GroupByName(w.trim_matches('"').to_string())),
        Some(_) => Ok(Term::Groups(parse_ids(state, rest)?)),
        None => Err("Which group?".to_string()),
    }
}

/// Split a selection line at its top-level `+` / `-`.
fn terms<'a>(tokens: &[&'a str]) -> Vec<(char, Vec<&'a str>)> {
    let mut out: Vec<(char, Vec<&'a str>)> = vec![(' ', Vec::new())];
    for &t in tokens {
        match t {
            "+" => out.push(('+', Vec::new())),
            "-" => out.push(('-', Vec::new())),
            _ => out.last_mut().expect("never empty").1.push(t),
        }
    }
    out.retain(|(_, t)| !t.is_empty());
    out
}

/// Select by a selection line. The first term is selected by `first`
/// (Plain: the selection rule), `+` terms add, `-` terms take out.
/// `default_group`: bare numbers name groups (the line started with Group).
fn select_line(
    state: &mut AppState,
    tokens: &[&str],
    first: SelectMode,
    default_group: bool,
) -> Result<usize, String> {
    let parts = terms(tokens);
    if parts.is_empty() {
        return Err("Select what?".to_string());
    }
    let mut count = state
        .current_programmer()
        .map_or(0, |p| p.selected_fixtures.len());
    for (i, (op, words)) in parts.iter().enumerate() {
        let default_group = if i == 0 { default_group } else { false };
        let how = if i == 0 && *op == ' ' {
            first
        } else {
            SelectMode::Add
        };
        let deselect = *op == '-';
        let term = term(state, words, default_group)?;
        let fixtures: Vec<Uuid> = match &term {
            Term::Groups(ids) if deselect => ids
                .iter()
                .filter_map(|id| state.user_groups().get(id))
                .flat_map(|g| g.fixtures())
                .collect(),
            Term::GroupByName(name) if deselect => group_by_name(state, name)
                .map(|(_, g)| g.fixtures())
                .unwrap_or_default(),
            Term::Fixtures(words) => {
                let ids = resolve_fixtures(state, words)?;
                if !deselect {
                    count = state.select_fixtures(&ids, how)?;
                    continue;
                }
                state.root.show_data.fixture_guids(&ids)
            }
            Term::Groups(ids) => {
                // The one range rule (R1-35, Q6): groups that don't exist
                // are skipped; only none at all is an error (nothing was
                // selected then, so nothing changed).
                let existing: Vec<u32> = ids.iter().copied().filter(|id| state.user_groups().contains_key(id)).collect();
                if existing.is_empty() {
                    return Err(not_found(ids.first().copied().unwrap_or(0)));
                }
                for (n, &id) in existing.iter().enumerate() {
                    count = select_group(state, id, if n == 0 { how } else { SelectMode::Add })?;
                }
                continue;
            }
            Term::GroupByName(name) => {
                let id = group_by_name(state, name)
                    .map(|(no, _)| no)
                    .ok_or_else(|| {
                        render_code(format!("No group named \"{name}\"."), "E-GROUP-003")
                    })?;
                count = select_group(state, id, how)?;
                continue;
            }
        };
        if let Some(prog) = state.current_programmer_mut() {
            prog.deselect_all(&fixtures);
            count = prog.selected_fixtures.len();
        }
    }
    Ok(count)
}

fn group_by_name<'a>(state: &'a AppState, name: &str) -> Option<(u32, &'a Group)> {
    state
        .user_groups()
        .iter()
        .find(|(_, g)| g.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(name)))
        .map(|(no, g)| (*no, g))
}

/// Fixture ids named by selection words (`2 Thru 5`, `Fixture "Spot 1"`, `Thru`) — the one
/// resolver (decisions §33 57).
fn resolve_fixtures(state: &AppState, words: &[&str]) -> Result<Vec<u32>, String> {
    crate::resolve::fixture_words(state, words)
}

fn selected_text(count: usize) -> String {
    render_success(&format!(
        "Selected {count} fixture{}",
        if count == 1 { "" } else { "s" }
    ))
}

/// `+ <selection>` — always adds (bare numbers are fixtures).
pub fn cmd_plus(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        return render_code("Usage: + Group 3 | + Fixture 4 Thru 6", "E-CMD-006");
    }
    match select_line(state, args, SelectMode::Add, false) {
        Ok(n) => selected_text(n),
        Err(e) => error_reply(e, "E-CMD-006"),
    }
}

/// `- <selection>` — take out of the selection.
pub fn cmd_minus(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        return render_code("Usage: - Fixture 2 | - Group 4", "E-CMD-006");
    }
    let mut line = vec!["-"];
    line.extend_from_slice(args);
    // A leading `-` term: everything deselects; keep the current selection.
    let parts = terms(&line);
    let mut count = state
        .current_programmer()
        .map_or(0, |p| p.selected_fixtures.len());
    for (_, words) in parts {
        let fixtures: Vec<Uuid> = match term(state, &words, false) {
            Ok(Term::Groups(ids)) => ids
                .iter()
                .filter_map(|id| state.user_groups().get(id))
                .flat_map(|g| g.fixtures())
                .collect(),
            Ok(Term::GroupByName(name)) => group_by_name(state, &name)
                .map(|(_, g)| g.fixtures())
                .unwrap_or_default(),
            Ok(Term::Fixtures(words)) => match resolve_fixtures(state, &words) {
                Ok(ids) => state.root.show_data.fixture_guids(&ids),
                Err(e) => return error_reply(e, "E-CMD-007"),
            },
            Err(e) => return error_reply(e, "E-CMD-006"),
        };
        if let Some(prog) = state.current_programmer_mut() {
            prog.deselect_all(&fixtures);
            count = prog.selected_fixtures.len();
        }
    }
    selected_text(count)
}

/// `Grid [x [y [z]]]` / `Grid x/y[/z]` — show or move the grid cursor.
pub fn cmd_grid(state: &mut AppState, args: &[&str]) -> String {
    // `Grid Group 3 "RemoveGaps"`: a tool on a group's grid (§30 "12e").
    if let Some(out) = crate::tools::run_line(state, args) {
        return out;
    }
    let Some(prog) = state.current_programmer_mut() else {
        return render_code("No programmer available.", "E-CMD-011");
    };
    if args.is_empty() {
        let c = prog.grid_cursor;
        return format!("Grid cursor {}/{}/{}", c.x, c.y, c.z);
    }
    let numbers: Result<Vec<i32>, _> = args
        .iter()
        .flat_map(|a| a.split('/'))
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<i32>())
        .collect();
    let Ok(numbers) = numbers else {
        return render_code("Usage: Grid <x> [<y> [<z>]]  (or Grid x/y/z)", "E-CMD-006");
    };
    if numbers.is_empty() || numbers.len() > 3 {
        return render_code("Usage: Grid <x> [<y> [<z>]]  (or Grid x/y/z)", "E-CMD-006");
    }
    prog.grid_cursor = GridCell {
        x: numbers[0],
        y: numbers.get(1).copied().unwrap_or(prog.grid_cursor.y),
        z: numbers.get(2).copied().unwrap_or(prog.grid_cursor.z),
        ..GridCell::default()
    };
    let c = prog.grid_cursor;
    render_success(&format!("Grid cursor {}/{}/{}", c.x, c.y, c.z))
}

// ── Group (called) ───────────────────────────────────────────────────────────

/// `Group …` — select, `… At <v>` sets values (a group's master is `Master Group 3 At 50`,
/// decisions §33 90).
pub fn cmd_group(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        return list_groups(state);
    }
    let at = args.iter().position(|w| w.eq_ignore_ascii_case("at"));
    let (sel, value) = match at {
        Some(p) => (&args[..p], Some(&args[p + 1..])),
        None => (args, None),
    };
    let count = match select_line(state, sel, SelectMode::Plain, true) {
        Ok(n) => n,
        Err(e) => return error_reply(e, "E-CMD-006"),
    };
    match value {
        Some(v) => super::control::cmd_at(state, v),
        None => selected_text(count),
    }
}

// ── Group master ─────────────────────────────────────────────────────────────

/// A group's master: shown (no args), or `At 50 [Fade 2]` (`Master Group 3 At 50 Fade 2`).
pub fn master(state: &mut AppState, id: u32, args: &[&str]) -> String {
    let Some(group) = state.user_groups().get(&id) else {
        return not_found(id);
    };
    let (guid, mode) = (group.meta.identity.guid, group.master);
    let now = lumacore_base::clock::session_ms();
    if args.is_empty() {
        let level = state
            .root
            .show_data
            .playbacks
            .live.masters.groups
            .get(&guid)
            .copied()
            .unwrap_or_default();
        return format!(
            "Group {id} master: {} at {:.0}%",
            mode.label(),
            level.at(now) * 100.0
        );
    }
    if !args[0].eq_ignore_ascii_case("at") {
        return render_code("Usage: Group <n> Master At <0-100> [Fade <s>]", "E-CMD-006");
    }
    // Masters take relative input (decisions §30 "12f"): `At + 10` from where it stands.
    let now_pct = state
        .root
        .show_data
        .playbacks
        .live
        .masters
        .groups
        .get(&guid)
        .copied()
        .unwrap_or_default()
        .at(now)
        * 100.0;
    let (level, used) = match crate::commands::playback::level_after_at(&args[1..], now_pct) {
        Ok((pct, used)) => (pct / 100.0, used),
        Err(e) => return e,
    };
    let fade_at = 1 + used;
    let fade_s = match args.get(fade_at) {
        Some(w) if w.eq_ignore_ascii_case("fade") => {
            match args.get(fade_at + 1).and_then(|s| s.parse::<f64>().ok()) {
                // Finite and bounded like a cue time (R1-05): `inf` or a
                // huge number would overflow the fade in the render tick.
                Some(s) if (0.0..=3600.0).contains(&s) => s,
                Some(_) => return render_code("A master fade is 0–3600 s.", "E-CMD-008"),
                None => return render_code("Fade: seconds, e.g. Fade 2", "E-CMD-008"),
            }
        }
        Some(other) => {
            return render_code(
                format!("Unexpected '{other}'. Usage: … At 50 Fade 2"),
                "E-CMD-006",
            );
        }
        None => 0.0,
    };
    set_master_level(state, guid, level, (fade_s * 1000.0) as u64, now);
    let note = if mode == GroupMasterMode::None {
        " (mode None: no effect until `Set Group … \"Mode\" Positive|Negative|Scaling|Additive`)"
    } else {
        ""
    };
    render_success(&format!(
        "Group {id} master → {:.0}%{}{note}",
        level * 100.0,
        if fade_s > 0.0 {
            format!(" in {fade_s} s")
        } else {
            String::new()
        }
    ))
}

fn set_master_level(state: &mut AppState, group: Uuid, level: f64, fade_ms: u64, now: u64) {
    let entry = state
        .root
        .show_data
        .playbacks
        .live.masters.groups
        .entry(group)
        .or_default();
    if fade_ms == 0 {
        *entry = MasterLevel::snap(level);
    } else {
        entry.go_to(level, fade_ms, now);
    }
}

// ── Store / Update ───────────────────────────────────────────────────────────

/// `Store Group [<n>] ["name"] [/Merge | /Overwrite | /Remove] [/GridMerge AppendX|Off]`.
pub fn store_group(state: &mut AppState, args: &[&str]) -> String {
    let plain = plain(args);
    let append_x = match F::value(args, &GRID_MERGE) {
        None => true,
        Some(v) if v.eq_ignore_ascii_case("appendx") => true,
        Some(v) if v.eq_ignore_ascii_case("off") => false,
        Some(v) => {
            return render_code(format!("/GridMerge '{v}': AppendX or Off."), "E-CMD-008");
        }
    };
    // `Store Group` alone takes the first free number.
    let (id, name) = match plain.first() {
        Some(raw) if raw.starts_with('"') => {
            (first_free(state), Some(raw.trim_matches('"').to_string()))
        }
        Some(raw) => match parse_id(state, raw) {
            Ok(id) => (id, plain.get(1).map(|s| s.trim_matches('"').to_string())),
            Err(e) => return render_code(e, "E-CMD-008"),
        },
        None => (first_free(state), None),
    };
    let mode = if F::has(args, &F::OVERWRITE) {
        StoreMode::Overwrite
    } else if F::has(args, &F::MERGE) {
        StoreMode::Merge
    } else if F::has(args, &F::REMOVE) {
        StoreMode::Remove
    } else {
        StoreMode::Ask
    };
    store_group_at(state, id, name, mode, append_x)
}

/// What a Store over an existing group does.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StoreMode {
    Ask,
    Merge,
    Overwrite,
    Remove,
}

/// Store your selection into group `id` of your DataPool (a new one, or by `mode`).
pub fn store_group_at(state: &mut AppState, id: u32, name: Option<String>, mode: StoreMode, append_x: bool) -> String {
    let members = state
        .current_programmer()
        .map(|p| p.selection_members())
        .unwrap_or_default();
    let count = members.len();
    let groups = groups_mut(state);
    let Some(group) = groups.get_mut(&id) else {
        if mode == StoreMode::Remove {
            return not_found(id);
        }
        // Nothing selected: an empty group, filled later (decisions §30 "Empty pool objects by Store").
        let mut group = Group::new(name);
        group.members = members;
        let shown = group.meta.name().map(|n| format!(" \"{n}\"")).unwrap_or_default();
        groups.insert(id, group);
        return render_success(&if count == 0 {
            format!("Group {id}{shown} created — empty.")
        } else {
            format!("Group {id}{shown} stored — {count} fixtures.")
        });
    };
    let renamed = name.is_some();
    if let Some(n) = name {
        group.meta.identity.name = Some(n);
    }
    // An existing group is never emptied by an empty selection (no silent wipe).
    if members.is_empty() {
        return if renamed {
            render_success(&format!("Group {id} labeled; nothing selected, its fixtures stay."))
        } else {
            render_code(format!("Group {id} exists and nothing is selected — nothing stored."), "E-CMD-009")
        };
    }
    let fixtures: Vec<Uuid> = members.iter().map(|m| m.fixture).collect();
    if mode == StoreMode::Overwrite {
        group.members = members;
        render_success(&format!("Group {id} overwritten — {count} fixtures."))
    } else if mode == StoreMode::Merge {
        let added = group.merge_members(&members, append_x);
        render_success(&format!("Group {id} merged — {added} fixtures added."))
    } else if mode == StoreMode::Remove {
        let removed = group.remove_fixtures(&fixtures);
        render_success(&format!("Group {id} — {removed} fixtures removed."))
    } else {
        // Ask; a one-shot path is told the flags (decisions §30 "12b").
        let had = group.members.len();
        let label = group.meta.name().map(|n| format!(" \"{n}\"")).unwrap_or_default();
        let line = format!("Store Group {id}");
        state.asking = Some(
            crate::dialogs::Ask::new(
                format!("Store Group {id}"),
                format!("Group {id}{label} already has {had} fixtures. Your selection has {count}."),
            )
            .button("Merge", ButtonKind::Primary, format!("{line} /Merge"))
            .button("Overwrite", ButtonKind::Danger, format!("{line} /Overwrite"))
            .button("Remove", ButtonKind::Plain, format!("{line} /Remove"))
            .cancel("Cancel")
            .flags("/Merge, /Overwrite or /Remove"),
        );
        render_code(
            format!("Group {id} exists. Store it with /Merge, /Overwrite or /Remove."),
            "E-GROUP-001",
        )
    }
}

fn first_free(state: &AppState) -> u32 {
    crate::pools::door::next_free(&state.user_groups().keys().copied().collect::<Vec<_>>())
}

// ── Delete / Copy ────────────────────────────────────────────────────────────

// ── Set ──────────────────────────────────────────────────────────────────────

/// A member's grid spot and place in the order, one `key value` pair or more (the
/// registry's member properties write one: `Set Group 3 Fixture 12 "X" 4`).
fn set_member(state: &mut AppState, id: u32, args: &[&str]) -> String {
    let usage = "Usage: Set Group <n> Fixture <f> X|Y|Z|Rotation|MirrorX|MirrorY|Order <value> …";
    let Some(fid) = args.first().and_then(|s| s.parse::<u32>().ok()) else {
        return render_code(usage, "E-CMD-006");
    };
    let Some(fguid) = state.root.show_data.fixture_guid(fid) else {
        return render_code(format!("Fixture {fid} not found."), "E-CMD-007");
    };
    let pairs = &args[1..];
    if pairs.is_empty() || !pairs.len().is_multiple_of(2) {
        return render_code(usage, "E-CMD-006");
    }
    let group = groups_mut(state).get_mut(&id).expect("checked by caller");
    let Some(pos) = group.members.iter().position(|m| m.fixture == fguid) else {
        return render_code(format!("Fixture {fid} is not in Group {id}."), "E-CMD-007");
    };
    let mut order = None;
    for pair in pairs.chunks(2) {
        let (key, raw) = (pair[0].to_ascii_lowercase(), pair[1]);
        let int = || {
            raw.parse::<i32>()
                .map_err(|_| format!("{} '{raw}': a whole number", pair[0]))
        };
        let flag = || match raw.to_ascii_lowercase().as_str() {
            "on" | "yes" | "true" | "1" => Ok(true),
            "off" | "no" | "false" | "0" => Ok(false),
            _ => Err(format!("{} '{raw}': on/off", pair[0])),
        };
        let cell = &mut group.members[pos].cell;
        let result = match key.as_str() {
            "x" => int().map(|v| cell.x = v),
            "y" => int().map(|v| cell.y = v),
            "z" => int().map(|v| cell.z = v),
            "rotation" => raw
                .parse::<f32>()
                .ok()
                .filter(|v| v.is_finite())
                .map(|v| cell.rotation = v.rem_euclid(360.0))
                .ok_or_else(|| format!("Rotation '{raw}': degrees")),
            "mirrorx" => flag().map(|v| cell.mirror_x = v),
            "mirrory" => flag().map(|v| cell.mirror_y = v),
            "order" => raw
                .parse::<usize>()
                .ok()
                .filter(|&n| n >= 1)
                .map(|n| order = Some(n))
                .ok_or_else(|| format!("Order '{raw}': 1 or more")),
            _ => Err(format!("unknown '{}'", pair[0])),
        };
        if let Err(e) = result {
            return render_code(
                format!("Group {id} Fixture {fid}: {e}. {usage}"),
                "E-SET-003",
            );
        }
    }
    if let Some(n) = order {
        let m = group.members.remove(pos);
        let at = (n - 1).min(group.members.len());
        group.members.insert(at, m);
    }
    render_success(&format!("Group {id} Fixture {fid} set."))
}

// ── Edit ─────────────────────────────────────────────────────────────────────

/// Members in order with their grid spots (Edit / Info).
fn member_lines(state: &AppState, group: &Group) -> Vec<String> {
    group
        .members
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let fid = state.root.show_data.fixture_id_for_guid(m.fixture);
            let name = fid
                .and_then(|id| state.root.show_data.devices.dmx.patch.fixture(id))
                .and_then(|f| f.meta.name())
                .unwrap_or("");
            let shown = fid.map_or_else(
                || Color::Red.paint("deleted").to_string(),
                |id| Color::Cyan.paint(id.to_string()).to_string(),
            );
            let c = m.cell;
            let mut extra = String::new();
            if c.rotation != 0.0 {
                extra.push_str(&format!(" rot {}°", c.rotation));
            }
            if c.mirror_x {
                extra.push_str(" mirror X");
            }
            if c.mirror_y {
                extra.push_str(" mirror Y");
            }
            format!(
                "  {:>3}. [{shown}] {name}  grid {}/{}/{}{extra}",
                i + 1,
                c.x,
                c.y,
                c.z
            )
        })
        .collect()
}

/// `Edit Group <n>`: claims it (via `edit_with`) and opens the members and
/// grid editor (`Add Fixture`, `Remove Fixture`, `Set Group … Fixture …`).
pub fn edit_group(state: &mut AppState, id: u32) -> DispatchResult {
    if !state.user_groups().contains_key(&id) {
        return out(not_found(id));
    }
    let sheet = property_sheet(state, id);
    // Owner first (`Edit DataPool 2 Group 3`, R1-55): the editor keeps its
    // DataPool, so edit-mode lines change that pool's group.
    let owner = state.datapool_override.map(|n| format!("DataPool {n} ")).unwrap_or_default();
    state.context = crate::ReplContext::Custom(format!("{owner}Group {id}"));
    out(format!(
        "{sheet}\nEdit mode: Add Fixture <f>, Remove Fixture <f>, \
         Set Group {id} Fixture <f> X|Y|Rotation|MirrorX|Order <v>, Label \"…\", exit"
    ))
}

/// Add a fixture to, or remove it from, a group (edit mode).
pub fn membership(
    state: &mut AppState,
    group_id: u32,
    fixture_id: u32,
    add: bool,
) -> String {
    let Some(guid) = state.root.show_data.fixture_guid(fixture_id) else {
        return render_code(format!("Fixture {fixture_id} not found."), "E-CMD-007");
    };
    let Some(group) = groups_mut(state).get_mut(&group_id) else {
        return not_found(group_id);
    };
    if add {
        let next_x = group
            .members
            .iter()
            .map(|m| m.cell.x + 1)
            .max()
            .unwrap_or(0);
        group.merge_members(
            &[lumacore_tree::GroupMember {
                fixture: guid,
                cell: GridCell {
                    x: next_x,
                    ..GridCell::default()
                },
            }],
            false,
        );
        render_success(&format!("Fixture {fixture_id} added to Group {group_id}."))
    } else {
        group.remove_fixtures(&[guid]);
        render_success(&format!(
            "Fixture {fixture_id} removed from Group {group_id}."
        ))
    }
}

/// `Info Group <n>`.
pub fn property_sheet(state: &AppState, id: u32) -> String {
    let Some(group) = state.user_groups().get(&id) else {
        return not_found(id);
    };
    let level = state
        .root
        .show_data
        .playbacks
        .live.masters.groups
        .get(&group.meta.identity.guid)
        .copied()
        .unwrap_or_default()
        .at(lumacore_base::clock::session_ms());
    let mut sheet = PropertySheet::new(&format!("Group {id}"))
        .field("ID", &Color::Cyan.paint(id.to_string()).to_string())
        .field("Name", group.meta.name().unwrap_or("<unnamed>"))
        .field("Fixtures", &group.members.len().to_string())
        .field(
            "Master",
            &format!("{} at {:.0}%", group.master.label(), level * 100.0),
        )
        .field(
            "PreserveGrid",
            if group.preserve_grid { "On" } else { "Off" },
        )
        .field("MoveGridCursor", group.move_grid_cursor.label())
        .separator()
        .section("Members (order, grid x/y/z)");
    let lines = member_lines(state, group);
    if lines.is_empty() {
        sheet = sheet.line(&format!("  {}", Color::Yellow.paint("(empty)")));
    }
    for line in &lines {
        sheet = sheet.line(line);
    }
    sheet.build()
}

// ── List ─────────────────────────────────────────────────────────────────────

#[derive(Tabled)]
struct GroupRow {
    #[tabled(rename = "No")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Count")]
    count: String,
    #[tabled(rename = "Master")]
    master: String,
    #[tabled(rename = "Fixtures")]
    fixtures: String,
}

/// `List Group` — the groups of the user's datapool.
pub fn list_groups(state: &AppState) -> String {
    let now = lumacore_base::clock::session_ms();
    let rows: Vec<GroupRow> = state
        .user_groups()
        .iter()
        .map(|(no, g)| {
            let ids: Vec<String> = g
                .members
                .iter()
                .map(|m| {
                    state
                        .root
                        .show_data
                        .fixture_id_for_guid(m.fixture)
                        .map_or_else(|| "?".to_string(), |id| id.to_string())
                })
                .collect();
            let fixtures = if ids.is_empty() {
                "—".to_string()
            } else if ids.len() <= 8 {
                ids.join(", ")
            } else {
                format!("{}, … (+{})", ids[..8].join(", "), ids.len() - 8)
            };
            let master = match g.master {
                GroupMasterMode::None => "—".to_string(),
                m => {
                    let level = state
                        .root
                        .show_data
                        .playbacks
                        .live.masters.groups
                        .get(&g.meta.identity.guid)
                        .copied()
                        .unwrap_or_default();
                    format!("{} {:.0}%", m.label(), level.at(now) * 100.0)
                }
            };
            GroupRow {
                id: no.to_string(),
                name: g.meta.name().unwrap_or("—").to_string(),
                count: g.members.len().to_string(),
                master,
                fixtures,
            }
        })
        .collect();
    if rows.is_empty() {
        return nu_ansi_term::Style::new()
            .dimmed()
            .paint("No groups defined. Use 'Store Group <id>' to create one.")
            .to_string();
    }
    format!(
        "{}\n{}",
        nu_ansi_term::Style::new().bold().paint("Groups"),
        Table::new(rows).with(TableStyle::rounded())
    )
}

// ── The Group object ─────────────────────────────────────────────────────────

/// The fixtures `Group <n> [Thru <m>]` names (On / Off).
fn group_fixtures(state: &AppState, args: &[&str]) -> Result<Vec<Uuid>, String> {
    if args.is_empty() {
        return Ok(state
            .current_programmer()
            .map(|p| p.selected_fixtures.clone())
            .unwrap_or_default());
    }
    let ids = parse_ids(state, args)?;
    let mut out = Vec::new();
    for id in ids {
        out.extend(
            state
                .user_groups()
                .get(&id)
                .ok_or_else(|| not_found(id))?
                .fixtures(),
        );
    }
    Ok(out)
}

/// Your DataPool's group numbers a typed target names (the one resolver, §33 57).
fn group_numbers(state: &AppState, target: &lumacore_syntax::ast::Target) -> Result<Vec<u32>, String> {
    let r = crate::resolve::resolve_one(state, target, &GROUP)?;
    if r.members.is_empty() {
        return Err(render_code("None of these groups exist (not found).", "E-GROUP-003"));
    }
    Ok(r.members.iter().map(|m| m.no).collect())
}

/// `Store Group [<n> | "name"] ["name"] [/Merge | /Overwrite | /Remove] [/GridMerge AppendX|Off]`:
/// the slot by the resolver's Store rule (a new number, an existing name, a new name, or the next
/// free one).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Group")]
pub struct StoreGroup {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
    #[lum(value)]
    pub name: Option<String>,
    #[lum(option = "Merge")]
    pub merge: bool,
    #[lum(option = "Overwrite")]
    pub overwrite: bool,
    #[lum(option = "Remove")]
    pub remove: bool,
    #[lum(option = "GridMerge")]
    pub grid: Option<lumacore_syntax::ast::Value>,
}

impl super::typed::Command for StoreGroup {
    fn run(self, state: &mut AppState) -> DispatchResult {
        use crate::resolve::Slot;
        let reply = (|| {
            let append_x = match &self.grid {
                None => true,
                Some(v) => match match v {
                    lumacore_syntax::ast::Value::Word(w) => Some(crate::lum::vocabulary().word(w.node).name.clone()),
                    _ => None,
                }
                .as_deref()
                {
                    Some(w) if w.eq_ignore_ascii_case("AppendX") => true,
                    Some(w) if w.eq_ignore_ascii_case("Off") => false,
                    _ => return Err(render_code("/GridMerge AppendX or Off.", "E-CMD-008")),
                },
            };
            let (slot, datapool) = crate::resolve::slot(state, self.target.as_ref(), &GROUP)?;
            let mode = match (self.merge, self.overwrite, self.remove) {
                (true, _, _) => StoreMode::Merge,
                (_, true, _) => StoreMode::Overwrite,
                (_, _, true) => StoreMode::Remove,
                _ => StoreMode::Ask,
            };
            // An owner in front (`Store DataPool 2 Group 3`) stores there.
            let pool = datapool.or(state.datapool_override);
            let before = std::mem::replace(&mut state.datapool_override, pool);
            let stored = (|| {
                if let Some(n) = &self.name {
                    crate::resolve::check_name(n)?;
                }
                let (id, name) = match slot {
                    Slot::Number(n) => (parse_id(state, &n.to_string()).map_err(|e| render_code(e, "E-CMD-008"))?, self.name),
                    Slot::NewName(n) => (first_free(state), Some(n)),
                    Slot::Next => (first_free(state), self.name),
                    Slot::Key(k) => return Err(render_code(format!("'{k}' isn't a Group number."), "E-CMD-008")),
                };
                Ok(store_group_at(state, id, name, mode, append_x))
            })();
            state.datapool_override = before;
            stored
        })();
        out(reply.unwrap_or_else(|e: String| e))
    }
}

/// `Update Group <target>`: merge the selection into each (AppendX).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Update", object = "Group")]
pub struct UpdateGroup {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
}

impl super::typed::Command for UpdateGroup {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(group_numbers(state, &self.target).map_or_else(
            |e| e,
            |ids| ids.iter().map(|n| store_group(state, &[&n.to_string(), "/Merge"])).collect::<Vec<_>>().join("\n"),
        ))
    }
}

fn group_call(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_group(state, args))
}
fn group_edit(state: &mut AppState, args: &[&str]) -> DispatchResult {
    super::edit::edit_with(state, args, edit_group)
}
fn group_list(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    out(list_groups(state))
}
fn group_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(super::info::info_group(state, args.first().copied()))
}
fn group_dump(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(super::show::dump_group(state, args))
}
fn group_on(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(match group_fixtures(state, args) {
        Ok(f) => super::control::knock_in_fixtures(state, &f),
        Err(e) => error_reply(e, "E-CMD-006"),
    })
}
fn group_off(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(match group_fixtures(state, args) {
        Ok(f) => super::control::knock_out_fixtures(state, &f),
        Err(e) => error_reply(e, "E-CMD-006"),
    })
}

fn groups_in(root: &lumacore_tree::Root, dp: u32) -> Vec<crate::vfs::DynChild> {
    root.show_data
        .datapools
        .get(&dp)
        .map(|d| crate::vfs::btree_to_children(&d.groups))
        .unwrap_or_default()
}
fn group_in(root: &lumacore_tree::Root, dp: u32, id: u32) -> Option<crate::vfs::NodeView> {
    root.show_data
        .datapools
        .get(&dp)?
        .groups
        .get(&id)
        .map(|t| crate::vfs::object_view(id, t))
}
fn groups_children(root: &lumacore_tree::Root) -> Vec<crate::vfs::DynChild> {
    crate::vfs::first_datapool_groups(root)
}
fn group_view(root: &lumacore_tree::Root, id: u32) -> Option<crate::vfs::NodeView> {
    root.show_data
        .datapools
        .values()
        .next()?
        .groups
        .get(&id)
        .map(|t| crate::vfs::object_view(id, t))
}

/// The Groups pool window: tap selects (by the selection rule); cells show
/// how much of each group is selected and its master (decisions §22).
static GROUP_POOL: PoolDef = PoolDef {
    window: "groups",
    actions: &[
        PoolAction {
            name: "Select",
            command: "{obj}",
            release: "",
        },
        PoolAction {
            name: "Edit",
            command: "Edit {obj}",
            release: "",
        },
        PoolAction {
            name: "None",
            command: "",
            release: "",
        },
    ],
    cells: group_cells,
    detail: Some(group_detail),
    head: None,
};

/// The group editor's content: members in order with fixture numbers, names
/// and grid spots (a deleted fixture shows as missing).
fn group_detail(ctx: &PoolContext, no: u32) -> Option<serde_json::Value> {
    let g = ctx.show.datapools.get(&ctx.datapool)?.groups.get(&no)?;
    let members: Vec<serde_json::Value> = g
        .members
        .iter()
        .map(|m| {
            let fid = ctx.show.fixture_id_for_guid(m.fixture);
            let name = fid
                .and_then(|id| ctx.show.devices.dmx.patch.fixture(id))
                .and_then(|f| f.meta.identity.name.clone());
            serde_json::json!({
                "fixture": fid,
                "name": name,
                "missing": fid.is_none(),
                "x": m.cell.x, "y": m.cell.y, "z": m.cell.z,
                "rotation": m.cell.rotation,
                "mirror_x": m.cell.mirror_x, "mirror_y": m.cell.mirror_y,
            })
        })
        .collect();
    Some(serde_json::json!({
        "no": no,
        "name": g.meta.identity.name,
        "mode": g.master.label(),
        "preserve_grid": g.preserve_grid,
        "move_grid_cursor": g.move_grid_cursor.label(),
        "owner": ctx.claims.get(&g.meta.identity.guid),
        "members": members,
    }))
}

fn group_cells(ctx: &PoolContext) -> Vec<PoolCell> {
    let Some(dp) = ctx.show.datapools.get(&ctx.datapool) else {
        return Vec::new();
    };
    let selected: std::collections::HashSet<Uuid> = ctx
        .show
        .programmer_of(ctx.user)
        .map(|p| p.selected_fixtures.iter().copied().collect())
        .unwrap_or_default();
    dp.groups
        .iter()
        .map(|(no, g)| {
            let mut cell = PoolCell::of(*no, &g.meta, ctx);
            let picked = g
                .members
                .iter()
                .filter(|m| selected.contains(&m.fixture))
                .count();
            cell.applicability = Some(match picked {
                0 => "none",
                n if n == g.members.len() => "all",
                _ => "some",
            });
            if g.master != GroupMasterMode::None {
                let level = ctx
                    .show
                    .playbacks
                    .live.masters.groups
                    .get(&g.meta.identity.guid)
                    .copied()
                    .unwrap_or_default()
                    .at(ctx.now_ms);
                cell.bar = Some(CellBar {
                    kind: "master",
                    value: level,
                    text: format!("{:.0}%", level * 100.0),
                });
                cell.badges.push(g.master.label().to_string());
            }
            cell
        })
        .collect()
}

// ── A group's properties (decisions §33 58, 88, 90) ─────────────────────────

static GROUP_OWN: [crate::props::Property; 3] = [
    crate::props::Property {
        name: "Mode",
        access: crate::props::Access::Field("master"),
        kind: crate::props::PropKind::Choice(GroupMasterMode::NAMES),
        doc: "how the group's master acts on its members' intensity (GroupMasterMode)",
        default: Some("None"),
        ..crate::props::Property::BASE
    },
    crate::props::Property {
        name: "PreserveGrid",
        access: crate::props::Access::Field("preserve_grid"),
        kind: crate::props::PropKind::OnOff,
        doc: "recall keeps the stored grid gaps and offset (PreserveGridPositions)",
        default: Some("On"),
        ..crate::props::Property::BASE
    },
    crate::props::Property {
        name: "MoveGridCursor",
        access: crate::props::Access::Field("move_grid_cursor"),
        kind: crate::props::PropKind::Choice(GridCursorMove::NAMES),
        doc: "where your grid cursor goes after the group is selected",
        default: Some("Linked"),
        ..crate::props::Property::BASE
    },
];

thread_local! {
    /// The group a member line sets (`Set Group 3 Fixture 101 "X" 4`) while its guard lives.
    static MEMBER_GROUP: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

struct MemberGroup(Option<u32>);

impl Drop for MemberGroup {
    fn drop(&mut self) {
        MEMBER_GROUP.with(|g| g.set(self.0));
    }
}

/// A group member's grid spot and place: one property of the member `fid` of the scoping group.
fn member_write(state: &mut AppState, fid: &str, prop: &str, value: &str) -> Result<String, String> {
    let Some(id) = MEMBER_GROUP.with(std::cell::Cell::get) else {
        return Err(render_code(format!("\"{prop}\" is a group member's: Set Group <n> Fixture {fid} \"{prop}\" …"), "E-SET-001"));
    };
    crate::props::from_reply(set_member(state, id, &[fid, prop, value]))
}

macro_rules! member_prop {
    ($name:literal, $kind:expr, $doc:literal) => {
        crate::props::Property {
            name: $name,
            kind: $kind,
            section: "Group member",
            doc: $doc,
            access: crate::props::Access::Named {
                get: |_, _, _| String::new(),
                set: |s, k, p, v| member_write(s, k, p, v),
            },
            ..crate::props::Property::BASE
        }
    };
}

/// A fixture's properties as a group's member (`Set Group 3 Fixture 101 "X" 4`).
pub static MEMBER_PROPS: [crate::props::Property; 7] = [
    member_prop!("X", crate::props::PropKind::Int { min: -9999, max: 9999 }, "its grid column"),
    member_prop!("Y", crate::props::PropKind::Int { min: -9999, max: 9999 }, "its grid row"),
    member_prop!("Z", crate::props::PropKind::Int { min: -9999, max: 9999 }, "its grid layer"),
    member_prop!("Rotation", crate::props::PropKind::Declared, "degrees"),
    member_prop!("MirrorX", crate::props::PropKind::OnOff, "mirrored left–right"),
    member_prop!("MirrorY", crate::props::PropKind::OnOff, "mirrored top–bottom"),
    member_prop!("Order", crate::props::PropKind::Int { min: 1, max: 99999 }, "its place in the group's order (1 = first)"),
];

static GROUP_SETTINGS: crate::props::Props = crate::props::Props {
    meta: crate::props::MetaLevel::Node,
    lists: &[&GROUP_OWN],
    apply: Some(group_write),
    within: Some(|_, key| {
        let no = key.parse::<u32>().map_err(|_| render_code(format!("No Group {key}."), "E-CMD-007"))?;
        Ok(Box::new(MemberGroup(MEMBER_GROUP.with(|g| g.replace(Some(no))))))
    }),
    hint: Some("A group master's level is set with At: Master Group <n> At 50 (decisions §33 90)."),
    ..crate::props::Props::BASE
};

// The object door (decisions §35): Copy / Cut / Paste / Move / Insert / Compact / Delete.
crate::door_verbs!(group_door, "Group", super::GROUP);

fn door_map(state: &mut AppState, dp: Option<u32>) -> Option<&mut std::collections::BTreeMap<u32, Group>> {
    let dp = dp?;
    ensure_datapool(state, dp);
    state.root.show_data.datapools.get_mut(&dp).map(|d| &mut d.groups)
}

fn door_map_ref(state: &AppState, dp: Option<u32>) -> Option<&std::collections::BTreeMap<u32, Group>> {
    state.root.show_data.datapools.get(&dp?).map(|d| &d.groups)
}

/// Groups on the door: a group's master level goes with it; a merge adds members (AppendX).
pub static GROUP_DOOR: crate::pools::door::BTreePool<Group> = crate::pools::door::BTreePool {
    object: "Group",
    in_datapool: true,

    max: crate::pools::MAX_NO,
    map: door_map,
    map_ref: door_map_ref,
    name: |g| g.meta.identity.name.clone(),
    identify: |g, guid| g.meta.identity.guid = guid,
    rules: crate::pools::door::PoolRules {
        can_delete: |_, _| Ok(()),
        deleted: |state, g| {
            state.root.show_data.playbacks.live.masters.groups.remove(&g.meta.identity.guid);
        },
        changed: |_| {},
        merge: Some(|into, from| format!("{} fixtures added.", into.merge_members(&from.members, true))),
        orders: &[],
    },
};

fn group_delete_ask(state: &AppState, args: &[&str]) -> crate::objects::DeleteAsk {
    super::door::delete_ask(state, &GROUP, args)
}

pub static GROUP: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Group",
        // A member is the group's Fixture child (`Set Group 3 Fixture 101 "X" 4`).
        lum: crate::objects::LumShape::ids(1).children(&[&crate::objects::FIXTURE.decl]),
        default_verb: crate::objects::Verb::SelectFixtures,
        flags: &[F::MERGE.on(&[Verb::Store, Verb::Copy, Verb::Paste]), F::OVERWRITE.on(&[Verb::Store, Verb::Copy, Verb::Paste]), F::REMOVE.on(&[Verb::Store]), GRID_MERGE.on(&[Verb::Store]), F::BY.on(&[Verb::Compact])],
        aliases: &["groups", "g"],
        doc: crate::commands::registry::Doc {
            display: "Group",
            category: crate::commands::help::HelpCategory::Group,
            also_applies_to: &[],
            syntax: "Group <n> [+ | - <selection>] [At <value>] | Group <n> Master At <v> [Fade <s>]",
            description: "A stored selection in your datapool: members in order with grid spots \
                          (decisions §22). Calling it selects: a plain select adds to the selection \
                          until a value is set, then the next one replaces it (a common console convention); `+` \
                          always adds. A group can be a master (Set Group 3 \"Mode\" Positive / \
                          Negative / Scaling / Additive) whose level is `Master Group 3 At 50`. \
                          Store over an existing group needs /Merge, /Overwrite or /Remove.",
            examples: &[
                "Group 1                      -- select group 1",
                "Group 1 + Group 2 - Fixture 5",
                "Group 3 At 50                -- select, then dimmer 50",
                "Master Group 3 At 50 Fade 2  -- the group master",
                "Store Group 4 /Merge",
                "Set Group 4 \"Mode\" Scaling",
            ],
            related: &["store", "select", "plus", "minus", "grid"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: Some(crate::commands::registry::CommandFamily::Selection),
        call_undo: crate::commands::registry::SELECTION,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::content,
        planned: &[],
    },
    pool: Some(&GROUP_POOL),
    verbs: &[
        (Verb::SelectFixtures, group_call),
        (Verb::At, group_call),
        (Verb::Store, super::typed::typed::<StoreGroup>),
        (Verb::Delete, super::typed::typed::<group_door::Delete>),
        (Verb::Edit, group_edit),
        (Verb::Copy, super::typed::typed::<group_door::Copy>),
        (Verb::Cut, super::typed::typed::<group_door::Cut>),
        (Verb::Paste, super::typed::typed::<group_door::Paste>),
        (Verb::Update, super::typed::typed::<UpdateGroup>),
        (Verb::Move, super::typed::typed::<group_door::Move>),
        (Verb::Insert, super::typed::typed::<group_door::Insert>),
        (Verb::Compact, super::typed::typed::<group_door::Compact>),
        (Verb::List, group_list),
        (Verb::Info, group_info),
        (Verb::Dump, group_dump),
        (Verb::On, group_on),
        (Verb::Off, group_off),
    ],
    delete_ask: Some(group_delete_ask),
    complete: Some(collect_group_completions),
    props: Some(&GROUP_SETTINGS),
    tree: Some(crate::objects::TreeDef {
        door: Some(&GROUP_DOOR),
        datapool: Some(crate::objects::DatapoolTree {
            children: groups_in,
            object: group_in,
        }),
        name: None,
        node: crate::tree_path::Node::Groups,
        children: groups_children,
        object: group_view,
        // Name, Note, Locked, Hidden, Color, Icon (NodeMeta; §30 "12e").
    }),
};
fn group_write(
    root: &mut lumacore_tree::Root,
    dp: Option<u32>,
    id: u32,
    edit: &mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>,
) -> Result<(), String> {
    let group = root
        .show_data
        .datapools
        .get_mut(&dp.ok_or("no datapool")?)
        .and_then(|d| d.groups.get_mut(&id))
        .ok_or("no such group")?;
    crate::props::edit_json(group, edit)
}

/// Collect group IDs and names for completion after `Group` or `Store Group`.
fn collect_group_completions(state: &AppState, word_lower: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let dp_id = state.selected_datapool();
    if let Some(dp) = state.root.show_data.datapools.get(&dp_id) {
        for (&id, group) in &dp.groups {
            let id_str = id.to_string();
            if id_str.starts_with(word_lower) {
                candidates.push(id_str);
            }
            if let Some(name) = group.name()
                && name.to_lowercase().starts_with(word_lower)
            {
                candidates.push(format!("\"{name}\""));
            }
        }
    }
    candidates
}
