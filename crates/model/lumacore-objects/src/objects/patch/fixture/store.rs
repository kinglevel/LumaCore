// ███████╗████████╗ ██████╗ ██████╗ ███████╗
// ██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
// ███████╗   ██║   ██║   ██║██████╔╝█████╗
// ╚════██║   ██║   ██║   ██║██╔══██╗██╔══╝
// ███████║   ██║   ╚██████╔╝██║  ██║███████╗
// ╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: store

//! `Store Fixture <numbers> [At DMX …] /FixtureType <slot | "name" | "file.gdtf"> [/Mode …]
//! [/Revision n] [/Every n] [/KeepSpacing]` (decisions §34 round 3 1, 19): creates fixtures at the
//! numbers typed (every number of a range — they don't exist yet), from a type of the show (by
//! slot or name) or a `.gdtf` of `library/gdtf/` imported first in the same step; the newest
//! revision unless `/Revision`, the first mode unless `/Mode`. With `At DMX …` they get their
//! addresses by the one placement rule (`address`), without it none. A number that exists is
//! refused (a type change is `Set Fixture … "FixtureType"`). Needs the Devices claim.
//!
//! Owns:          Store Fixture, reading `/FixtureType`, `/Mode`, `/Revision`.
//! Doesn't own:   where they land (address), importing a file (commands::import).
//! Entry points:  StoreFixture, FIXTURE_TYPE, MODE, REVISION, EVERY, KEEP_SPACING.
//! Decisions:     §34 round 3 1, 5, 19.

use std::collections::BTreeMap;

use lumacore_syntax::ast::{Id, Item, Range, SetOp, Target, Value};

use super::address::{self, Item as Place, Spacing};
use crate::commands::flags::FlagDef;
use crate::errors::render_code;
use crate::objects::patch::dmx;
use crate::objects::{Verb, out, patch_claimed};
use crate::text::render_success;
use crate::{AppState, DispatchResult};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

pub const FIXTURE_TYPE: FlagDef =
    FlagDef::new("FixtureType", "The fixture type: the show's slot, its name, or a .gdtf file of library/gdtf").value("type").on(&[Verb::Store]);
pub const MODE: FlagDef = FlagDef::new("Mode", "The fixture type's DMX mode (name or number)").value("mode").on(&[Verb::Store]);
pub const REVISION: FlagDef = FlagDef::new("Revision", "The fixture type's revision (1 = the first)").value("revision").on(&[Verb::Store]);
pub const EVERY: FlagDef = FlagDef::new("Every", "Each fixture starts n slots after the one before").value("slots").on(&[Verb::Store, Verb::Assign]);
pub const KEEP_SPACING: FlagDef = FlagDef::new("KeepSpacing", "Keep the gaps the fixtures already have").on(&[Verb::Assign]);
pub const SWAP: FlagDef = FlagDef::new("Swap", "Trade addresses with the fixtures named (At Fixture … DMX)").on(&[Verb::Assign]);

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Store Fixture <numbers> [At DMX …] /FixtureType … [/Mode …] [/Revision n] [/Every n]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Fixture")]
pub struct StoreFixture {
    #[lum(target)]
    pub target: Target,
    #[lum(at = "destination")]
    pub to: Option<Value>,
    #[lum(option = "FixtureType")]
    pub fixture_type: Option<Value>,
    #[lum(option = "Mode")]
    pub mode: Option<Value>,
    #[lum(option = "Revision")]
    pub revision: Option<Value>,
    #[lum(option = "Every")]
    pub every: Option<Value>,
    /// `/Drive "usb1"`: a `.gdtf` file on that drive's library (decisions §38).
    #[lum(option = "Drive")]
    pub drive: Option<Value>,
}

impl crate::commands::typed::Command for StoreFixture {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        out(store(state, self).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn store(state: &mut AppState, cmd: StoreFixture) -> Result<String, String> {
    if let Some((fixtures, copies)) = super::multipatch::split(&cmd.target) {
        let dest = cmd.to.as_ref().map(|v| dmx::dest_of(state, v).unwrap_or_else(|| Err(render_code("… Multipatch … At DMX <address>.", "E-CMD-008")))).transpose()?;
        let every = cmd.every.as_ref().map(|v| number(v, "/Every")).transpose()?;
        return super::multipatch::store(state, &fixtures, copies.as_ref(), dest, every);
    }
    let numbers = new_numbers(&cmd.target)?;
    let patch = &state.root.show_data.devices.dmx.patch;
    let taken: Vec<u32> = numbers.iter().copied().filter(|n| patch.has_fid(n)).collect();
    if !taken.is_empty() {
        return Err(render_code(
            format!("Fixture {} exists: a new fixture takes a free number (a type change is Set Fixture … \"FixtureType\").", address::numbers(&taken)),
            "E-CMD-010",
        ));
    }
    let dest = match &cmd.to {
        None => None,
        Some(v) => Some(dmx::dest_of(state, v).unwrap_or_else(|| Err(render_code("Store Fixture … At DMX <address> | DMX <universe> | DMX.", "E-CMD-008")))?),
    };
    let every = cmd.every.as_ref().map(|v| number(v, "/Every")).transpose()?;
    let drive = cmd.drive.as_ref().map(text);
    let (imported, type_no, revision) = fixture_type(state, cmd.fixture_type.as_ref(), cmd.revision.as_ref(), drive.as_deref())?;
    let ft = state.root.show_data.devices.dmx.patch.fixture_types[&type_no].clone();
    let rev = &ft.revisions[revision];
    let mode = match &cmd.mode {
        None => 0,
        Some(v) => {
            let word = text(v);
            let by_name = rev.modes.iter().position(|m| m.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(&word)));
            let by_no = word.parse::<usize>().ok().filter(|n| (1..=rev.modes.len()).contains(n)).map(|n| n - 1);
            by_name.or(by_no).ok_or_else(|| {
                let names: Vec<String> = rev.modes.iter().enumerate().map(|(i, m)| format!("{} \"{}\"", i + 1, m.meta.name().unwrap_or(""))).collect();
                render_code(format!("FixtureType {type_no} has no mode {word}. Modes: {}", names.join(", ")), "E-CMD-008")
            })?
        }
    };
    let m = rev.modes.get(mode).ok_or_else(|| render_code(format!("FixtureType {type_no} revision {} has no modes.", revision + 1), "E-CMD-010"))?;
    let footprint = m.dmx_footprint;
    // Where they go, before anything changes: a refused placement leaves nothing behind.
    let plan = match dest {
        Some(d) => {
            let items: Vec<Place> = numbers.iter().map(|no| Place { no: *no, footprint, now: None }).collect();
            Some(address::plan(&state.root.show_data, &items, d, Spacing { every, keep: false })?)
        }
        None => None,
    };
    let patch = &mut state.root.show_data.devices.dmx.patch;
    for no in &numbers {
        let mut f = lumacore_fixtures::Fixture::new(None, ft.meta.identity.guid, rev.meta.identity.guid, m.meta.identity.guid, uuid::Uuid::nil(), 1, footprint);
        f.mvr.breaks.clear();
        patch.insert_fid(no, f);
        patch.fixture_id_pool.reserve(*no);
    }
    let name = ft.meta.name().unwrap_or("").to_string();
    let mut lines = Vec::new();
    if let Some(reply) = imported {
        lines.push(reply);
    }
    let what = format!("Fixture {} \"{name}\" (mode \"{}\")", address::numbers(&numbers), m.meta.name().unwrap_or(""));
    match plan {
        None => lines.push(render_success(&format!("Stored {what}, no address"))),
        Some(plan) => {
            let footprints: BTreeMap<u32, u16> = numbers.iter().map(|n| (*n, footprint)).collect();
            address::apply(&mut state.root.show_data, &plan, &footprints);
            lines.push(render_success(&format!("Stored {what} at {}", address::placed_text(&plan))));
            lines.extend(address::plan_notes(&plan));
        }
    }
    Ok(lines.join("\n"))
}

/// The type `/FixtureType` names and its revision: (an import's reply, slot, revision index).
fn fixture_type(state: &mut AppState, value: Option<&Value>, revision: Option<&Value>, drive: Option<&str>) -> Result<(Option<String>, u32, usize), String> {
    let value = value.ok_or_else(|| render_code("Store Fixture needs /FixtureType <slot | \"name\" | \"file.gdtf\">.", "E-CMD-006"))?;
    let types = || &state.root.show_data.devices.dmx.patch.fixture_types;
    let pick = |state: &AppState, no: u32| -> Result<usize, String> {
        let n = state.root.show_data.devices.dmx.patch.fixture_types[&no].revisions.len();
        match revision {
            None => Ok(n.saturating_sub(1)),
            Some(v) => text(v).parse::<usize>().ok().filter(|r| (1..=n).contains(r)).map(|r| r - 1).ok_or_else(|| render_code(format!("FixtureType {no} has revisions 1–{n}."), "E-CMD-008")),
        }
    };
    match value {
        Value::Number(n) => {
            let no = n.node.parse::<u32>().ok().filter(|no| types().contains_key(no)).ok_or_else(|| render_code(format!("No FixtureType {}.", n.node), "E-PATCH-005"))?;
            Ok((None, no, pick(state, no)?))
        }
        Value::Text(t) if t.node.to_lowercase().ends_with(".gdtf") => {
            // A file: the type that has it (that revision), else imported first (decisions round 3 5).
            let folder = state.library_folder(drive, "gdtf", false)?;
            let path = lumacore_luma::drive::library_file(&folder, &t.node).map_err(|e| render_code(e, "E-CMD-008"))?;
            let bytes = lumacore_luma::drive::read(&path).map_err(|e| render_code(format!("{}: {e}", t.node), "E-CMD-012"))?;
            let hash = lumacore_gdtf::file_hash(&bytes);
            let find = |state: &AppState| {
                state.root.show_data.devices.dmx.patch.fixture_types.iter().find_map(|(no, ft)| ft.revisions.iter().position(|r| r.file_hash == hash).map(|r| (*no, r)))
            };
            if let Some((no, r)) = find(state) {
                return Ok((None, no, r));
            }
            let reply = crate::commands::import::import_gdtf_file(state, &path);
            let (no, r) = find(state).ok_or_else(|| reply.clone())?;
            Ok((Some(reply), no, r))
        }
        Value::Text(t) => {
            let named: Vec<u32> = types().iter().filter(|(_, ft)| ft.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(&t.node))).map(|(no, _)| *no).collect();
            match named.as_slice() {
                [no] => Ok((None, *no, pick(state, *no)?)),
                [] => Err(render_code(format!("No fixture type is called \"{}\" (a .gdtf file imports).", t.node), "E-PATCH-005")),
                many => Err(render_code(format!("FixtureType {} share the name \"{}\": give its number.", address::numbers(many), t.node), "E-PATCH-010")),
            }
        }
        _ => Err(render_code("/FixtureType takes the show's slot, a quoted name or a quoted .gdtf file.", "E-CMD-008")),
    }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The numbers a Store target types, every one of a range (new objects don't exist yet).
fn new_numbers(target: &Target) -> Result<Vec<u32>, String> {
    let bad = || render_code("Store Fixture takes numbers: Store Fixture 201 Thru 206.", "E-CMD-008");
    let whole = |id: &Option<Id>| match id {
        Some(Id::Number(n)) => n.node.parse::<u32>().ok().filter(|n| *n > 0).ok_or_else(bad),
        _ => Err(bad()),
    };
    let mut out: Vec<u32> = Vec::new();
    for (_, item) in &target.items {
        let ids = match item {
            Item::Path(segments) => segments.last().and_then(|s| s.ids.as_ref()),
            Item::Ids(ids) => Some(ids),
            _ => None,
        }
        .ok_or_else(bad)?;
        for (op, range) in &ids.ranges {
            let these: Vec<u32> = match range {
                Range::One(id) => vec![whole(&Some(id.clone()))?],
                Range::Thru { from, to, .. } => (whole(from)?..=whole(to)?).collect(),
            };
            match op.as_ref().map(|o| o.node) {
                Some(SetOp::Minus) => out.retain(|n| !these.contains(n)),
                _ => {
                    for n in these {
                        if !out.contains(&n) {
                            out.push(n);
                        }
                    }
                }
            }
        }
    }
    if out.is_empty() { Err(bad()) } else { Ok(out) }
}

fn text(v: &Value) -> String {
    match v {
        Value::Number(n) => n.node.clone(),
        Value::Text(t) => t.node.clone(),
        other => format!("{other:?}"),
    }
}

fn number(v: &Value, what: &str) -> Result<u16, String> {
    text(v).parse::<u16>().ok().filter(|n| (1..=512).contains(n)).ok_or_else(|| render_code(format!("{what} takes slots, 1–512."), "E-CMD-008"))
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
