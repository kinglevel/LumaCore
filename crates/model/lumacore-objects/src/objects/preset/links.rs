// ██╗     ██╗███╗   ██╗██╗  ██╗███████╗
// ██║     ██║████╗  ██║██║ ██╔╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╔╝ ███████╗
// ██║     ██║██║╚██╗██║██╔═██╗ ╚════██║
// ███████╗██║██║ ╚████║██║  ██╗███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝
// FILE: links

//! What Update, Recast, Extract and Delete do to presets and the cues that link them (decisions
//! 0293). A cue value that came from a preset holds only the link; these are the writes around
//! it, shared by the words (`update`, `recast`, `extract`, `delete`) and the Update ask:
//!
//! - **Update** puts the programmer's changes back into the preset its values came from
//!   (`ProgValue::origin`): `/Merge` only what the preset holds, `/Add` also new attributes of its
//!   fixtures, `/Overwrite` the preset becomes them. The programmer's values link it again.
//! - **Recast** adds links for the preset's attributes a linking cue lacks and drops links the
//!   preset no longer answers.
//! - **Extract** turns links into the preset's values as they are now (a link the preset gives
//!   nothing becomes a release: it played nothing before either).
//!
//! Owns:          Content, CueScope, update_preset, recast_plan, recast, extract_cues,
//!                extract_programmer, cue_scope, preset_line.
//! Doesn't own:   the link itself (lumacore-playback `CueData`), the lookups (lumacore-tree
//!                `preset_links`), the words' rows (`update`, `recast`, `extract`).
//! Entry points:  update_preset, recast_plan, recast, extract_cues, extract_programmer, cue_scope.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

use std::collections::BTreeSet;

use lumacore_base::Value;
use lumacore_tree::{CueData, CueValue, ShowData};
use uuid::Uuid;

use crate::AppState;
use crate::errors::render_code;
use crate::objects::playback::{CUE, SEQUENCE};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What an update writes into a preset (`/Merge`, `/Add`, `/Overwrite`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Content {
    /// Only what the preset already holds (the ask's `OnlyExisting`).
    #[default]
    Merge,
    /// Also the attributes it doesn't hold yet, on the fixtures its values are on.
    Add,
    /// The preset becomes exactly those values.
    Overwrite,
}

impl Content {
    pub const ALL: [Content; 3] = [Content::Merge, Content::Add, Content::Overwrite];

    /// The flag's word (`/Merge`, `/Add`, `/Overwrite`).
    pub fn word(self) -> &'static str {
        match self {
            Content::Merge => "Merge",
            Content::Add => "Add",
            Content::Overwrite => "Overwrite",
        }
    }

    /// The ask's choice and the profile's `UpdateContent` value: its own word, never a flag's
    /// or a verb's (one name, one meaning).
    pub fn choice(self) -> &'static str {
        match self {
            Content::Merge => "OnlyExisting",
            Content::Add => "AddNew",
            Content::Overwrite => "Overwrite",
        }
    }

    /// A choice word back (the profile's value).
    pub fn parse(choice: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.choice().eq_ignore_ascii_case(choice))
    }
}

/// Which cues a Recast or an Extract reaches: every sequence, one sequence or one cue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CueScope {
    All,
    Sequence { dp: u32, no: u32 },
    Cue { dp: u32, no: u32, cue: u32 },
}

impl CueScope {
    fn holds(&self, dp: u32, no: u32, cue: u32) -> bool {
        match *self {
            CueScope::All => true,
            CueScope::Sequence { dp: d, no: n } => d == dp && n == no,
            CueScope::Cue { dp: d, no: n, cue: c } => d == dp && n == no && c == cue,
        }
    }
}

/// What a Recast changes in one cue: the attributes it gains and loses links for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecastCue {
    pub dp: u32,
    pub sequence: u32,
    pub cue: u32,
    pub name: String,
    pub gains: Vec<String>,
    pub loses: Vec<String>,
    /// (fixture, attribute) links to add, and to drop.
    add: Vec<(Uuid, String)>,
    drop: Vec<(Uuid, String)>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Update Preset <n>` of DataPool `dp`: the acting user's active values whose home is the preset
/// go back into it (`content`), then link it again. Err when the programmer holds nothing from it.
pub fn update_preset(state: &mut AppState, dp: u32, no: u32, content: Content) -> Result<String, String> {
    let preset = state
        .root
        .show_data
        .datapools
        .get(&dp)
        .and_then(|d| d.presets.get(&no))
        .ok_or_else(|| render_code(format!("No Preset {no} in DataPool {dp}."), "E-CMD-007"))?;
    let guid = preset.meta.identity.guid;
    let Some(prog) = state.current_programmer() else {
        return Err(render_code("No programmer available.", "E-CMD-011"));
    };
    // The values whose home it is, and (Add, Overwrite) the plain values on the same fixtures.
    let homed: Vec<(Uuid, String, Value)> = prog
        .values
        .iter()
        .filter(|(_, pv)| pv.active && !pv.value.release && pv.origin == Some(guid))
        .map(|((f, a), pv)| (*f, a.clone(), plain(&pv.value)))
        .collect();
    if homed.is_empty() {
        return Err(render_code(format!("Update Preset {no}: your programmer holds no values from it."), "E-CMD-009"));
    }
    let fixtures: BTreeSet<Uuid> = homed.iter().map(|(f, ..)| *f).collect();
    let extra: Vec<(Uuid, String, Value)> = if content == Content::Merge {
        Vec::new()
    } else {
        prog.values
            .iter()
            .filter(|((f, _), pv)| pv.active && !pv.value.release && pv.origin.is_none() && pv.value.preset.is_none() && fixtures.contains(f))
            .map(|((f, a), pv)| (*f, a.clone(), plain(&pv.value)))
            .collect()
    };
    let held: BTreeSet<String> = preset.values.iter().map(|e| e.attribute.to_ascii_lowercase()).collect();
    let numbers: std::collections::HashMap<Uuid, u32> =
        fixtures.iter().chain(preset.fixtures().collect::<Vec<_>>().iter()).filter_map(|g| Some((*g, state.root.show_data.fixture_id_for_guid(*g)?))).collect();

    let mut written: Vec<(Uuid, String)> = Vec::new();
    let mut all: Vec<(Option<Uuid>, String, Value)> = match content {
        Content::Overwrite => Vec::new(),
        _ => preset.values.iter().map(|e| (e.fixture_id, e.attribute.clone(), e.value.clone())).collect(),
    };
    let incoming = homed.into_iter().chain(extra);
    for (f, a, v) in incoming {
        let new_attr = !held.contains(&a.to_ascii_lowercase());
        if content == Content::Merge && new_attr {
            continue;
        }
        // The fixture's own entry is replaced; else the value joins as the fixture's own (the
        // universal entry stays for the others).
        match all.iter_mut().find(|(ef, ea, _)| *ef == Some(f) && ea.eq_ignore_ascii_case(&a)) {
            Some(e) => e.2 = v,
            None => {
                let universal_equal = all.iter().any(|(ef, ea, ev)| ef.is_none() && ea.eq_ignore_ascii_case(&a) && *ev == v);
                if !universal_equal {
                    all.push((Some(f), a.clone(), v));
                }
            }
        }
        written.push((f, a));
    }
    all.sort_by_key(|(f, ..)| f.and_then(|g| numbers.get(&g).copied()).unwrap_or(u32::MAX));
    let (mode, entries) = super::store::mode_of(all);
    let before = preset.values.len();
    let preset = state.root.show_data.datapools.get_mut(&dp).and_then(|d| d.presets.get_mut(&no)).expect("looked up above");
    preset.mode = mode;
    preset.values = entries;
    let after = preset.values.len();
    let shown = preset.meta.name().map(|n| format!(" \"{n}\"")).unwrap_or_default();
    // The programmer's values are the preset's again: they link it.
    if let Some(prog) = state.current_programmer_mut() {
        for (f, a) in &written {
            if let Some(pv) = prog.values.get_mut(&(*f, a.clone())) {
                pv.value.preset = Some(guid);
                pv.origin = Some(guid);
            }
        }
    }
    let follow = lumacore_tree::PresetUses::build(&state.root.show_data).count(guid);
    let follows = match follow {
        0 => String::new(),
        1 => "; 1 cue follows".to_string(),
        n => format!("; {n} cues follow"),
    };
    Ok(crate::text::render_success(&format!(
        "Updated Preset {no}{shown} /{}: {} value{} ({before} → {after} entries){follows}.",
        content.word(),
        written.len(),
        if written.len() == 1 { "" } else { "s" }
    )))
}

/// What a Recast of the preset `guid` would change in the cues of `scope`.
pub fn recast_plan(show: &ShowData, guid: Uuid, scope: CueScope) -> Vec<RecastCue> {
    let Some((_, _, preset)) = show.preset_by_guid(guid) else { return Vec::new() };
    let attrs: Vec<&str> = preset.attributes();
    let mut out = Vec::new();
    for (dp, no, seq) in show.sequences() {
        for (&key, cue) in std::iter::once((&lumacore_tree::CUE_ZERO, &seq.cue_zero)).chain(seq.cues.iter()) {
            if !scope.holds(dp, no, key) {
                continue;
            }
            let linked: Vec<&CueValue> = cue.values().filter(|v| v.value.link() == Some(guid)).collect();
            if linked.is_empty() {
                continue;
            }
            let fixtures: BTreeSet<Uuid> = linked.iter().map(|v| v.fixture_id).collect();
            let mut add = Vec::new();
            for f in &fixtures {
                for a in &attrs {
                    let has = cue.values().any(|v| v.fixture_id == *f && v.attribute.eq_ignore_ascii_case(a));
                    if !has && preset.value_for(*f, a).is_some() && fixture_has(show, *f, a) {
                        add.push((*f, (*a).to_string()));
                    }
                }
            }
            let drop: Vec<(Uuid, String)> =
                linked.iter().filter(|v| preset.value_for(v.fixture_id, &v.attribute).is_none()).map(|v| (v.fixture_id, v.attribute.clone())).collect();
            if add.is_empty() && drop.is_empty() {
                continue;
            }
            let names = |list: &[(Uuid, String)]| {
                let mut seen: Vec<String> = Vec::new();
                for (_, a) in list {
                    if !seen.contains(a) {
                        seen.push(a.clone());
                    }
                }
                seen
            };
            out.push(RecastCue {
                dp,
                sequence: no,
                cue: key,
                name: cue.meta.name().unwrap_or_default().to_string(),
                gains: names(&add),
                loses: names(&drop),
                add,
                drop,
            });
        }
    }
    out
}

/// Apply a Recast plan: links added beside the cue's other links to the preset, dropped links
/// removed. Returns how many cues changed.
pub fn recast(show: &mut ShowData, guid: Uuid, plan: &[RecastCue]) -> usize {
    for r in plan {
        let Some(seq) = show.sequence_mut(r.dp, r.sequence) else { continue };
        let Some(cue) = (if r.cue == lumacore_tree::CUE_ZERO { Some(&mut seq.cue_zero) } else { seq.cues.get_mut(&r.cue) }) else { continue };
        for part in cue.parts.values_mut() {
            part.values.retain(|v| !(v.value.link() == Some(guid) && r.drop.iter().any(|(f, a)| *f == v.fixture_id && *a == v.attribute)));
        }
        let home = cue.parts.iter().find(|(_, p)| p.values.iter().any(|v| v.value.link() == Some(guid))).map_or(0, |(k, _)| *k);
        let part = cue.parts.entry(home).or_default();
        for (f, a) in &r.add {
            part.values.push(CueValue { fixture_id: *f, attribute: a.clone(), value: CueData::Link(guid), timing: Default::default() });
        }
    }
    plan.len()
}

/// Extract in cues: every link (to `preset`, or to any preset when None) in `scope` becomes the
/// preset's value as it is now; a link the preset gives nothing becomes a release. Returns how
/// many values changed.
pub fn extract_cues(show: &mut ShowData, preset: Option<Uuid>, scope: CueScope) -> usize {
    // Work out every value first (the show is read), then write.
    let mut writes: Vec<(u32, u32, u32, u32, usize, Value)> = Vec::new();
    for (dp, no, seq) in show.sequences() {
        for (&key, cue) in std::iter::once((&lumacore_tree::CUE_ZERO, &seq.cue_zero)).chain(seq.cues.iter()) {
            if !scope.holds(dp, no, key) {
                continue;
            }
            for (&part_no, part) in &cue.parts {
                for (i, v) in part.values.iter().enumerate() {
                    let Some(g) = v.value.link() else { continue };
                    if preset.is_some_and(|p| p != g) {
                        continue;
                    }
                    let value = lumacore_tree::PresetValues::preset_value(show, g, v.fixture_id, &v.attribute).map_or_else(Value::release, plain);
                    writes.push((dp, no, key, part_no, i, value));
                }
            }
        }
    }
    let count = writes.len();
    for (dp, no, key, part_no, i, value) in writes {
        let Some(seq) = show.sequence_mut(dp, no) else { continue };
        let cue = if key == lumacore_tree::CUE_ZERO { Some(&mut seq.cue_zero) } else { seq.cues.get_mut(&key) };
        if let Some(v) = cue.and_then(|c| c.parts.get_mut(&part_no)).and_then(|p| p.values.get_mut(i)) {
            v.value = CueData::Value(value);
        }
    }
    count
}

/// Extract in the acting user's programmer: its values linked to one of `presets` become plain
/// values (the preset's as it is now). Returns how many.
pub fn extract_programmer(state: &mut AppState, presets: &[Uuid]) -> usize {
    let user = state.effective_user_name().to_string();
    let show = &state.root.show_data;
    let mut now: Vec<((Uuid, String), Option<Value>)> = Vec::new();
    if let Some(prog) = show.programmer_of(&user) {
        for ((f, a), pv) in &prog.values {
            if let Some(g) = pv.value.preset.filter(|g| presets.contains(g)) {
                now.push(((*f, a.clone()), lumacore_tree::PresetValues::preset_value(show, g, *f, a).cloned()));
            }
        }
    }
    let Some(prog) = state.root.show_data.programmer_of_mut(&user) else { return 0 };
    for (key, value) in &now {
        if let Some(pv) = prog.values.get_mut(key) {
            if let Some(v) = value {
                pv.value.steps.clone_from(&v.steps);
            }
            pv.value.preset = None;
        }
    }
    now.len()
}

/// The cues a value names (`Sequence 4`, `Sequence 4 Cue 9`, owner first): a scope.
pub fn cue_scope(state: &mut AppState, value: &lumacore_syntax::ast::Value) -> Result<CueScope, String> {
    let lumacore_syntax::ast::Value::Target(t) = value else {
        return Err(render_code("Name a sequence or a cue: Sequence 4, or Sequence 4 Cue 9.", "E-CMD-008"));
    };
    let names_cue = match t.items.as_slice() {
        [(None, lumacore_syntax::ast::Item::Path(segs))] => segs.last().is_some_and(|l| {
            crate::objects::find(&crate::lum::vocabulary().word(l.object.node).name).is_some_and(|o| std::ptr::eq(o, &CUE))
        }),
        _ => false,
    };
    if names_cue {
        let (r, owners) = crate::resolve::resolve_owned(state, t, &CUE)?;
        let seq_no = owners
            .iter()
            .find(|(o, _)| std::ptr::eq(*o, &SEQUENCE))
            .and_then(|(_, k)| k.parse::<u32>().ok())
            .ok_or_else(|| render_code("Name the cue's sequence: Sequence 4 Cue 9.", "E-CMD-006"))?;
        let dp = r.datapool.or(state.datapool_override).unwrap_or_else(|| state.selected_datapool());
        let cue = match r.members.as_slice() {
            [one] => one.name.as_deref().and_then(crate::commands::playback::cue_ref),
            _ => None,
        }
        .ok_or_else(|| render_code("Name one cue that exists (List Cue).", "E-PLAY-001"))?;
        return Ok(CueScope::Cue { dp, no: seq_no, cue });
    }
    let r = crate::resolve::resolve_one(state, t, &SEQUENCE)?;
    if !std::ptr::eq(r.object, &SEQUENCE) {
        return Err(render_code(format!("Name a sequence or a cue, not {}.", r.object.keyword), "E-CMD-008"));
    }
    let dp = r.datapool.unwrap_or_else(|| state.selected_datapool());
    match r.members.as_slice() {
        [one] => Ok(CueScope::Sequence { dp, no: one.no }),
        [] => Err(render_code("That sequence doesn't exist.", "E-CMD-007")),
        _ => Err(render_code("Name one sequence.", "E-CMD-008")),
    }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// A value as a preset or a cue keeps it: its steps, no link.
pub fn plain(v: &Value) -> Value {
    let mut v = v.clone();
    v.preset = None;
    v
}

/// Whether the fixture's mode has the attribute (a universal value never lands on one without).
fn fixture_has(show: &ShowData, fixture: Uuid, attribute: &str) -> bool {
    show.devices
        .dmx
        .patch
        .fixtures
        .values()
        .find(|f| f.meta.identity.guid == fixture)
        .is_some_and(|f| lumacore_render::merge::channels(show, f).iter().any(|cf| cf.attribute_name.eq_ignore_ascii_case(attribute)))
}

/// How a line names a preset of DataPool `dp` for the acting user: `Preset 3`, or owner first
/// (`DataPool 2 Preset 3`) when it isn't the user's DataPool.
pub fn preset_line(state: &AppState, dp: u32, no: u32) -> String {
    format!("{}Preset {no}", owner(state, dp))
}

/// How a line names a cue: `Sequence 1 Cue 2` (owner first for another DataPool).
pub fn cue_line(state: &AppState, dp: u32, seq: u32, cue: u32) -> String {
    let cue = if cue == lumacore_tree::CUE_ZERO { "0".to_string() } else { lumacore_tree::cue_label(cue) };
    format!("{}Sequence {seq} Cue {cue}", owner(state, dp))
}

/// `DataPool <n> ` when `dp` isn't the acting user's DataPool (owner first), else nothing.
pub fn owner(state: &AppState, dp: u32) -> String {
    if dp == crate::pools::user_datapool(&state.root.show_data, state.effective_user_name()) { String::new() } else { format!("DataPool {dp} ") }
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
